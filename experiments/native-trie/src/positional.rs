//! Independent concrete positional encodings. Value occurrence indexing is generic reference
//! resolution, not a rotation/translation catalogue. Ambiguous correspondences remain recorded.
use crate::arc_grids::{Grid, GridError, Grids};
use crate::records::{self, RecordError, Selection};
use crate::{Address, Fault, Machine, Trie, install, source_field, target_field};
#[derive(Debug)]
pub enum Error {
    WorkspaceLimit,
    Grid(GridError),
    Record(RecordError),
    Region(crate::region_witness::RegionError),
    Fault(Fault),
    Persistent(crate::disk_trie::DiskFault),
    Shape,
    NotReady,
    Ambiguous,
    CycleLimit,
    ValidationMismatch,
    Incomplete,
}
impl From<crate::region_witness::RegionError> for Error {
    fn from(e: crate::region_witness::RegionError) -> Self {
        Self::Region(e)
    }
}
impl From<GridError> for Error {
    fn from(e: GridError) -> Self {
        Self::Grid(e)
    }
}
impl From<RecordError> for Error {
    fn from(e: RecordError) -> Self {
        Self::Record(e)
    }
}
impl From<Fault> for Error {
    fn from(e: Fault) -> Self {
        Self::Fault(e)
    }
}
impl From<crate::disk_trie::DiskFault> for Error {
    fn from(e: crate::disk_trie::DiskFault) -> Self {
        Self::Persistent(e)
    }
}
fn field(a: &Address, k: &[u8]) -> Result<Address, Error> {
    a.child(k).ok_or(Error::Incomplete)
}
#[derive(Clone)]
pub struct Encoding {
    pub root: Address,
    owner: Address,
    width: usize,
    height: usize,
    entry: Address,
    halt: Address,
}
pub struct Derived {
    owner: Address,
    width: usize,
    height: usize,
    pub evidence: Address,
    pub encoding: Option<Encoding>,
    pub unresolved: usize,
}
pub struct Positional {
    pub root: Address,
    pub machine: Machine,
    frozen: bool,
    active: Option<(Grid, Encoding, Vec<Address>)>,
    serial: usize,
}
impl Default for Positional {
    fn default() -> Self {
        Self {
            root: Trie::default().root(),
            machine: Machine::default(),
            frozen: false,
            active: None,
            serial: 0,
        }
    }
}
impl Positional {
    pub fn agree(derived: &[Derived]) -> Option<Encoding> {
        if derived.len() < 2 {
            return None;
        }
        let first = derived.first()?.encoding.as_ref()?;
        if derived.iter().all(|d| {
            d.encoding
                .as_ref()
                .is_some_and(|e| e.root.same(&first.root))
        }) {
            Some(first.clone())
        } else {
            None
        }
    }
    fn position(&self, y: usize, x: usize) -> Address {
        self.root
            .provision(b"positions")
            .provision(&y.to_be_bytes())
            .provision(&x.to_be_bytes())
    }
    pub fn derive(&mut self, id: &[u8], input: &Grid, output: &Grid) -> Result<Derived, Error> {
        if self.frozen {
            return Err(Error::NotReady);
        }
        if !input.owner.same(&output.owner) {
            return Err(GridError::ForeignSpace.into());
        }
        if input.width() != output.width() || input.height() != output.height() {
            return Err(Error::Shape);
        }
        let episodes = self.root.provision(b"derivations");
        if episodes.child(id).is_some() {
            return Err(Error::NotReady);
        }
        let evidence = episodes.provision(id);
        evidence.provision(b"input").point_to(&input.anchor);
        evidence.provision(b"output").point_to(&output.anchor);
        let index = evidence.provision(b"by-value");
        for y in 0..input.height() {
            for x in 0..input.width() {
                let cell = input.cell(y, x)?;
                let v = field(&cell, b"value")?.follow()?;
                records::record(
                    &index.provision(&v.identity_key()),
                    b"position",
                    &self.position(y, x),
                    &cell,
                )?;
            }
        }
        let mut path = self
            .root
            .provision(b"encodings")
            .provision(&input.height().to_be_bytes())
            .provision(&input.width().to_be_bytes());
        let mut pairs = Vec::new();
        let mut unresolved = 0;
        for y in 0..output.height() {
            for x in 0..output.width() {
                let cell = output.cell(y, x)?;
                let v = field(&cell, b"value")?.follow()?;
                let at = evidence
                    .provision(b"correspondences")
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes());
                at.provision(b"output-cell").point_to(&cell);
                let dest = self.position(y, x);
                let anchor = index.provision(&v.identity_key());
                at.provision(b"lookup").point_to(&anchor);
                match records::resolve(&anchor, b"position")? {
                    Selection::Unique { target, .. } => {
                        at.provision(b"source-position").point_to(&target);
                        path = path
                            .provision(&dest.identity_key())
                            .provision(&target.identity_key());
                        pairs.push((dest, target));
                    }
                    _ => {
                        unresolved += 1;
                        at.provision(b"unresolved").terminate();
                    }
                }
            }
        }
        if unresolved != 0 {
            return Ok(Derived {
                owner: self.root.clone(),
                width: input.width(),
                height: input.height(),
                evidence,
                encoding: None,
                unresolved,
            });
        }
        let encoding = self.assemble(path, &pairs, input.width(), input.height());
        records::record(&encoding.root, b"support", &encoding.root, &evidence)?;
        evidence.provision(b"encoding").point_to(&encoding.root);
        Ok(Derived {
            owner: self.root.clone(),
            width: input.width(),
            height: input.height(),
            evidence,
            encoding: Some(encoding),
            unresolved: 0,
        })
    }
    /// Compare compact sets of positional constructions, never their Cartesian product.
    /// Every demonstration is mandatory; absent source support is NOT an unconstrained head.
    pub fn agree_candidates(&mut self, derived: &[Derived]) -> Result<Option<Encoding>, Error> {
        if self.frozen {
            return Err(Error::NotReady);
        }
        if derived.len() < 2 {
            return Ok(None);
        }
        let first = &derived[0];
        if derived.iter().any(|d| {
            !d.owner.same(&self.root) || d.width != first.width || d.height != first.height
        }) {
            return Ok(None);
        }
        let mut path = self
            .root
            .provision(b"encodings")
            .provision(&first.height.to_be_bytes())
            .provision(&first.width.to_be_bytes());
        let mut pairs = Vec::new();
        for y in 0..first.height {
            for x in 0..first.width {
                let mut heads = Vec::new();
                for d in derived {
                    let at = field(
                        &field(&field(&d.evidence, b"correspondences")?, &y.to_be_bytes())?,
                        &x.to_be_bytes(),
                    )?;
                    let anchor = field(&at, b"lookup")?.follow()?;
                    if matches!(records::resolve(&anchor, b"position")?, Selection::Unknown) {
                        return Ok(None);
                    }
                    heads.push(anchor);
                }
                match records::intersect(&heads.iter().map(Some).collect::<Vec<_>>(), b"position")?
                {
                    Selection::Unique { target, .. } => {
                        let dest = self.position(y, x);
                        path = path
                            .provision(&dest.identity_key())
                            .provision(&target.identity_key());
                        pairs.push((dest, target));
                    }
                    _ => return Ok(None),
                }
            }
        }
        let encoding = self.assemble(path, &pairs, first.width, first.height);
        for d in derived {
            records::record(&encoding.root, b"support", &encoding.root, &d.evidence)?;
            d.evidence
                .provision(b"compatible-encoding")
                .point_to(&encoding.root);
        }
        Ok(Some(encoding))
    }
    fn assemble(
        &self,
        path: Address,
        pairs: &[(Address, Address)],
        width: usize,
        height: usize,
    ) -> Encoding {
        let code = path.provision(b"program");
        let halt = code.provision(b"halt");
        halt.terminate();
        let mut next = halt.clone();
        for (i, (dest, src)) in pairs.iter().enumerate().rev() {
            let part = code.provision(&i.to_be_bytes());
            let emit = part.provision(b"copy");
            let patch_target = part.provision(b"bind-target");
            let patch_source = part.provision(b"bind-source");
            let placeholder = part.provision(b"placeholder");
            let link = |a: &Address, k: &[u8], n: &Address| {
                let c = a.provision(k);
                c.point_to(n);
                c
            };
            install(
                &emit,
                &placeholder,
                &placeholder,
                &link(&part, b"after-copy", &next),
            );
            install(
                &patch_target,
                &dest.provision(b"output-slot"),
                &target_field(&emit).unwrap(),
                &link(&part, b"to-copy", &emit),
            );
            install(
                &patch_source,
                &src.provision(b"input-slot"),
                &source_field(&emit).unwrap(),
                &link(&part, b"to-target", &patch_target),
            );
            next = patch_source;
        }
        path.provision(b"entry").point_to(&next);
        Encoding {
            owner: self.root.clone(),
            root: path,
            width,
            height,
            entry: next,
            halt,
        }
    }
    pub fn begin(&mut self, input: &Grid, e: &Encoding) -> Result<(), Error> {
        if !self.root.same(&e.owner) {
            return Err(Error::NotReady);
        }
        if self.active.is_some() {
            return Err(Error::NotReady);
        }
        if input.width() != e.width || input.height() != e.height {
            return Err(Error::Shape);
        }
        self.frozen = true;
        let run = self
            .root
            .provision(b"runs")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        run.provision(b"input").point_to(&input.anchor);
        run.provision(b"encoding").point_to(&e.root);
        let mut rows = Vec::new();
        for y in 0..e.height {
            let row = run.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..e.width {
                let out = row
                    .provision(b"cells")
                    .provision(&x.to_be_bytes())
                    .provision(b"value");
                let p = self.position(y, x);
                p.provision(b"input-slot")
                    .point_to(&field(&input.cell(y, x)?, b"value")?);
                p.provision(b"output-slot").point_to(&out);
            }
            rows.push(row);
        }
        self.machine.enter(&e.entry);
        self.active = Some((input.clone(), e.clone(), rows));
        Ok(())
    }
    pub fn execute(&mut self, limit: usize) -> Result<u64, Error> {
        let (_, e, _) = self.active.as_ref().ok_or(Error::NotReady)?;
        for _ in 0..limit {
            if self.machine.current()?.same(&e.halt) {
                return Ok(self.machine.cycles);
            }
            self.machine.cycle()?;
        }
        if self.machine.current()?.same(&e.halt) {
            Ok(self.machine.cycles)
        } else {
            Err(Error::Incomplete)
        }
    }
    pub fn record(&mut self, grids: &mut Grids) -> Result<Grid, Error> {
        let (input, e, rows) = self.active.as_ref().ok_or(Error::NotReady)?;
        if !self.machine.current()?.same(&e.halt) {
            return Err(Error::Incomplete);
        }
        let result = grids.derived(input, &e.root, rows)?;
        self.active = None;
        Ok(result)
    }
}
