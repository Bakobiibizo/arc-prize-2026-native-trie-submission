//! Specific paired singleton-color source-slot family; computation remains stored native code.
use crate::{
    Address,
    arc_colors::{Choice, ColorError, ColorModel, ColorRun},
    arc_grids::{Grid, GridError, Grids},
    records::{self, Selection},
    reference_branch::{BranchError, ReferenceBranch},
    region_witness::{RegionError, RegionWitness},
    singleton_witness::SingletonWitness,
};
use std::time::Instant;
impl From<BranchError> for ColorError {
    fn from(e: BranchError) -> Self {
        Self::Branch(e)
    }
}
fn field(a: &Address, k: &[u8]) -> Result<Address, ColorError> {
    a.child(k).ok_or(GridError::Malformed.into())
}
#[derive(Default, Clone)]
pub struct MarkerStats {
    pub region_copies: u64,
    pub role_copies: u64,
    pub expression_copies: u64,
    pub region_setup_ns: u128,
    pub region_execute_ns: u128,
    pub role_setup_ns: u128,
    pub role_execute_ns: u128,
    pub expression_setup_ns: u128,
    pub expression_execute_ns: u128,
    pub guard_ns: u128,
    pub peak_charge: usize,
    pub proofs: usize,
    pub cache_hits: usize,
}
pub struct MarkerModel {
    pub base: ColorModel,
    pub comparison: ReferenceBranch,
    pub marker_source: Address,
    pub last: MarkerStats,
    pub max_workspace_bytes: usize,
    serial: usize,
    unresolved: Address,
    active: Option<Address>,
    queried: bool,
    constraints: bool,
}
impl MarkerModel {
    pub fn new(grids: &Grids, scope: &[u8]) -> Result<Self, ColorError> {
        let base = ColorModel::new(grids, scope)?;
        RegionWitness::cardinality_zero(&base.root);
        let marker_source = base.root.provision(b"current-singleton-color-source");
        marker_source.point_to(&grids.unresolved());
        Ok(Self {
            base,
            comparison: ReferenceBranch::new(),
            marker_source,
            last: MarkerStats::default(),
            max_workspace_bytes: 286720000,
            serial: 0,
            unresolved: grids.unresolved(),
            active: None,
            queried: false,
            constraints: false,
        })
    }
    /// Separate opt-in reference-constraint grammar; strict marker family is unchanged.
    pub fn new_constraints(grids: &Grids, scope: &[u8]) -> Result<Self, ColorError> {
        let mut m = Self::new(grids, scope)?;
        m.constraints = true;
        m.base
            .root
            .provision(b"inference-grammar")
            .provision(b"source-reference-constraints-v2-native-applicability");
        Ok(m)
    }
    fn guard(&mut self, reserve: usize) -> Result<(), ColorError> {
        let t = Instant::now();
        let charge = RegionWitness::storage_charge(&self.base.root)
            + RegionWitness::storage_charge(&self.comparison.machine.states.root())
            + RegionWitness::storage_charge(&self.comparison.machine.output.root())
            + reserve;
        self.last.guard_ns += t.elapsed().as_nanos();
        self.last.peak_charge = self.last.peak_charge.max(charge);
        if charge > self.max_workspace_bytes {
            Err(ColorError::WorkspaceLimit)
        } else {
            Ok(())
        }
    }
    fn derive(
        &mut self,
        input: &Grid,
        cell: &Address,
    ) -> Result<(Address, Address, Address), ColorError> {
        let cache = self
            .base
            .root
            .provision(b"counted-cache")
            .provision(&input.anchor.identity_key());
        let (proof, kind, role) = if let Some((proof, kind)) = RegionWitness::cached(&cache, cell)?
        {
            let role = field(
                &self.base.root.provision(b"singleton-cache"),
                &proof.identity_key(),
            )?
            .follow()?;
            if !role.child(b"committed").is_some_and(|c| c.is_terminal()) {
                return Err(RegionError::Incomplete.into());
            }
            self.last.cache_hits += 1;
            (proof, kind, role)
        } else {
            let views = input.width() * input.height() + 1;
            if views > 8192 {
                return Err(ColorError::WorkspaceLimit);
            }
            self.guard(views * 38000)?;
            let t = Instant::now();
            let mut w = RegionWitness::prepare_counted_bound(
                input,
                cell,
                &self.base.root,
                &self.serial.to_be_bytes(),
                &cache,
            )?;
            self.serial += 1;
            self.last.region_setup_ns += t.elapsed().as_nanos();
            let t = Instant::now();
            self.last.region_copies += w.execute(w.budget)?;
            self.last.region_execute_ns += t.elapsed().as_nanos();
            let p = w.record()?;
            let proof = p.proof.clone();
            let kind = p.value.clone();
            let facts = p.recorded_views;
            self.guard((32 * facts + 128) * 512)?;
            let t = Instant::now();
            let mut w = SingletonWitness::prepare(p, &self.base.root, &self.serial.to_be_bytes())?;
            self.serial += 1;
            self.last.role_setup_ns += t.elapsed().as_nanos();
            let t = Instant::now();
            self.last.role_copies += w.execute(w.budget)?;
            self.last.role_execute_ns += t.elapsed().as_nanos();
            let p = w.record()?;
            let role = p.proof;
            self.base
                .root
                .provision(b"singleton-cache")
                .provision(&proof.identity_key())
                .point_to(&role);
            self.last.proofs += 1;
            (proof, kind, role)
        };
        let status = field(&role, b"status")?.follow()?;
        Ok((proof, kind, status))
    }
    fn census(&mut self, input: &Grid) -> Result<Address, ColorError> {
        let root = self
            .base
            .root
            .provision(b"marker-inputs")
            .provision(&input.anchor.identity_key());
        root.provision(b"input").point_to(&input.anchor);
        let candidates = root.provision(b"singleton-color-candidates");
        for y in 0..input.height() {
            for x in 0..input.width() {
                let cell = input.cell(y, x)?;
                let (proof, kind, status) = self.derive(input, &cell)?;
                let role = field(
                    &self.base.root.provision(b"singleton-cache"),
                    &proof.identity_key(),
                )?
                .follow()?;
                let node = root
                    .provision(b"cells")
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes());
                node.provision(b"region-proof").point_to(&proof);
                node.provision(b"singleton-proof").point_to(&role);
                node.provision(b"kind").point_to(&kind);
                node.provision(b"size-status").point_to(&status);
                root.provision(b"by-source")
                    .provision(&cell.identity_key())
                    .point_to(&node);
                match field(&role, b"candidate")?.follow() {
                    Ok(color) => {
                        records::record(&candidates, b"value", &color, &role)?;
                    }
                    Err(crate::Fault::Unset) => {}
                    Err(e) => return Err(e.into()),
                }
            }
        }
        // Generic evidence resolution/binding only; candidates were emitted by native predicates.
        let bound = match records::resolve(&candidates, b"value")? {
            Selection::Unique { target, .. } => target,
            Selection::Unknown | Selection::Ambiguous { .. } => self.unresolved.clone(),
        };
        self.marker_source.point_to(&bound);
        root.provision(b"bound-value").point_to(&bound);
        if self.constraints {
            self.guard(160 * 512)?;
            let start = Instant::now();
            let scopes = self.base.root.provision(b"source-head-scopes");
            let defined = scopes.provision(b"role-bound");
            let missing = scopes.provision(b"role-unbound");
            let active = self.base.root.provision(b"applicable-marker-head");
            let inactive = self.base.root.provision(b"inactive-marker-observations");
            let empty = self.base.root.provision(b"inactive-marker-query-prefix");
            for (scope, record, query) in
                [(&defined, &active, &active), (&missing, &inactive, &empty)]
            {
                scope.provision(b"record-prefix").point_to(record);
                scope.provision(b"query-prefix").point_to(query);
            }
            self.comparison.set_outputs(&missing, &defined)?;
            self.comparison.bind(&bound, &self.unresolved);
            self.last.role_setup_ns += start.elapsed().as_nanos();
            let start = Instant::now();
            self.last.role_copies += self.comparison.execute(6)?;
            self.last.role_execute_ns += start.elapsed().as_nanos();
            let applicability = self.comparison.record_compact()?;
            applicability
                .evidence
                .provision(b"input-census")
                .point_to(&root);
            applicability
                .evidence
                .provision(b"bound-marker-value")
                .point_to(&bound);
            root.provision(b"head-scope").point_to(&applicability.value);
            root.provision(b"applicability-proof")
                .point_to(&applicability.evidence);
        }
        self.guard(0)?;
        Ok(root)
    }
    pub fn teach(&mut self, id: &[u8], input: &Grid, output: &Grid) -> Result<(), ColorError> {
        if self.queried || self.active.is_some() {
            return Err(ColorError::NotComplete);
        }
        self.last = MarkerStats::default();
        self.guard(input.width() * input.height() * 64 * 512)?;
        self.base.teach(id, input, output)?;
        let census = self.census(input)?;
        let marker = self.marker_source.follow()?;
        for y in 0..input.height() {
            for x in 0..input.width() {
                self.guard(160 * 512)?;
                let before = input.cell(y, x)?;
                let after = output.cell(y, x)?;
                let value = field(&after, b"value")?.follow()?;
                let literal = self
                    .base
                    .root
                    .provision(b"literal-color-sources")
                    .provision(&value.identity_key());
                literal.point_to(&value);
                let context = field(
                    &field(&field(&census, b"cells")?, &y.to_be_bytes())?,
                    &x.to_be_bytes(),
                )?;
                let kind = field(&context, b"kind")?.follow()?;
                let size = field(&context, b"size-status")?.follow()?;
                let t = Instant::now();
                self.comparison.set_outputs(&self.marker_source, &literal)?;
                self.comparison.bind(&value, &marker);
                self.last.expression_setup_ns += t.elapsed().as_nanos();
                let t = Instant::now();
                self.last.expression_copies += self.comparison.execute(6)?;
                self.last.expression_execute_ns += t.elapsed().as_nanos();
                let expr = if self.constraints {
                    self.comparison.record_compact()?
                } else {
                    self.comparison.record()?
                };
                expr.evidence.provision(b"input-cell").point_to(&before);
                expr.evidence.provision(b"output-cell").point_to(&after);
                expr.evidence.provision(b"input-census").point_to(&census);
                expr.evidence
                    .provision(b"source-context")
                    .point_to(&context);
                expr.evidence
                    .provision(b"bound-marker-value")
                    .point_to(&marker);
                let anchor = self
                    .base
                    .root
                    .provision(b"marker-context")
                    .provision(&kind.identity_key())
                    .provision(&size.identity_key());
                anchor
                    .provision(b"construction")
                    .point_to(&self.base.source_construction);
                records::record(&anchor, b"source-slot", &expr.value, &expr.evidence)?;
                if self.constraints {
                    let color = field(&before, b"value")?.follow()?;
                    let region = self
                        .base
                        .root
                        .provision(b"source-region")
                        .provision(&kind.identity_key())
                        .provision(&color.identity_key());
                    let trait_head = self
                        .base
                        .root
                        .provision(b"source-color")
                        .provision(&color.identity_key());
                    let scope = field(&census, b"head-scope")?.follow()?;
                    let prefix = field(&scope, b"record-prefix")?.follow()?;
                    let applicable = prefix
                        .provision(&kind.identity_key())
                        .provision(&size.identity_key());
                    for (head, target) in [
                        (&region, &expr.value),
                        (&trait_head, &literal),
                        (&applicable, &expr.value),
                    ] {
                        head.provision(b"construction")
                            .point_to(&self.base.source_construction);
                        records::record(head, b"source-slot", target, &expr.evidence)?;
                    }
                }
            }
        }
        self.guard(0)?;
        Ok(())
    }
    pub fn begin(&mut self, id: &[u8], input: &Grid) -> Result<ColorRun, ColorError> {
        if self.active.is_some() {
            return Err(ColorError::NotComplete);
        }
        self.last = MarkerStats::default();
        self.guard(input.width() * input.height() * 32 * 512)?;
        let mut run = self.base.begin(id, input)?;
        self.active = Some(run.root.clone());
        self.queried = true;
        let census = self.census(input)?;
        run.root.provision(b"marker-census").point_to(&census);
        run.root
            .provision(b"bound-marker-value")
            .point_to(&self.marker_source.follow()?);
        for cell in &run.cells {
            let context =
                field(&field(&census, b"by-source")?, &cell.input.identity_key())?.follow()?;
            let kind = field(&context, b"kind")?.follow()?;
            let size = field(&context, b"size-status")?.follow()?;
            cell.node.provision(b"marker-kind").point_to(&kind);
            cell.node.provision(b"marker-size").point_to(&size);
            cell.node.provision(b"source-context").point_to(&context);
            if self.constraints {
                let color = field(&cell.input, b"value")?.follow()?;
                let scope = field(&census, b"head-scope")?.follow()?;
                let prefix = field(&scope, b"query-prefix")?.follow()?;
                let marker = prefix
                    .child(&kind.identity_key())
                    .and_then(|r| r.child(&size.identity_key()));
                cell.node.provision(b"applicability-scope").point_to(&scope);
                let region = self
                    .base
                    .root
                    .child(b"source-region")
                    .and_then(|r| r.child(&kind.identity_key()))
                    .and_then(|r| r.child(&color.identity_key()));
                let broad = self
                    .base
                    .root
                    .child(b"source-color")
                    .and_then(|r| r.child(&color.identity_key()));
                for (key, head) in [
                    (b"head-marker".as_slice(), marker),
                    (b"head-region".as_slice(), region),
                    (b"head-color".as_slice(), broad),
                ] {
                    let at = cell.node.provision(key);
                    if let Some(h) = head {
                        at.point_to(&h);
                    }
                }
            }
        }
        run.relation_copies = self.last.region_copies + self.last.role_copies;
        run.relation_setup_ns =
            self.last.region_setup_ns + self.last.role_setup_ns + self.last.guard_ns;
        run.relation_execute_ns = self.last.region_execute_ns + self.last.role_execute_ns;
        run.relation_cache_hits = self.last.cache_hits;
        run.relation_proofs = self.last.proofs;
        Ok(run)
    }
    pub fn execute(&mut self, run: &mut ColorRun, limit: usize) -> Result<(), ColorError> {
        if !self.active.as_ref().is_some_and(|a| a.same(&run.root)) {
            return Err(ColorError::NotComplete);
        }
        let root = self.base.root.clone();
        let constraints = self.constraints;
        let result = self.base.execute_sources_with(run, limit, |cell| {
            if constraints {
                let read = |key: &[u8]| -> Result<Option<Address>, ColorError> {
                    match field(&cell.node, key)?.follow() {
                        Ok(at) => Ok(Some(at)),
                        Err(crate::Fault::Unset) => Ok(None),
                        Err(e) => Err(e.into()),
                    }
                };
                let marker = read(b"head-marker")?;
                let region = read(b"head-region")?;
                let broad = read(b"head-color")?;
                let structured =
                    records::intersect(&[marker.as_ref(), region.as_ref()], b"source-slot")?;
                let selected = match structured {
                    Selection::Unknown => match broad.as_ref() {
                        Some(h) => records::resolve(h, b"source-slot")?,
                        None => Selection::Unknown,
                    },
                    Selection::Ambiguous { alternatives: 0 } => structured,
                    Selection::Ambiguous { .. } => records::intersect(
                        &[marker.as_ref(), region.as_ref(), broad.as_ref()],
                        b"source-slot",
                    )?,
                    Selection::Unique { .. } => structured,
                };
                if let Selection::Unique { candidate, .. } = &selected {
                    cell.lookup
                        .point_to(&field(candidate, b"anchor")?.follow()?);
                } else if let Some(h) = marker.as_ref().or(region.as_ref()).or(broad.as_ref()) {
                    cell.lookup.point_to(h);
                }
                return Ok(selected.into());
            }
            let kind = field(&cell.node, b"marker-kind")?.follow()?;
            let size = field(&cell.node, b"marker-size")?.follow()?;
            match root
                .child(b"marker-context")
                .and_then(|r| r.child(&kind.identity_key()))
                .and_then(|r| r.child(&size.identity_key()))
            {
                Some(anchor) => {
                    cell.lookup.point_to(&anchor);
                    Ok(records::resolve(&anchor, b"source-slot")?.into())
                }
                None => Ok(Choice::Unknown),
            }
        });
        if result.is_ok() {
            self.active = None;
        }
        result
    }
    pub fn record(&self, grids: &mut Grids, run: ColorRun) -> Result<Grid, ColorError> {
        self.base.record(grids, run)
    }
}
