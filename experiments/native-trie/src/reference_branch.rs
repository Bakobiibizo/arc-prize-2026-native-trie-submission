//! A stored six-copy conditional over supplied native reference identity.
//! No host comparison of the arguments, token decoder, pairwise answer table or new opcode.
use crate::{Address, Fault, Machine, install, next_field, source_field, target_field};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchError {
    NotReady,
    Limit,
    TraceLimit,
    Incomplete,
    Machine(Fault),
}
impl From<Fault> for BranchError {
    fn from(value: Fault) -> Self {
        Self::Machine(value)
    }
}
struct Step {
    before: Address,
    source: Address,
    target: Address,
    next: Address,
    copied: Address,
    after: Address,
    source_fault: Address,
    target_fault: Address,
    next_fault: Address,
    copied_fault: Address,
}
impl Step {
    fn new(root: &Address) -> Self {
        Self {
            before: root.provision(b"before"),
            source: root.provision(b"source"),
            target: root.provision(b"target"),
            next: root.provision(b"next-cell"),
            copied: root.provision(b"copied-reference"),
            after: root.provision(b"after"),
            source_fault: root.provision(b"source-read-fault"),
            target_fault: root.provision(b"target-read-fault"),
            next_fault: root.provision(b"next-read-fault"),
            copied_fault: root.provision(b"copied-read-fault"),
        }
    }
}
struct Bound {
    frame: Address,
    inputs: [Address; 2],
    steps: [Step; 6],
}
pub struct BranchProjection {
    pub value: Address,
    pub evidence: Address,
    pub cycles: u64,
    _owner: Address,
    _output_owner: Address,
    _inputs: [Address; 2],
}
pub struct ReferenceBranch {
    pub machine: Machine,
    pub construction: Address,
    arguments: [Address; 2],
    result: Address,
    halt: Address,
    entry: Address,
    tags: [Address; 2],
    not_produced: Address,
    faults: [Address; 4],
    bound: Option<Bound>,
    completed: bool,
    failed: bool,
    serial: usize,
}
fn fault_index(f: Fault) -> usize {
    match f {
        Fault::MissingField => 0,
        Fault::Unset => 1,
        Fault::Dangling => 2,
        Fault::TerminalUsedAsReference => 3,
    }
}
fn observe(
    slot: &Address,
    error: &Address,
    value: Result<Address, Fault>,
    faults: &[Address; 4],
) -> Option<Address> {
    match value {
        Ok(value) => {
            slot.point_to(&value);
            Some(value)
        }
        Err(f) => {
            error.point_to(&faults[fault_index(f)]);
            None
        }
    }
}
impl Default for ReferenceBranch {
    fn default() -> Self {
        Self::new()
    }
}
impl ReferenceBranch {
    pub fn new() -> Self {
        let machine = Machine::default();
        let root = machine.states.root();
        let construction = root.provision(b"construction");
        let arguments = [
            root.provision(b"argument-x-view"),
            root.provision(b"argument-y-view"),
        ];
        let result = machine.output.root().provision(b"branch-tag");
        let halt = root.provision(b"halt");
        halt.terminate();
        let ret = root.provision(b"return");
        ret.point_to(&halt);
        let tags = [
            root.provision(b"result-tags").provision(b"same"),
            root.provision(b"result-tags").provision(b"different"),
        ];
        let not_produced = root.provision(b"not-produced");
        let scratch = root.provision(b"initial-descriptor-placeholder");
        let set_y = construction.provision(b"bind-y-target");
        let set_x = construction.provision(b"bind-x-target");
        let set_next = construction.provision(b"bind-y-next");
        let init = construction.provision(b"initialize-y");
        let conditional = construction.provision(b"write-x-follow-y");
        let same = construction.provision(b"same");
        let different = construction.provision(b"different");
        let to_same = root.provision(b"same-continuation");
        to_same.point_to(&same);
        let to_different = root.provision(b"different-continuation");
        to_different.point_to(&different);
        let emit_same = root.provision(b"same-result");
        emit_same.point_to(&tags[0]);
        let emit_different = root.provision(b"different-result");
        emit_different.point_to(&tags[1]);
        let link = |id: &[u8], state: &Address| {
            let at = root.provision(b"continuations").provision(id);
            at.point_to(state);
            at
        };
        install(&same, &emit_same, &result, &ret);
        install(&different, &emit_different, &result, &ret);
        install(&conditional, &to_same, &scratch, &scratch);
        install(
            &init,
            &to_different,
            &scratch,
            &link(b"conditional", &conditional),
        );
        install(
            &set_y,
            &arguments[1],
            &target_field(&init).unwrap(),
            &link(b"set-x", &set_x),
        );
        install(
            &set_x,
            &arguments[0],
            &target_field(&conditional).unwrap(),
            &link(b"set-next", &set_next),
        );
        install(
            &set_next,
            &arguments[1],
            &next_field(&conditional).unwrap(),
            &link(b"initialize", &init),
        );
        let fault_root = root.provision(b"read-faults");
        let faults = [
            fault_root.provision(b"missing-field"),
            fault_root.provision(b"unset"),
            fault_root.provision(b"dangling"),
            fault_root.provision(b"terminal"),
        ];
        Self {
            machine,
            construction,
            arguments,
            result,
            halt,
            entry: set_y,
            tags,
            not_produced,
            faults,
            bound: None,
            completed: false,
            failed: false,
            serial: 0,
        }
    }
    /// Bind emitted references before activation. These may be source-slot addresses;
    /// the stored conditional chooses them, not a host truth-tag interpreter.
    pub fn set_outputs(&mut self, same: &Address, different: &Address) -> Result<(), BranchError> {
        if self.bound.is_some() || self.completed {
            return Err(BranchError::NotReady);
        }
        self.machine
            .states
            .root()
            .child(b"same-result")
            .unwrap()
            .point_to(same);
        self.machine
            .states
            .root()
            .child(b"different-result")
            .unwrap()
            .point_to(different);
        Ok(())
    }
    pub fn same_tag(&self) -> Address {
        self.tags[0].clone()
    }
    pub fn different_tag(&self) -> Address {
        self.tags[1].clone()
    }
    /// Mechanical private-view interning and input ownership only. It never decides a branch.
    pub fn bind(&mut self, left: &Address, right: &Address) -> Address {
        self.bound = None;
        self.completed = false;
        self.failed = false;
        let frame = self
            .machine
            .states
            .root()
            .provision(b"activations")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        frame.provision(b"input-x").point_to(left);
        frame.provision(b"input-y").point_to(right);
        for key in [b"same-result".as_slice(), b"different-result".as_slice()] {
            let source = self.machine.states.root().child(key).unwrap();
            frame
                .provision(b"output-sources")
                .provision(key)
                .point_to(&source);
            observe(
                &frame.provision(key),
                &frame.provision(b"output-source-faults").provision(key),
                source.follow(),
                &self.faults,
            );
        }
        frame
            .provision(b"construction")
            .point_to(&self.construction);
        let views = frame.provision(b"private-views");
        for (argument, input) in self.arguments.iter().zip([left, right]) {
            let view = views.provision(&input.identity_key());
            view.provision(b"input").point_to(input);
            argument.point_to(&view);
        }
        let steps = std::array::from_fn(|i| {
            Step::new(&frame.provision(b"steps").provision(&i.to_be_bytes()))
        });
        self.result.point_to(&self.not_produced);
        self.machine.enter(&self.entry);
        self.bound = Some(Bound {
            frame: frame.clone(),
            inputs: [left.clone(), right.clone()],
            steps,
        });
        frame
    }
    /// Native execution plus fixed-slot observation recording; no semantic argument equality.
    pub fn execute(&mut self, limit: usize) -> Result<u64, BranchError> {
        if self.bound.is_none() || self.completed || self.failed {
            return Err(BranchError::NotReady);
        }
        for _ in 0..limit {
            let index = self.machine.cycles as usize;
            if index >= 6 {
                return Err(BranchError::TraceLimit);
            }
            let step = &self.bound.as_ref().unwrap().steps[index];
            let state = self.machine.current()?;
            step.before.point_to(&state);
            let read =
                |field: Option<Address>| field.ok_or(Fault::MissingField).and_then(|a| a.follow());
            let source = observe(
                &step.source,
                &step.source_fault,
                read(source_field(&state)),
                &self.faults,
            );
            observe(
                &step.target,
                &step.target_fault,
                read(target_field(&state)),
                &self.faults,
            );
            observe(
                &step.next,
                &step.next_fault,
                read(next_field(&state)),
                &self.faults,
            );
            if let Some(source) = source {
                observe(
                    &step.copied,
                    &step.copied_fault,
                    source.follow(),
                    &self.faults,
                );
            }
            if let Err(e) = self.machine.cycle() {
                self.failed = true;
                return Err(e.into());
            }
            let after = self.machine.current()?;
            step.after.point_to(&after);
            if after.same(&self.halt) {
                self.completed = true;
                return Ok(self.machine.cycles);
            }
        }
        Err(BranchError::Limit)
    }
    /// Opt-in recording-only retirement of unused diagnostic reservation scaffolding.
    /// Actual fault tags (including observed Unset), nonempty slots, every native fact,
    /// and all incomplete/faulted activations remain untouched. Native code is unchanged.
    pub fn record_compact(&mut self) -> Result<BranchProjection, BranchError> {
        let projection = self.record()?;
        if let Some(steps) = projection.evidence.child(b"steps") {
            for step in steps.inspect_children() {
                step.0.children.borrow_mut().retain(|key, slot| {
                    let diagnostic = matches!(
                        key.as_ref(),
                        b"source-read-fault"
                            | b"target-read-fault"
                            | b"next-read-fault"
                            | b"copied-read-fault"
                    );
                    !(diagnostic
                        && matches!(slot.follow(), Err(Fault::Unset))
                        && slot.0.children.borrow().is_empty())
                });
            }
        }
        projection
            .evidence
            .provision(b"empty-fault-reservations-retired")
            .terminate();
        Ok(projection)
    }
    /// Publication snapshots the actual emitted reference, never computes its truth value.
    pub fn record(&mut self) -> Result<BranchProjection, BranchError> {
        if !self.completed || self.failed {
            return Err(BranchError::NotReady);
        }
        let value = self.result.follow()?;
        if value.same(&self.not_produced) {
            return Err(BranchError::Incomplete);
        }
        let bound = self.bound.take().ok_or(BranchError::NotReady)?;
        bound.frame.provision(b"emitted").point_to(&value);
        bound
            .frame
            .provision(b"copies")
            .provision(&self.machine.cycles.to_be_bytes());
        self.completed = false;
        Ok(BranchProjection {
            value,
            evidence: bound.frame,
            cycles: self.machine.cycles,
            _owner: self.machine.states.root(),
            _output_owner: self.machine.output.root(),
            _inputs: bound.inputs,
        })
    }
}
