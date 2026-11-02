//! Sparse positive relations / unchanged negative evidence. Supplied vocabulary: four actual
//! adjacency edges, conjunction, neighbour-copy. No host comparison of grid values.
use crate::{
    Address, Machine, Trie,
    arc_grids::{Grid, Grids},
    install,
    positional::Error,
    records::{self, Selection},
    reference_branch::ReferenceBranch,
    source_field,
};
#[path = "native_axis_profile.rs"]
mod native_axis_profile;
pub use native_axis_profile::{
    NativeAxisActivationProof, NativeAxisBridgeProof, NativeAxisLineProof,
};
#[path = "sparse_component_relations.rs"]
mod component_relations;
#[path = "sparse_input_relations.rs"]
mod input_relations;
#[path = "sparse_source_groups.rs"]
mod source_groups;
pub use source_groups::{GroupLesson, GroupProgram};
#[path = "sparse_action_partitions.rs"]
mod action_partitions;
#[path = "source_binding_generalization.rs"]
mod binding_generalization;
#[path = "component_contexts.rs"]
mod component_contexts;
#[path = "context_action_learning.rs"]
mod context_action_learning;
#[path = "context_composition.rs"]
mod context_composition;
#[path = "path_value_copy.rs"]
mod path_value_copy;
#[path = "rectangle_copy.rs"]
mod rectangle_copy;
#[path = "rectangle_source.rs"]
mod rectangle_source;
pub use rectangle_copy::RectangleCopyWitness;
#[path = "cyclic_lattice.rs"]
mod cyclic_lattice;
pub use cyclic_lattice::CyclicProgram;
#[path = "broad_structural_induction.rs"]
mod broad_structural_induction;
#[path = "cyclic_learning.rs"]
mod cyclic_learning;
pub use cyclic_learning::{
    GeneralizedCandidate, GeneralizedCandidateOrigin, GeneralizedCandidateSet,
};
#[path = "ordered_object_learning.rs"]
mod ordered_object_learning;
pub use rectangle_source::RectangleProjection;
#[path = "path_value_fields.rs"]
mod path_value_fields;
#[path = "relative_actions.rs"]
mod relative_actions;
#[path = "source_pair_actions.rs"]
mod source_pair_actions;
const PORTS: [&[u8]; 4] = [b"north", b"west", b"south", b"east"];
fn field(a: &Address, k: &[u8]) -> Result<Address, Error> {
    a.child(k).ok_or(Error::Incomplete)
}
fn transduction_error(error: crate::episode_transduction::TransductionError) -> Error {
    match error {
        crate::episode_transduction::TransductionError::Shape => Error::Shape,
        crate::episode_transduction::TransductionError::Ambiguous => Error::Ambiguous,
        crate::episode_transduction::TransductionError::Limit => Error::CycleLimit,
        crate::episode_transduction::TransductionError::ValidationMismatch => {
            Error::ValidationMismatch
        }
        crate::episode_transduction::TransductionError::Persistent(fault) => {
            Error::Persistent(fault)
        }
        _ => Error::Incomplete,
    }
}
#[derive(Clone)]
pub struct Rule {
    pub root: Address,
    owner: Address,
    ports: Vec<&'static [u8]>,
    input: Option<input_relations::InputSelection>,
    component: bool,
}
impl Rule {
    pub fn clauses(&self) -> usize {
        if self.component {
            return 1;
        }
        self.input
            .as_ref()
            .map_or(self.ports.len(), |p| p.body.inspect_children().len())
    }
}
pub struct Derived {
    pub evidence: Address,
    pub rule: Option<Rule>,
    pub changed: usize,
    pub positive_tests: usize,
    pub negative_checks: usize,
    pub negative_unchecked: usize,
}
pub struct SparseRewrite {
    pub root: Address,
    pub machine: Machine,
    branch: ReferenceBranch,
    yes: Address,
    no: Address,
    outside: Address,
    unknown: Address,
    space: Address,
    serial: usize,
    frozen: bool,
    pub comparison_copies: u64,
    pub emission_copies: u64,
    pub region_copies: u64,
    pub region_proofs: u64,
    pub region_cache_hits: u64,
    pub count_copies: u64,
    pub count_proofs: u64,
    pub set_copies: u64,
    pub set_proofs: u64,
    relative_factory: Option<crate::relative_ordinal_count::RelativeOrdinalCount>,
    relative_observations: u64,
    source_pair_observations: u64,
    path_observations: u64,
    rectangle_observations: u64,
    ordered_inputs: std::collections::BTreeMap<Vec<u8>, ordered_object_learning::OrderedInput>,
    rectangle_boundaries: std::collections::BTreeMap<
        Vec<u8>,
        crate::reference_set_membership::SetMembershipProjection,
    >,
    path_upper: std::cell::Cell<usize>,
    path_values: std::collections::BTreeMap<
        Vec<u8>,
        crate::reference_set_membership::SetMembershipProjection,
    >,
    pub membership_copies: u64,
    pub membership_proofs: u64,
    relative_inputs:
        std::collections::BTreeMap<Vec<u8>, std::rc::Rc<crate::ordinal_relation::OrdinalInput>>,
    relative_values:
        std::collections::BTreeMap<Vec<u8>, std::rc::Rc<crate::ordinal_relation::OrdinalInput>>,
    intrinsic_actions: crate::intrinsic_actions::IntrinsicActions,
    episode_transducer: crate::episode_transduction::LocalEpisodeTransducer,
    persistent_arc: Option<crate::disk_arc::DiskArc>,
}
impl SparseRewrite {
    pub fn new(s: &Grids) -> Self {
        let root = Trie::default().root();
        let branch = ReferenceBranch::new();
        let machine = Machine::default();
        root.provision(b"comparisons")
            .point_to(&branch.machine.states.root());
        root.provision(b"emitter").point_to(&machine.states.root());
        Self {
            yes: root.provision(b"tags").provision(b"true"),
            no: root.provision(b"tags").provision(b"false"),
            outside: s.outside(),
            unknown: s.unresolved(),
            space: s.storage.root(),
            root,
            machine,
            branch,
            serial: 0,
            frozen: false,
            comparison_copies: 0,
            emission_copies: 0,
            region_copies: 0,
            region_proofs: 0,
            region_cache_hits: 0,
            count_copies: 0,
            count_proofs: 0,
            set_copies: 0,
            set_proofs: 0,
            relative_factory: None,
            relative_observations: 0,
            source_pair_observations: 0,
            path_observations: 0,
            rectangle_observations: 0,
            ordered_inputs: std::collections::BTreeMap::new(),
            rectangle_boundaries: std::collections::BTreeMap::new(),
            path_upper: std::cell::Cell::new(0),
            path_values: std::collections::BTreeMap::new(),
            membership_copies: 0,
            membership_proofs: 0,
            relative_inputs: std::collections::BTreeMap::new(),
            relative_values: std::collections::BTreeMap::new(),
            intrinsic_actions: crate::intrinsic_actions::IntrinsicActions::default(),
            episode_transducer: crate::episode_transduction::LocalEpisodeTransducer::default(),
            persistent_arc: None,
        }
    }
    pub fn persistent_arc_store_root(&self) -> Option<crate::disk_trie::DiskAddress> {
        self.persistent_arc
            .as_ref()
            .map(crate::disk_arc::DiskArc::store_root)
    }

    fn persistent_arc(&mut self) -> Result<&crate::disk_arc::DiskArc, Error> {
        if self.persistent_arc.is_none() {
            static NEXT_STORE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let trace_root = std::env::var_os("NATIVE_TRIE_TRACE_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("native-trie-traces"));
            let store = trace_root.join(format!(
                "arc-execution-{}-{:016x}",
                std::process::id(),
                NEXT_STORE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            self.persistent_arc =
                Some(crate::disk_arc::DiskArc::create(store).map_err(|_| Error::Incomplete)?);
        }
        self.persistent_arc.as_ref().ok_or(Error::Incomplete)
    }

    // Same prior ARC estimated envelope, not RSS. Charge ALL five owned model/machine
    // trees, never follow weak links into separately charged supplied source grids.
    fn owned_roots(&self) -> [Address; 5] {
        [
            self.root.clone(),
            self.branch.machine.states.root(),
            self.branch.machine.output.root(),
            self.machine.states.root(),
            self.machine.output.root(),
        ]
    }
    pub fn workspace_bytes(&self) -> usize {
        self.owned_roots()
            .iter()
            .map(crate::region_witness::RegionWitness::storage_charge)
            .sum()
    }
    fn guard(&self, reserve: usize) -> Result<(), Error> {
        if self.workspace_bytes().saturating_add(reserve) > 286_720_000 {
            Err(Error::WorkspaceLimit)
        } else {
            Ok(())
        }
    }
    fn compare(
        &mut self,
        a: &Address,
        b: &Address,
        same: &Address,
        different: &Address,
    ) -> Result<(Address, Address), Error> {
        // Check at most128 comparisons apart. Reserve160 native/recording nodes per
        // comparison plus24 metadata nodes per maximum900-cell grid; each charged640
        // (512+128 token allowance). This bounds unchecked growth without an O(N) scan
        // on every pulse. No native kernel execution allocates these reservations.
        if self.comparison_copies.is_multiple_of(6 * 128) {
            self.guard((128 * 160 + 900 * 24) * 640)?;
        }
        self.branch
            .set_outputs(same, different)
            .map_err(|_| Error::Incomplete)?;
        self.branch.bind(a, b);
        self.branch.execute(6).map_err(|_| Error::Incomplete)?;
        let p = self
            .branch
            .record_compact()
            .map_err(|_| Error::Incomplete)?;
        self.comparison_copies += p.cycles;
        Ok((p.value, p.evidence))
    }
    fn tag(&self, anchor: &Address) -> Result<Address, Error> {
        Ok(match records::resolve(anchor, b"predicate")? {
            Selection::Unique { target, .. } => target,
            _ => self.unknown.clone(),
        })
    }
    fn neighbour(&self, cell: &Address, port: &[u8]) -> Result<Address, Error> {
        let n = field(cell, port)?.follow()?;
        Ok(n.child(b"value").unwrap_or_else(|| {
            let slot = self.root.provision(b"outside-value");
            slot.point_to(&self.outside);
            slot
        }))
    }
    fn predicate(
        &mut self,
        cell: &Address,
        ports: &[&[u8]],
        at: &Address,
    ) -> Result<Address, Error> {
        let source = self.neighbour(cell, ports[0])?;
        let value = source.follow()?;
        let centre = field(cell, b"value")?.follow()?;
        let yes = self.yes.clone();
        let no = self.no.clone();
        for (other, invert) in [
            (self.outside.clone(), true),
            (self.unknown.clone(), true),
            (centre, true),
        ]
        .into_iter()
        .chain(
            ports
                .iter()
                .map(|p| {
                    self.neighbour(cell, p)
                        .and_then(|v| v.follow().map_err(Error::from))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|v| (v, false)),
        ) {
            let (tag, proof) = if invert {
                self.compare(&value, &other, &no, &yes)?
            } else {
                self.compare(&value, &other, &yes, &no)?
            };
            records::record(at, b"predicate", &tag, &proof)?;
        }
        self.tag(at)
    }
    pub fn agree(ds: &[Derived]) -> Option<Rule> {
        if ds.len() < 2 {
            return None;
        }
        let r = ds.first()?.rule.as_ref()?;
        ds.iter()
            .all(|d| d.rule.as_ref().is_some_and(|x| x.root.same(&r.root)))
            .then(|| r.clone())
    }
    pub fn derive(&mut self, id: &[u8], input: &Grid, output: &Grid) -> Result<Derived, Error> {
        if self.frozen {
            return Err(Error::NotReady);
        }
        if !input.owner.same(&self.space) || !output.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        if input.width() != output.width() || input.height() != output.height() {
            return Err(Error::Shape);
        }
        self.guard((128 * 160 + 900 * 24) * 640 + id.len())?;
        let all = self.root.provision(b"derivations");
        if all.child(id).is_some() {
            return Err(Error::NotReady);
        }
        let evidence = all.provision(id);
        evidence.provision(b"input").point_to(&input.anchor);
        evidence.provision(b"output").point_to(&output.anchor);
        let changed = evidence.provision(b"changed");
        let unchanged = evidence.provision(b"unchanged");
        for y in 0..input.height() {
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let after = output.cell(y, x)?;
                let frame = evidence
                    .provision(b"pairs")
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes());
                frame.provision(b"before").point_to(&before);
                frame.provision(b"after").point_to(&after);
                let a = field(&before, b"value")?.follow()?;
                let b = field(&after, b"value")?.follow()?;
                let (destination, proof) = self.compare(&a, &b, &unchanged, &changed)?;
                frame.provision(b"change-proof").point_to(&proof);
                records::record(&destination, b"source-colour", &a, &frame)?;
            }
        }
        let mut result = Derived {
            evidence: evidence.clone(),
            rule: None,
            changed: 0,
            positive_tests: 0,
            negative_checks: 0,
            negative_unchecked: 0,
        };
        let (candidate, colour) = match records::resolve(&changed, b"source-colour")? {
            Selection::Unique { candidate, target } => (candidate, target),
            _ => return Ok(result),
        };
        if colour.same(&self.unknown) {
            return Ok(result);
        }
        let frames = field(&candidate, b"evidence")?.inspect_children();
        result.changed = frames.len();
        let relations = evidence.provision(b"positive-relations");
        let yes = self.yes.clone();
        let no = self.no.clone();
        for entry in frames {
            let f = entry.follow()?;
            let before = field(&f, b"before")?.follow()?;
            let after = field(&f, b"after")?.follow()?;
            let want = field(&after, b"value")?.follow()?;
            for port in PORTS {
                let value = self.neighbour(&before, port)?.follow()?;
                let (tag, proof) = self.compare(&value, &want, &yes, &no)?;
                proof.provision(b"changed-pair").point_to(&f);
                records::record(&relations.provision(port), b"predicate", &tag, &proof)?;
                result.positive_tests += 1;
            }
        }
        let included = evidence.provision(b"included");
        let excluded = evidence.provision(b"excluded");
        for port in PORTS {
            let value = self.tag(&relations.provision(port))?;
            let (destination, proof) = self.compare(&value, &yes, &included, &excluded)?;
            let role = self.root.provision(b"ports").provision(port);
            records::record(&destination, b"relation", &role, &proof)?;
        }
        let index = included
            .child(b"holds")
            .and_then(|v| v.child(b"relation"))
            .and_then(|v| v.child(b"candidates"));
        let ports: Vec<_> = PORTS
            .into_iter()
            .filter(|p| {
                let role = self.root.provision(b"ports").provision(p);
                index
                    .as_ref()
                    .is_some_and(|i| i.child(&role.identity_key()).is_some())
            })
            .collect();
        if ports.is_empty() {
            return Ok(result);
        }
        let rejected = evidence.provision(b"counterexamples");
        let accepted = evidence.provision(b"accepted-negatives");
        if unchanged.child(b"holds").is_some()
            && let Selection::Unique { candidate, .. } =
                records::intersect(&[Some(&unchanged), Some(&changed)], b"source-colour")?
        {
            for entry in field(&candidate, b"evidence")?.inspect_children() {
                let f = entry.follow()?;
                let before = field(&f, b"before")?.follow()?;
                let at = f.provision(b"negative-predicate");
                let tag = self.predicate(&before, &ports, &at)?;
                let (destination, proof) = self.compare(&tag, &yes, &rejected, &accepted)?;
                records::record(&destination, b"cell", &before, &proof)?;
                result.negative_checks += 1;
            }
        }
        if !matches!(records::resolve(&rejected, b"cell")?, Selection::Unknown) {
            return Ok(result);
        }
        let mut signature = self
            .root
            .provision(b"encodings")
            .provision(&colour.identity_key());
        for port in &ports {
            signature = signature.provision(port);
        }
        signature.provision(b"source-colour").point_to(&colour);
        for port in &ports {
            signature
                .provision(b"relations")
                .provision(port)
                .point_to(&self.root.provision(b"ports").provision(port));
        }
        signature
            .provision(b"copy-source-port")
            .point_to(&self.root.provision(b"ports").provision(ports[0]));
        signature
            .provision(b"comparison-construction")
            .point_to(&self.branch.construction);
        records::record(&signature, b"support", &signature, &evidence)?;
        evidence.provision(b"encoding").point_to(&signature);
        result.rule = Some(Rule {
            root: signature,
            owner: self.root.clone(),
            ports,
            input: None,
            component: false,
        });
        Ok(result)
    }
    fn emit(&mut self, source: &Address, target: &Address) -> Result<(), Error> {
        let r = self.machine.states.root();
        let emit = r.provision(b"copy");
        let patch = r.provision(b"bind-source");
        let choice = r.provision(b"selected-source-slot");
        choice.point_to(source);
        let halt = r.provision(b"halt");
        halt.terminate();
        let end = r.provision(b"return");
        end.point_to(&halt);
        let next = r.provision(b"to-copy");
        next.point_to(&emit);
        install(&emit, &choice, target, &end);
        install(&patch, &choice, &source_field(&emit).unwrap(), &next);
        self.machine.enter(&patch);
        self.machine.cycle()?;
        self.machine.cycle()?;
        self.emission_copies += self.machine.cycles;
        Ok(())
    }
    pub fn apply(&mut self, s: &mut Grids, input: &Grid, rule: &Rule) -> Result<Grid, Error> {
        if !rule.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        self.frozen = true;
        let root = self
            .root
            .provision(b"queries")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        root.provision(b"input").point_to(&input.anchor);
        root.provision(b"rule").point_to(&rule.root);
        let mut rows = Vec::new();
        let scope = root.provision(b"scope");
        let colour = field(&rule.root, b"source-colour")?.follow()?;
        records::record(&scope, b"rule", &colour, &rule.root)?;
        let yes = self.yes.clone();
        for y in 0..input.height() {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let value = field(&before, b"value")?.follow()?;
                let cell = row.provision(b"cells").provision(&x.to_be_bytes());
                cell.provision(b"before").point_to(&before);
                let source = field(&before, b"value")?;
                let scoped = scope
                    .child(b"holds")
                    .and_then(|a| a.child(b"rule"))
                    .and_then(|a| a.child(b"candidates"))
                    .and_then(|a| a.child(&value.identity_key()));
                let selected = if scoped.is_some() {
                    let (tag, neighbour) = if rule.component {
                        (
                            self.predicate_component(
                                input,
                                &before,
                                &rule.root,
                                &cell.provision(b"predicate"),
                            )?,
                            field(&rule.root, b"literal-action-slot")?.follow()?,
                        )
                    } else if let Some(ref program) = rule.input {
                        (
                            self.predicate_input(&before, program, &cell.provision(b"predicate"))?,
                            field(&rule.root, b"literal-action-slot")?.follow()?,
                        )
                    } else {
                        (
                            self.predicate(&before, &rule.ports, &cell.provision(b"predicate"))?,
                            self.neighbour(&before, rule.ports[0])?,
                        )
                    };
                    let (slot, proof) = self.compare(&tag, &yes, &neighbour, &source)?;
                    cell.provision(b"selection-proof").point_to(&proof);
                    slot
                } else {
                    source
                };
                self.emit(&selected, &cell.provision(b"value"))?;
                cell.provision(b"emitted-source-slot").point_to(&selected);
                cell.provision(b"source-value-after-copy")
                    .point_to(&selected.follow()?);
                cell.provision(b"written-value")
                    .point_to(&field(&cell, b"value")?.follow()?);
            }
            rows.push(row);
        }
        self.guard(0)?;
        Ok(s.derived(input, &rule.root, &rows)?)
    }
}
#[cfg(test)]
mod budget {
    use super::*;
    #[test]
    fn unchecked_comparison_reserve_and_fixed_limit() {
        let s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let yes = m.yes.clone();
        let no = m.no.clone();
        let before = m.workspace_bytes();
        for _ in 0..128 {
            let (value, proof) = m.compare(&yes, &no, &yes, &no).unwrap();
            records::record(
                &m.root.provision(b"test-records"),
                b"predicate",
                &value,
                &proof,
            )
            .unwrap();
        }
        assert!(m.workspace_bytes() - before < 128 * 160 * 640);
        assert!(matches!(m.guard(286_720_000), Err(Error::WorkspaceLimit)));
        assert_eq!(m.comparison_copies, 128 * 6);
    }
}
