//! Specific pointwise color family: learn exact trait associations from supplied pairs.
//! No host color table, image transform, inferred identity, or task-ID dispatch.
use crate::{
    Address, Fault, Machine,
    arc_grids::{Grid, GridError, Grids},
    install,
    records::{self, RecordError, Selection},
    source_field, target_field,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorError {
    Grid(GridError),
    Record(RecordError),
    Machine(Fault),
    Duplicate,
    Limit,
    NotComplete,
    Region(crate::region_witness::RegionError),
    WorkspaceLimit,
    Branch(crate::reference_branch::BranchError),
}
impl From<GridError> for ColorError {
    fn from(e: GridError) -> Self {
        Self::Grid(e)
    }
}
impl From<RecordError> for ColorError {
    fn from(e: RecordError) -> Self {
        Self::Record(e)
    }
}
impl From<Fault> for ColorError {
    fn from(e: Fault) -> Self {
        Self::Machine(e)
    }
}
/// Native reference agreement only; no palette/geometry classification.
pub(crate) enum Choice {
    Unknown,
    Unique { candidate: Address, target: Address },
    Ambiguous,
}
impl From<Selection> for Choice {
    fn from(value: Selection) -> Self {
        match value {
            Selection::Unknown => Self::Unknown,
            Selection::Unique { candidate, target } => Self::Unique { candidate, target },
            Selection::Ambiguous { .. } => Self::Ambiguous,
        }
    }
}
impl Choice {
    pub(crate) fn agree(self, other: Self) -> Self {
        match (self, other) {
            (Self::Ambiguous, _) | (_, Self::Ambiguous) => Self::Ambiguous,
            (Self::Unknown, right) => right,
            (left, Self::Unknown) => left,
            (Self::Unique { candidate, target }, Self::Unique { target: other, .. }) => {
                if target.same(&other) {
                    Self::Unique { candidate, target }
                } else {
                    Self::Ambiguous
                }
            }
        }
    }
}
pub(crate) struct Cell {
    pub(crate) input: Address,
    pub(crate) node: Address,
    output: Address,
    pub(crate) lookup: Address,
    candidate: Address,
    status: Address,
}
pub struct ColorRun {
    model: Address,
    pub(crate) root: Address,
    indirect: bool,
    pub(crate) input: Grid,
    rows: Vec<Address>,
    pub(crate) cells: Vec<Cell>,
    at: usize,
    pub cycles: u64,
    pub relation_copies: u64,
    pub relation_setup_ns: u128,
    pub relation_execute_ns: u128,
    pub relation_cache_hits: usize,
    pub relation_proofs: usize,
    pub unknown: usize,
    pub ambiguous: usize,
}
pub struct ColorModel {
    pub root: Address,
    pub machine: Machine,
    pub construction: Address,
    pub source_construction: Address,
    source_emit: Address,
    unknown_source: Address,
    selected: Address,
    halt: Address,
    owner: Address,
    unknown: Address,
    statuses: [Address; 3],
}
impl ColorModel {
    pub fn new(grids: &Grids, scope: &[u8]) -> Result<Self, ColorError> {
        let owner = grids.storage.root();
        let models = owner.provision(b"arc-color-models");
        if models.child(scope).is_some() {
            return Err(ColorError::Duplicate);
        }
        let root = models.provision(scope);
        let selected = root.provision(b"selected-value");
        let scratch = root.provision(b"scratch");
        let halt = root.provision(b"halt");
        halt.terminate();
        let next = root.provision(b"return");
        next.point_to(&halt);
        let construction = root
            .provision(b"construction")
            .provision(b"emit-selected-reference");
        install(&construction, &selected, &scratch, &next);
        let unknown_source = root.provision(b"unresolved-source");
        unknown_source.point_to(&grids.unresolved());
        let source_emit = root
            .provision(b"construction")
            .provision(b"copy-from-source-slot");
        install(&source_emit, &unknown_source, &scratch, &next);
        let source_construction = root
            .provision(b"construction")
            .provision(b"bind-selected-source-slot");
        let source_next = root.provision(b"source-emission-continuation");
        source_next.point_to(&source_emit);
        install(
            &source_construction,
            &selected,
            &source_field(&source_emit).unwrap(),
            &source_next,
        );
        let statuses = [
            root.provision(b"unique"),
            root.provision(b"unknown"),
            root.provision(b"ambiguous"),
        ];
        Ok(Self {
            root,
            machine: Machine::default(),
            construction,
            source_construction,
            source_emit,
            unknown_source,
            selected,
            halt,
            owner,
            unknown: grids.unresolved(),
            statuses,
        })
    }
    fn anchor(&self, value: &Address) -> Option<Address> {
        self.root
            .child(b"traits")?
            .child(b"color")?
            .child(&value.identity_key())
    }
    pub fn lookup(&self, value: &Address) -> Result<Selection, ColorError> {
        match self.anchor(value) {
            Some(a) => Ok(records::resolve(&a, b"output")?),
            None => Ok(Selection::Unknown),
        }
    }
    /// All paired cells, including unchanged ones, contribute actual observations.
    /// Same topology is this family's explicit grammar, not a universal ARC restriction.
    pub fn teach(&self, id: &[u8], input: &Grid, output: &Grid) -> Result<(), ColorError> {
        if !input.owner.same(&self.owner) || !output.owner.same(&self.owner) {
            return Err(GridError::ForeignSpace.into());
        }
        if input.width() != output.width() || input.height() != output.height() {
            return Err(GridError::InvalidShape.into());
        }
        let episodes = self.root.provision(b"examples");
        if let Some(old) = episodes.child(id)
            && (!old
                .child(b"input")
                .ok_or(GridError::Malformed)?
                .follow()?
                .same(&input.anchor)
                || !old
                    .child(b"output")
                    .ok_or(GridError::Malformed)?
                    .follow()?
                    .same(&output.anchor))
        {
            return Err(ColorError::Duplicate);
        }
        let episode = episodes.provision(id);
        episode.provision(b"input").point_to(&input.anchor);
        episode.provision(b"output").point_to(&output.anchor);
        for y in 0..input.height() {
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let after = output.cell(y, x)?;
                let value = before
                    .child(b"value")
                    .ok_or(GridError::Malformed)?
                    .follow()?;
                let target = after
                    .child(b"value")
                    .ok_or(GridError::Malformed)?
                    .follow()?;
                let anchor = self
                    .root
                    .provision(b"traits")
                    .provision(b"color")
                    .provision(&value.identity_key());
                anchor.provision(b"observed-color").point_to(&value);
                anchor
                    .provision(b"construction")
                    .point_to(&self.construction);
                let evidence = episode
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes());
                evidence.provision(b"input-cell").point_to(&before);
                evidence.provision(b"output-cell").point_to(&after);
                evidence
                    .provision(b"construction")
                    .point_to(&self.construction);
                records::record(&anchor, b"output", &target, &evidence)?;
            }
        }
        Ok(())
    }
    /// Input/workspace materialization only. No mapping or answers are supplied by the adapter.
    pub fn begin(&self, id: &[u8], input: &Grid) -> Result<ColorRun, ColorError> {
        if !input.owner.same(&self.owner) {
            return Err(GridError::ForeignSpace.into());
        }
        let runs = self.root.provision(b"runs");
        if runs.child(id).is_some() {
            return Err(ColorError::Duplicate);
        }
        let root = runs.provision(id);
        root.provision(b"input").point_to(&input.anchor);
        root.provision(b"construction").point_to(&self.construction);
        let mut rows = Vec::new();
        let mut cells = Vec::new();
        for y in 0..input.height() {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            row.provision(b"input-row").point_to(&input.row(y)?);
            for x in 0..input.width() {
                let work = row.provision(b"cells").provision(&x.to_be_bytes());
                let source = input.cell(y, x)?;
                work.provision(b"input-cell").point_to(&source);
                work.provision(b"construction").point_to(&self.construction);
                cells.push(Cell {
                    input: source,
                    node: work.clone(),
                    output: work.provision(b"value"),
                    lookup: work.provision(b"lookup"),
                    candidate: work.provision(b"candidate"),
                    status: work.provision(b"status"),
                });
            }
            rows.push(row);
        }
        Ok(ColorRun {
            model: self.root.clone(),
            root,
            indirect: false,
            input: input.clone(),
            rows,
            cells,
            at: 0,
            cycles: 0,
            relation_copies: 0,
            relation_setup_ns: 0,
            relation_execute_ns: 0,
            relation_cache_hits: 0,
            relation_proofs: 0,
            unknown: 0,
            ambiguous: 0,
        })
    }
    /// Actual trie lookup then one stored native copy per cell. No allocation or palette decode.
    pub fn execute(&mut self, run: &mut ColorRun, limit: usize) -> Result<(), ColorError> {
        let root = self.root.clone();
        self.execute_with(run, limit, |cell| {
            let value = cell
                .input
                .child(b"value")
                .ok_or(GridError::Malformed)?
                .follow()?;
            let anchor = root
                .child(b"traits")
                .and_then(|a| a.child(b"color"))
                .and_then(|a| a.child(&value.identity_key()));
            Ok(match anchor {
                Some(anchor) => {
                    cell.lookup.point_to(&anchor);
                    records::resolve(&anchor, b"output")?.into()
                }
                None => Choice::Unknown,
            })
        })
    }
    /// Shared emission/controller. The resolver performs physical addressing and reference
    /// agreement; it never executes a host transformation or computes a palette value.
    pub(crate) fn execute_with(
        &mut self,
        run: &mut ColorRun,
        limit: usize,
        resolve: impl FnMut(&Cell) -> Result<Choice, ColorError>,
    ) -> Result<(), ColorError> {
        self.execute_mode(run, limit, resolve, false)
    }
    pub(crate) fn execute_sources_with(
        &mut self,
        run: &mut ColorRun,
        limit: usize,
        resolve: impl FnMut(&Cell) -> Result<Choice, ColorError>,
    ) -> Result<(), ColorError> {
        self.execute_mode(run, limit, resolve, true)
    }
    fn execute_mode(
        &mut self,
        run: &mut ColorRun,
        limit: usize,
        mut resolve: impl FnMut(&Cell) -> Result<Choice, ColorError>,
        indirect: bool,
    ) -> Result<(), ColorError> {
        if !run.model.same(&self.root) {
            return Err(GridError::ForeignSpace.into());
        }
        if run.at > 0 && run.indirect != indirect {
            return Err(ColorError::NotComplete);
        }
        run.indirect = indirect;
        let entry = if indirect {
            &self.source_construction
        } else {
            &self.construction
        };
        let emit = if indirect {
            &self.source_emit
        } else {
            &self.construction
        };
        run.root.provision(b"construction").point_to(entry);
        for _ in 0..limit {
            if run.at == run.cells.len() {
                return Ok(());
            }
            let cell = &run.cells[run.at];
            let selection = resolve(cell)?;
            let (target, status) = match selection {
                Choice::Unique { candidate, target } => {
                    cell.candidate.point_to(&candidate);
                    (target, &self.statuses[0])
                }
                Choice::Unknown => {
                    run.unknown += 1;
                    (
                        if indirect {
                            self.unknown_source.clone()
                        } else {
                            self.unknown.clone()
                        },
                        &self.statuses[1],
                    )
                }
                Choice::Ambiguous => {
                    run.ambiguous += 1;
                    (
                        if indirect {
                            self.unknown_source.clone()
                        } else {
                            self.unknown.clone()
                        },
                        &self.statuses[2],
                    )
                }
            };
            cell.status.point_to(status);
            self.selected.point_to(&target);
            cell.node.provision(b"construction").point_to(entry);
            target_field(emit)
                .ok_or(GridError::Malformed)?
                .point_to(&cell.output);
            self.machine.enter(entry);
            self.machine.cycle()?;
            if indirect {
                self.machine.cycle()?;
            }
            if !self.machine.current()?.same(&self.halt) {
                return Err(ColorError::NotComplete);
            }
            run.at += 1;
            run.cycles += if indirect { 2 } else { 1 };
        }
        if run.at == run.cells.len() {
            Ok(())
        } else {
            Err(ColorError::Limit)
        }
    }
    /// Fresh derived occurrences are recorded only after every cell completes.
    pub fn record(&self, grids: &mut Grids, run: ColorRun) -> Result<Grid, ColorError> {
        if !run.model.same(&self.root) || !self.owner.same(&grids.storage.root()) {
            return Err(GridError::ForeignSpace.into());
        }
        if run.at != run.cells.len() {
            return Err(ColorError::NotComplete);
        }
        Ok(grids.derived(
            &run.input,
            if run.indirect {
                &self.source_construction
            } else {
                &self.construction
            },
            &run.rows,
        )?)
    }
}
