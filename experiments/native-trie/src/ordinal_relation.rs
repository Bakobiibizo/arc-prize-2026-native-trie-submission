//! docs/native/NATIVE-ORDINAL-BANK-PROTOTYPE.md. Native strict successor reachability;
//! physical indexes bind entries, never decide ordinal ordering.
use crate::{
    Address, Fault, Machine, arc_grids::GridError, complete_count_witness::CompleteCountProjection,
    install, reference_set_count::SetCountProjection, region_witness::RegionError,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
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
/// A committed native producer, its unavailable binding, and its strong owners.
pub struct OrdinalInput {
    pub proof: Address,
    pub unavailable: Address,
    _complete: Option<CompleteCountProjection>,
    _set: Option<SetCountProjection>,
    _relative: Option<Box<crate::relative_ordinal_count::RelativeOrdinalProjection>>,
}
impl OrdinalInput {
    pub fn from_complete(p: CompleteCountProjection, unavailable: Address) -> Rc<Self> {
        Rc::new(Self {
            proof: p.proof.clone(),
            unavailable,
            _complete: Some(p),
            _set: None,
            _relative: None,
        })
    }
    pub fn from_set(p: SetCountProjection, unavailable: Address) -> Rc<Self> {
        Rc::new(Self {
            proof: p.proof.clone(),
            unavailable,
            _complete: None,
            _set: Some(p),
            _relative: None,
        })
    }
    pub fn from_relative(
        p: crate::relative_ordinal_count::RelativeOrdinalProjection,
        unavailable: Address,
    ) -> Rc<Self> {
        Rc::new(Self {
            proof: p.count().proof.clone(),
            unavailable,
            _complete: None,
            _set: None,
            _relative: Some(Box::new(p)),
        })
    }
    pub fn current(&self) -> Result<Address, RegionError> {
        if !self
            .proof
            .child(b"committed")
            .is_some_and(|n| n.is_terminal())
        {
            return Err(RegionError::Incomplete);
        }
        Ok(field(&self.proof, b"value")?.follow()?)
    }
}
pub struct OrdinalBank {
    pub root: Address,
    owner: Address,
    target: Address,
    _target_input: Rc<OrdinalInput>,
    entries: BTreeMap<Vec<u8>, Address>,
    unavailable_entry: Address,
    return_slot: Address,
    result: Address,
    greater: Address,
    not_greater: Address,
    unavailable: Address,
    poisoned: bool,
}
pub struct OrdinalWitness<'a> {
    pub machine: Machine,
    pub root: Address,
    pub budget: usize,
    bank: &'a mut OrdinalBank,
    halt: Address,
    completed: bool,
    failed: bool,
    a: Rc<OrdinalInput>,
    b: Rc<OrdinalInput>,
}
pub struct OrdinalProjection {
    pub proof: Address,
    pub value: Address,
    pub copies: u64,
    _a: Rc<OrdinalInput>,
    _b: Rc<OrdinalInput>,
    _bank: Address,
    _owner: Address,
}
impl OrdinalBank {
    pub fn reserve_bytes() -> usize {
        (48 * 901 + 512) * 640
    }
    pub fn invocation_reserve_bytes() -> usize {
        512 * 640
    }
    pub fn prepare(
        zero: &Address,
        target: Rc<OrdinalInput>,
        parent: &Address,
        id: &[u8],
    ) -> Result<Self, RegionError> {
        if !parent
            .child(b"cardinalities")
            .and_then(|n| n.child(b"zero"))
            .is_some_and(|n| n.same(zero))
        {
            return Err(GridError::ForeignSpace.into());
        }
        // Validate/enumerate the supplied finite representation, without inspecting
        // either operand's order or using payload/depth as a comparison result.
        let mut nodes = Vec::new();
        let mut seen = BTreeSet::new();
        let mut next = Some(zero.clone());
        for _ in 0..901 {
            let Some(n) = next.take() else { break };
            if !seen.insert(n.identity_key()) {
                return Err(RegionError::Incomplete);
            }
            next = match n.follow() {
                Ok(n) => Some(n),
                Err(Fault::Unset) => None,
                Err(e) => return Err(e.into()),
            };
            nodes.push(n);
        }
        if next.is_some() {
            return Err(RegionError::Limit);
        }
        if nodes.len() != 901 {
            return Err(GridError::Malformed.into());
        }
        let value = target.current()?;
        let banks = parent.provision(b"ordinal-relation-banks");
        if banks.child(id).is_some() {
            return Err(RegionError::Duplicate);
        }
        let root = banks.provision(id);
        root.provision(b"zero-at-bind").point_to(zero);
        root.provision(b"target-at-bind").point_to(&value);
        root.provision(b"target-producer").point_to(&target.proof);
        root.provision(b"constructor").point_to(
            &parent
                .provision(b"ordinal-constructors")
                .provision(b"strict-successor-private-view-native-return-v1"),
        );
        let tags = parent.provision(b"ordinal-relation-tags");
        let greater = tags.provision(b"greater");
        let not_greater = tags.provision(b"not-greater");
        let unavailable = tags.provision(b"unavailable");
        let result = root.provision(b"activation-result");
        let return_slot = root.provision(b"activation-return");
        let code = root.provision(b"construction");
        let yes = code.provision(b"greater-return");
        let no = code.provision(b"not-greater-return");
        let missing = code.provision(b"unavailable-return");
        // Deliberately no constant wrapper: actual native return data is followed.
        install(
            &yes,
            &constant(&yes, b"tag", &greater),
            &result,
            &return_slot,
        );
        install(
            &no,
            &constant(&no, b"tag", &not_greater),
            &result,
            &return_slot,
        );
        install(
            &missing,
            &constant(&missing, b"tag", &unavailable),
            &result,
            &return_slot,
        );
        let aliases = root.provision(b"activation-private-views");
        let b = aliases.provision(&value.identity_key());
        let mut states = BTreeMap::new();
        let mut continuation = no.clone();
        for ordinal in nodes.iter().rev() {
            let at = code
                .provision(b"ordinals")
                .provision(&ordinal.identity_key());
            at.provision(b"ordinal-at-bind").point_to(ordinal);
            let x = aliases.provision(&ordinal.identity_key());
            let entry = branch(&at, &x, &b, &yes, &continuation);
            states.insert(ordinal.identity_key().to_vec(), entry.clone());
            continuation = entry;
        }
        let supported = states.contains_key(value.identity_key().as_slice());
        let mut entries = BTreeMap::new();
        for (i, n) in nodes.iter().enumerate() {
            let start = if !supported {
                missing.clone()
            } else if let Some(successor) = nodes.get(i + 1) {
                states[&successor.identity_key().to_vec()].clone()
            } else {
                no.clone()
            };
            root.provision(b"strict-start-index")
                .provision(&n.identity_key())
                .point_to(&start);
            entries.insert(n.identity_key().to_vec(), start);
        }
        Ok(Self {
            root,
            owner: parent.clone(),
            target: value,
            _target_input: target,
            entries,
            unavailable_entry: missing,
            return_slot,
            result,
            greater,
            not_greater,
            unavailable,
            poisoned: false,
        })
    }
    pub fn greater_tag(&self) -> Address {
        self.greater.clone()
    }
    pub fn not_greater_tag(&self) -> Address {
        self.not_greater.clone()
    }
    pub fn unavailable_tag(&self) -> Address {
        self.unavailable.clone()
    }
    pub fn compare(
        &mut self,
        a: Rc<OrdinalInput>,
        b: Rc<OrdinalInput>,
        id: &[u8],
    ) -> Result<OrdinalWitness<'_>, RegionError> {
        if self.poisoned {
            return Err(RegionError::Incomplete);
        }
        let av = a.current()?;
        let bv = b.current()?;
        if !bv.same(&self.target) {
            return Err(RegionError::Incomplete);
        } // Cache-key authority, not order.
        let proofs = self.root.provision(b"proofs");
        if proofs.child(id).is_some() {
            return Err(RegionError::Duplicate);
        }
        let root = proofs.provision(id);
        let args = root.provision(b"arguments");
        for (k, input, value) in [(b"a".as_slice(), &a, &av), (b"b", &b, &bv)] {
            let arg = args.provision(k);
            arg.provision(b"producer").point_to(&input.proof);
            arg.provision(b"source-slot")
                .point_to(&field(&input.proof, b"value")?);
            arg.provision(b"value-at-bind").point_to(value);
            arg.provision(b"unavailable-at-bind")
                .point_to(&input.unavailable);
        }
        root.provision(b"bank").point_to(&self.root);
        let execution = root.provision(b"execution");
        let machine = Machine::default();
        execution
            .provision(b"machine")
            .publish_prepared_child(&machine.pointer());
        let halt = execution.provision(b"halt");
        halt.terminate();
        let code = execution.provision(b"construction");
        let output = root.provision(b"value");
        let emit = code.provision(b"copy-native-result");
        copy(&emit, &self.result, &output, &halt);
        let missing = code.provision(b"unavailable");
        copy(
            &missing,
            &constant(&missing, b"tag", &self.unavailable),
            &output,
            &halt,
        );
        let aliases = execution.provision(b"private-views");
        let alias = |v: &Address| aliases.provision(&v.identity_key());
        let mut start = self
            .entries
            .get(av.identity_key().as_slice())
            .cloned()
            .unwrap_or_else(|| self.unavailable_entry.clone());
        for (k, input, value) in [(b"b".as_slice(), &b, &bv), (b"a", &a, &av)] {
            start = branch(
                &code.provision(k),
                &alias(value),
                &alias(&input.unavailable),
                &missing,
                &start,
            );
        }
        let call = code.provision(b"bind-native-return");
        copy(
            &call,
            &constant(&call, b"return-address", &emit),
            &self.return_slot,
            &start,
        );
        machine.pointer().point_to(&call);
        Ok(OrdinalWitness {
            machine,
            root,
            budget: 2 * 901 + 8,
            bank: self,
            halt,
            completed: false,
            failed: false,
            a,
            b,
        })
    }
}
impl OrdinalWitness<'_> {
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
                Ok(n) => n,
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
    pub fn record(self) -> Result<OrdinalProjection, RegionError> {
        self.checkpoint();
        if !self.completed {
            return Err(RegionError::Incomplete);
        }
        let value = field(&self.root, b"value")?.follow()?;
        self.root.provision(b"committed").terminate();
        self.root
            .provision(b"copies")
            .provision(&self.machine.cycles.to_be_bytes());
        Ok(OrdinalProjection {
            proof: self.root.clone(),
            value,
            copies: self.machine.cycles,
            _a: self.a.clone(),
            _b: self.b.clone(),
            _bank: self.bank.root.clone(),
            _owner: self.bank.owner.clone(),
        })
    }
}
impl Drop for OrdinalWitness<'_> {
    fn drop(&mut self) {
        if !self.completed {
            self.bank.poisoned = true;
            self.bank
                .root
                .provision(b"poisoned-by")
                .point_to(&self.root);
        }
    }
}
