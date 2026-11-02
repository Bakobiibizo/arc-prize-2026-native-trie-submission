//! docs/native/NATIVE-COMPONENT-CONTEXT-PROTOCOL.md: actual unary count of a frozen
//! physical reference set, guarded by native unavailable-reference comparisons.
use crate::{
    Address, Machine,
    arc_grids::{Grid, GridError},
    install,
    region_witness::{RegionError, RegionWitness},
    source_field,
};
fn field(a: &Address, k: &[u8]) -> Result<Address, RegionError> {
    a.child(k).ok_or(GridError::Malformed.into())
}
fn constant(a: &Address, k: &[u8], v: &Address) -> Address {
    let n = a.provision(k);
    n.point_to(v);
    n
}
fn copy(a: &Address, s: &Address, t: &Address, next: &Address) {
    install(a, s, t, &constant(a, b"continuation", next));
}
fn branch(a: &Address, x: &Address, y: &Address, yes: &Address, no: &Address) -> Address {
    let reset = a.provision(b"reset");
    let check = a.provision(b"compare");
    copy(&reset, &constant(a, b"different", no), y, &check);
    install(&check, &constant(a, b"same", yes), x, y);
    reset
}
fn entries(set: &Address, head: &[u8]) -> Vec<Address> {
    set.child(b"holds")
        .and_then(|a| a.child(head))
        .and_then(|a| a.child(b"candidates"))
        .map_or_else(Vec::new, |a| a.inspect_children())
}
pub struct SetCountWitness {
    pub machine: Machine,
    pub root: Address,
    pub budget: usize,
    halt: Address,
    completed: bool,
    failed: bool,
    owner: Address,
    set: Address,
    inputs: Vec<Grid>,
}
pub struct SetCountProjection {
    pub proof: Address,
    pub value: Address,
    pub copies: u64,
    _owner: Address,
    _set: Address,
    _inputs: Vec<Grid>,
}
impl SetCountWitness {
    pub fn reserve_bytes(
        set: &Address,
        head: &[u8],
        guard_count: usize,
        parent: &Address,
    ) -> usize {
        let bank = if parent
            .child(b"cardinalities")
            .and_then(|b| b.child(b"ready-900"))
            .is_some_and(|n| n.is_terminal())
        {
            0
        } else {
            512000
        };
        bank + (32 * (entries(set, head).len() + guard_count) + 128) * 640
    }
    pub fn prepare(
        set: Address,
        head: &[u8],
        guards: &[(Address, Address)],
        parent: &Address,
        id: &[u8],
        inputs: Vec<Grid>,
    ) -> Result<Self, RegionError> {
        let members = entries(&set, head);
        if members.len() > 900 {
            return Err(RegionError::Limit);
        }
        let bank = parent.provision(b"set-count-proofs");
        if bank.child(id).is_some() {
            return Err(RegionError::Duplicate);
        }
        let root = bank.provision(id);
        root.provision(b"source-set").point_to(&set);
        root.provision(b"producer").point_to(
            &parent
                .provision(b"set-count-constructors")
                .provision(b"distinct-record-references-unary-guarded-v1"),
        );
        let execution = root.provision(b"execution");
        let code = execution.provision(b"construction");
        let machine = Machine::default();
        execution
            .provision(b"machine")
            .publish_prepared_child(&machine.pointer());
        let halt = execution.provision(b"halt");
        halt.terminate();
        let discard = execution.provision(b"discard");
        let value = root.provision(b"value");
        let status = root.provision(b"status");
        let tags = parent.provision(b"set-count-tags");
        let cursor = execution.provision(b"cursor");
        cursor.point_to(&RegionWitness::cardinality_zero(parent));
        let done = code.provision(b"mark-complete");
        let emit = code.provision(b"emit-count");
        copy(
            &done,
            &constant(&done, b"tag", &tags.provision(b"complete")),
            &status,
            &emit,
        );
        copy(&emit, &cursor, &value, &halt);
        let uncertain = code.provision(b"mark-unavailable");
        let missing = code.provision(b"emit-unavailable");
        copy(
            &uncertain,
            &constant(&uncertain, b"tag", &tags.provision(b"unavailable")),
            &status,
            &missing,
        );
        copy(
            &missing,
            &constant(&missing, b"tag", &tags.provision(b"unavailable")),
            &value,
            &halt,
        );
        let mut next = done;
        for (i, member) in members.iter().enumerate().rev() {
            let reference = member.follow()?;
            let at = code.provision(b"members").provision(&i.to_be_bytes());
            at.provision(b"set-candidate").point_to(member);
            at.provision(b"reference-at-bind").point_to(&reference);
            let patch = at.provision(b"patch");
            let step = at.provision(b"step");
            copy(&step, &discard, &cursor, &next);
            copy(
                &patch,
                &cursor,
                &source_field(&step).ok_or(GridError::Malformed)?,
                &step,
            );
            next = patch;
        }
        let aliases = execution.provision(b"private-reference-views");
        let alias = |v: &Address| aliases.provision(&v.identity_key());
        for (i, (slot, unavailable)) in guards.iter().enumerate().rev() {
            let actual = slot.follow()?;
            let arg = root.provision(b"arguments").provision(&i.to_be_bytes());
            arg.provision(b"source-slot").point_to(slot);
            arg.provision(b"bound-reference").point_to(&actual);
            arg.provision(b"unavailable-reference")
                .point_to(unavailable);
            next = branch(
                &code.provision(b"guards").provision(&i.to_be_bytes()),
                &alias(&actual),
                &alias(unavailable),
                &uncertain,
                &next,
            );
        }
        machine.pointer().point_to(&next);
        Ok(Self {
            machine,
            root,
            budget: 2 * (guards.len() + members.len()) + 2,
            halt,
            completed: false,
            failed: false,
            owner: parent.clone(),
            set,
            inputs,
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
            let at = match self.machine.current() {
                Ok(a) => a,
                Err(e) => {
                    self.failed = true;
                    self.checkpoint();
                    return Err(e.into());
                }
            };
            if at.same(&self.halt) {
                self.completed = true;
                self.checkpoint();
                return Ok(self.machine.cycles);
            }
        }
        self.checkpoint();
        Err(RegionError::Limit)
    }
    pub fn record(self) -> Result<SetCountProjection, RegionError> {
        self.checkpoint();
        if !self.completed {
            return Err(RegionError::Incomplete);
        }
        let value = field(&self.root, b"value")?.follow()?;
        self.root.provision(b"committed").terminate();
        self.root
            .provision(b"copies")
            .provision(&self.machine.cycles.to_be_bytes());
        Ok(SetCountProjection {
            proof: self.root,
            value,
            copies: self.machine.cycles,
            _owner: self.owner,
            _set: self.set,
            _inputs: self.inputs,
        })
    }
}
