//! docs/native/NATIVE-COMPLETE-CARDINALITY-PROTOCOL.md: observed unary count AND native
//! no-unknown-frontier certificate. No host classification/count interpretation.
#[cfg(test)]
#[path = "complete_count_retention_tests.rs"]
mod retention_tests;
use crate::{
    Address, Machine,
    arc_grids::GridError,
    install,
    region_witness::{RegionError, RegionProjection},
};
fn field(a: &Address, k: &[u8]) -> Result<Address, RegionError> {
    a.child(k).ok_or(GridError::Malformed.into())
}
fn constant(a: &Address, k: &[u8], v: &Address) -> Address {
    let c = a.provision(k);
    c.point_to(v);
    c
}
fn copy(a: &Address, s: &Address, t: &Address, next: &Address) {
    install(a, s, t, &constant(a, b"continuation", next));
}
fn branch(a: &Address, x: &Address, y: &Address, yes: &Address, no: &Address) -> Address {
    let reset = a.provision(b"reset-y");
    let check = a.provision(b"write-x-follow-y");
    copy(&reset, &constant(a, b"different", no), y, &check);
    install(&check, &constant(a, b"same", yes), x, y);
    reset
}
pub struct CompleteCountWitness {
    pub machine: Machine,
    pub root: Address,
    pub construction: Address,
    pub budget: usize,
    halt: Address,
    completed: bool,
    failed: bool,
    region: RegionProjection,
    owner: Address,
}
pub struct CompleteCountProjection {
    pub proof: Address,
    pub value: Address,
    pub status: Address,
    pub copies: u64,
    _region: RegionProjection,
    _owner: Address,
}
impl CompleteCountWitness {
    /// Checked retention of an existing native certificate; no truth reconstructed.
    pub(crate) fn projection_from_committed(
        parent: &Address,
        native_owner: &Address,
        input: &crate::arc_grids::Grid,
        proof: &Address,
    ) -> Result<CompleteCountProjection, RegionError> {
        if !proof.child(b"committed").is_some_and(|n| n.is_terminal())
            || !field(parent, b"complete-count-proofs")?
                .inspect_children()
                .iter()
                .any(|n| n.same(proof))
        {
            return Err(RegionError::Incomplete);
        }
        let region_proof = field(proof, b"region-proof")?.follow()?;
        let region = crate::region_witness::RegionWitness::projection_from_committed(
            native_owner,
            input,
            &region_proof,
        )?;
        let copy_records = field(proof, b"copies")?.inspect_children();
        let bytes = copy_records
            .first()
            .and_then(|n| n.token())
            .ok_or(RegionError::Incomplete)?;
        let copies = u64::from_be_bytes(bytes.try_into().map_err(|_| RegionError::Incomplete)?);
        Ok(CompleteCountProjection {
            proof: proof.clone(),
            value: field(proof, b"value")?.follow()?,
            status: field(proof, b"status")?.follow()?,
            copies,
            _region: region,
            _owner: parent.clone(),
        })
    }
    /// Resource enumeration only. The caller guards ALL model roots with this
    /// reserve before preparation; this is not an RSS/whole-model guard itself.
    pub fn reserve_bytes(region: &RegionProjection) -> Result<usize, RegionError> {
        Ok((32 * field(&region.proof, b"views")?.inspect_children().len() + 128) * 640)
    }
    pub fn prepare(
        region: RegionProjection,
        parent: &Address,
        id: &[u8],
    ) -> Result<Self, RegionError> {
        if !region
            .proof
            .child(b"committed")
            .is_some_and(|n| n.is_terminal())
        {
            return Err(RegionError::Incomplete);
        }
        let owner = region.owner_root();
        let count = field(&region.proof, b"observed-cardinality")?;
        let kind = field(&region.proof, b"result")?;
        let kind_value = kind.follow()?;
        let unresolved_kind = field(&field(&owner, b"relation-tags")?, b"unresolved")?;
        let unknown_class = field(&field(&owner, b"class-tags")?, b"unknown")?;
        let proofs = parent.provision(b"complete-count-proofs");
        if proofs.child(id).is_some() {
            return Err(RegionError::Duplicate);
        }
        let root = proofs.provision(id);
        root.provision(b"region-proof").point_to(&region.proof);
        root.provision(b"producer").point_to(
            &parent
                .provision(b"complete-count-constructors")
                .provision(b"known-kind-all-frontier-known-copy-count-v1"),
        );
        let args = root.provision(b"arguments");
        args.provision(b"count-source-slot").point_to(&count);
        args.provision(b"count-reference-at-bind")
            .point_to(&count.follow()?);
        args.provision(b"kind-source-slot").point_to(&kind);
        args.provision(b"kind-reference-at-bind")
            .point_to(&kind_value);
        args.provision(b"unknown-class").point_to(&unknown_class);
        let machine = Machine::default();
        let execution = root.provision(b"execution");
        execution
            .provision(b"machine")
            .publish_prepared_child(&machine.pointer());
        let construction = execution.provision(b"construction");
        let aliases = execution.provision(b"private-reference-views");
        let alias = |v: &Address| aliases.provision(&v.identity_key());
        let halt = execution.provision(b"halt");
        halt.terminate();
        let value = root.provision(b"value");
        let status = root.provision(b"status");
        let tags = parent.provision(b"complete-count-tags");
        let complete = construction.provision(b"mark-complete");
        let emit_count = construction.provision(b"copy-count");
        copy(
            &complete,
            &constant(&complete, b"tag", &tags.provision(b"complete")),
            &status,
            &emit_count,
        );
        copy(&emit_count, &count, &value, &halt);
        let uncertain = construction.provision(b"mark-unknown");
        let emit_unknown = construction.provision(b"copy-unavailable");
        copy(
            &uncertain,
            &constant(&uncertain, b"tag", &tags.provision(b"unknown-frontier")),
            &status,
            &emit_unknown,
        );
        copy(
            &emit_unknown,
            &constant(&emit_unknown, b"tag", &tags.provision(b"unavailable")),
            &value,
            &halt,
        );
        let facts = field(&region.proof, b"views")?.inspect_children();
        let mut next = complete;
        for (i, fact) in facts.iter().enumerate().rev() {
            let slot = field(fact, b"class")?;
            let class = slot.follow()?;
            let arg = args.provision(b"frontier").provision(&i.to_be_bytes());
            arg.provision(b"source-slot").point_to(&slot);
            arg.provision(b"bound-reference").point_to(&class);
            next = branch(
                &construction
                    .provision(b"check-frontier")
                    .provision(&i.to_be_bytes()),
                &alias(&class),
                &alias(&unknown_class),
                &uncertain,
                &next,
            );
        }
        let entry = branch(
            &construction.provision(b"check-kind"),
            &alias(&kind_value),
            &alias(&unresolved_kind),
            &uncertain,
            &next,
        );
        machine.pointer().point_to(&entry);
        Ok(Self {
            machine,
            root,
            construction,
            budget: 2 * facts.len() + 4,
            halt,
            completed: false,
            failed: false,
            region,
            owner: parent.clone(),
        })
    }
    fn checkpoint(&self) {
        self.root
            .provision(b"attempt-copies")
            .provision(&self.machine.cycles.to_be_bytes());
    }
    pub fn execute(&mut self, limit: usize) -> Result<u64, RegionError> {
        if self.completed || self.failed {
            return Err(RegionError::Incomplete);
        }
        for _ in 0..limit {
            if let Err(e) = self.machine.cycle() {
                self.failed = true;
                self.checkpoint();
                return Err(e.into());
            }
            let current = match self.machine.current() {
                Ok(at) => at,
                Err(e) => {
                    self.failed = true;
                    self.checkpoint();
                    return Err(e.into());
                }
            };
            if current.same(&self.halt) {
                self.completed = true;
                self.checkpoint();
                return Ok(self.machine.cycles);
            }
        }
        self.checkpoint();
        Err(RegionError::Limit)
    }
    pub fn record(self) -> Result<CompleteCountProjection, RegionError> {
        self.checkpoint();
        if !self.completed {
            return Err(RegionError::Incomplete);
        }
        let value = field(&self.root, b"value")?.follow()?;
        let status = field(&self.root, b"status")?.follow()?;
        self.root.provision(b"committed").terminate();
        self.root
            .provision(b"copies")
            .provision(&self.machine.cycles.to_be_bytes());
        Ok(CompleteCountProjection {
            proof: self.root,
            value,
            status,
            copies: self.machine.cycles,
            _region: self.region,
            _owner: self.owner,
        })
    }
}
