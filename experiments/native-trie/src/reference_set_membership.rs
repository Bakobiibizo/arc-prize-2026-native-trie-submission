//! Guarded native reference-set membership. Physical indexing builds the set;
//! native copy/private-view conditionals alone decide presence.
use crate::{
    Address, Machine,
    arc_grids::{Grid, GridError},
    install,
    region_witness::RegionError,
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
pub struct SetMembershipWitness {
    pub root: Address,
    pub machine: Machine,
    pub budget: usize,
    halt: Address,
    completed: bool,
    failed: bool,
    owner: Address,
    set: Address,
    expected: Address,
    expected_slot: Address,
    head: Vec<u8>,
    members: Vec<(Address, Address)>,
    guards: Vec<(Address, Address, Address)>,
    inputs: Vec<Grid>,
}
pub struct SetMembershipProjection {
    pub proof: Address,
    pub value: Address,
    pub copies: u64,
    _owner: Address,
    _set: Address,
    _expected: Address,
    _inputs: Vec<Grid>,
    _bindings: Vec<Address>,
    owners: Vec<Address>,
}
impl SetMembershipProjection {
    pub(crate) fn retain_owners(mut self, owners: Vec<Address>) -> Self {
        self.owners.extend(owners);
        self
    }
}
impl SetMembershipWitness {
    pub fn reserve_bytes(set: &Address, head: &[u8], guard_count: usize) -> usize {
        (32 * (entries(set, head).len() + guard_count) + 128) * 640
    }
    pub fn prepare(
        set: Address,
        head: &[u8],
        expected_slot: &Address,
        guards: &[(Address, Address)],
        parent: &Address,
        id: &[u8],
        inputs: Vec<Grid>,
    ) -> Result<Self, RegionError> {
        let members = entries(&set, head);
        if members.len() > 900 {
            return Err(RegionError::Limit);
        }
        let expected = expected_slot.follow()?;
        let member_bindings = members
            .iter()
            .map(|m| Ok((m.clone(), m.follow()?)))
            .collect::<Result<Vec<_>, RegionError>>()?;
        let guard_bindings = guards
            .iter()
            .map(|(s, u)| Ok((s.clone(), s.follow()?, u.clone())))
            .collect::<Result<Vec<_>, RegionError>>()?;
        let bank = parent.provision(b"set-membership-proofs");
        if bank.child(id).is_some() {
            return Err(RegionError::Duplicate);
        }
        let root = bank.provision(id);
        root.provision(b"source-set").point_to(&set);
        root.provision(b"expected-source-slot")
            .point_to(expected_slot);
        root.provision(b"expected-reference-at-bind")
            .point_to(&expected);
        root.provision(b"constructor").point_to(
            &parent
                .provision(b"set-membership-constructors")
                .provision(b"guarded-native-reference-conditionals-v1"),
        );
        let execution = root.provision(b"execution");
        let code = execution.provision(b"construction");
        let machine = Machine::default();
        execution
            .provision(b"machine")
            .publish_prepared_child(&machine.pointer());
        let halt = execution.provision(b"halt");
        halt.terminate();
        let value = root.provision(b"value");
        let tags = parent.provision(b"set-membership-tags");
        let yes = code.provision(b"present");
        let no = code.provision(b"absent");
        let missing = code.provision(b"unavailable");
        copy(
            &yes,
            &constant(&yes, b"tag", &tags.provision(b"present")),
            &value,
            &halt,
        );
        copy(
            &no,
            &constant(&no, b"tag", &tags.provision(b"absent")),
            &value,
            &halt,
        );
        copy(
            &missing,
            &constant(&missing, b"tag", &tags.provision(b"unavailable")),
            &value,
            &halt,
        );
        let views = execution.provision(b"private-views");
        let alias = |v: &Address| views.provision(&v.identity_key());
        let mut next = no;
        for (i, member) in members.iter().enumerate().rev() {
            let reference = member.follow()?;
            let at = code.provision(b"members").provision(&i.to_be_bytes());
            at.provision(b"set-candidate").point_to(member);
            at.provision(b"reference-at-bind").point_to(&reference);
            next = branch(&at, &alias(&reference), &alias(&expected), &yes, &next);
        }
        for (i, (slot, unavailable)) in guards.iter().enumerate().rev() {
            let actual = slot.follow()?;
            let arg = root.provision(b"arguments").provision(&i.to_be_bytes());
            arg.provision(b"source-slot").point_to(slot);
            arg.provision(b"value-at-bind").point_to(&actual);
            arg.provision(b"unavailable-reference")
                .point_to(unavailable);
            next = branch(
                &code.provision(b"guards").provision(&i.to_be_bytes()),
                &alias(&actual),
                &alias(unavailable),
                &missing,
                &next,
            );
        }
        machine.pointer().point_to(&next);
        Ok(Self {
            root,
            machine,
            budget: 2 * (members.len() + guards.len()) + 1,
            halt,
            completed: false,
            failed: false,
            owner: parent.clone(),
            set,
            expected,
            expected_slot: expected_slot.clone(),
            head: head.to_vec(),
            members: member_bindings,
            guards: guard_bindings,
            inputs,
        })
    }
    fn validate_bindings(&self) -> Result<(), RegionError> {
        if !field(&self.root, b"source-set")?.follow()?.same(&self.set)
            || !field(&self.root, b"expected-source-slot")?
                .follow()?
                .same(&self.expected_slot)
            || !self.expected_slot.follow()?.same(&self.expected)
            || !field(&self.root, b"expected-reference-at-bind")?
                .follow()?
                .same(&self.expected)
        {
            return Err(RegionError::Incomplete);
        }
        let current = entries(&self.set, &self.head);
        if current.len() != self.members.len() {
            return Err(RegionError::Incomplete);
        }
        for (now, (slot, value)) in current.iter().zip(&self.members) {
            if !now.same(slot) || !slot.follow()?.same(value) {
                return Err(RegionError::Incomplete);
            }
        }
        for (i, (slot, value, unavailable)) in self.guards.iter().enumerate() {
            let arg = field(&field(&self.root, b"arguments")?, &i.to_be_bytes())?;
            if !slot.follow()?.same(value)
                || !field(&arg, b"source-slot")?.follow()?.same(slot)
                || !field(&arg, b"value-at-bind")?.follow()?.same(value)
                || !field(&arg, b"unavailable-reference")?
                    .follow()?
                    .same(unavailable)
            {
                return Err(RegionError::Incomplete);
            }
        }
        Ok(())
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
        if let Err(e) = self.validate_bindings() {
            self.failed = true;
            self.checkpoint();
            return Err(e);
        }
        for _ in 0..limit {
            if let Err(e) = self.machine.cycle() {
                self.failed = true;
                self.checkpoint();
                return Err(e.into());
            }
            let at = match self.machine.current() {
                Ok(n) => n,
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
    pub fn record(self) -> Result<SetMembershipProjection, RegionError> {
        self.checkpoint();
        if !self.completed {
            return Err(RegionError::Incomplete);
        }
        self.validate_bindings()?;
        let mut bindings = vec![self.expected_slot.clone()];
        for (s, v) in &self.members {
            bindings.extend([s.clone(), v.clone()]);
        }
        for (s, v, u) in &self.guards {
            bindings.extend([s.clone(), v.clone(), u.clone()]);
        }
        let value = field(&self.root, b"value")?.follow()?;
        self.root.provision(b"committed").terminate();
        self.root
            .provision(b"copies")
            .provision(&self.machine.cycles.to_be_bytes());
        Ok(SetMembershipProjection {
            proof: self.root,
            value,
            copies: self.machine.cycles,
            _owner: self.owner,
            _set: self.set,
            _expected: self.expected,
            _inputs: self.inputs,
            _bindings: bindings,
            owners: Vec::new(),
        })
    }
}
