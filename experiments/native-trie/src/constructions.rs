//! Seed assembly and supplied application trees. None of this module is called by cycle().
//! Shared trie programs perform binding, nested calls, and output linking using the copy cycle.
use crate::{Address, Fault, Machine, Outcome, install, next_field, source_field, target_field};

#[derive(Clone)]
enum Operand {
    Cell(Address),
    Through(Address),
}
#[derive(Clone)]
enum Continuation {
    Cell(Address),
    Through(Address),
}
use Operand::{Cell, Through};

struct Seeds {
    instructions: Address,
    constants: Address,
    instruction_count: usize,
    constant_count: usize,
}
impl Seeds {
    fn value(&mut self, target: &Address) -> Address {
        let cell = self.constants.provision(&self.constant_count.to_be_bytes());
        self.constant_count += 1;
        cell.point_to(target);
        cell
    }
    fn fresh(&mut self) -> Address {
        let state = self
            .instructions
            .provision(&self.instruction_count.to_be_bytes());
        self.instruction_count += 1;
        state
    }
    fn copy(&mut self, read: Operand, write: Operand, continuation: Continuation) -> Address {
        let action = self.fresh();
        let source = match &read {
            Cell(cell) => cell.clone(),
            Through(_) => action.clone(),
        };
        let target = match &write {
            Cell(cell) => cell.clone(),
            Through(_) => action.clone(),
        };
        let next = match &continuation {
            Continuation::Cell(cell) => cell.clone(),
            Continuation::Through(_) => action.clone(),
        };
        install(&action, &source, &target, &next);
        let mut entry = action.clone();
        // Indirection is represented by executable writes to the action's field cells.
        // These enum descriptions exist only while assembling the initial seed.
        for (operand, field) in [
            (read, source_field(&action).unwrap()),
            (write, target_field(&action).unwrap()),
        ] {
            if let Through(pointer) = operand {
                let patch = self.fresh();
                let after_patch = self.value(&entry);
                install(&patch, &pointer, &field, &after_patch);
                entry = patch;
            }
        }
        if let Continuation::Through(pointer) = continuation {
            let patch = self.fresh();
            let after_patch = self.value(&entry);
            install(
                &patch,
                &pointer,
                &next_field(&action).unwrap(),
                &after_patch,
            );
            entry = patch;
        }
        entry
    }
    fn chain(&mut self, moves: Vec<(Operand, Operand)>, last: Continuation) -> Address {
        let mut follow = last;
        let mut entry = None;
        for (read, write) in moves.into_iter().rev() {
            let action = self.copy(read, write, follow);
            follow = Continuation::Cell(self.value(&action));
            entry = Some(action);
        }
        entry.expect("seed chain must contain a copy")
    }
}

struct Ports {
    context: Address,
    parent: Address,
    first: Address,
    end: Address,
    parent_slot: Address,
    phase_slot: Address,
    left_slot: Address,
    right_slot: Address,
    bound_first_slot: Address,
    bound_end_slot: Address,
    link_target: Address,
}
impl Ports {
    fn new(root: &Address) -> Self {
        Self {
            context: root.provision(b"context"),
            parent: root.provision(b"parent"),
            first: root.provision(b"result-first"),
            end: root.provision(b"result-end"),
            parent_slot: root.provision(b"parent-slot"),
            phase_slot: root.provision(b"phase-slot"),
            left_slot: root.provision(b"left-slot"),
            right_slot: root.provision(b"right-slot"),
            bound_first_slot: root.provision(b"bound-first-slot"),
            bound_end_slot: root.provision(b"bound-end-slot"),
            link_target: root.provision(b"link-target"),
        }
    }
}

/// One owned application occurrence. No Clone: its mutable frame and endpoints are single-use.
pub struct Application {
    pub entry: Address,
    pub context: Address,
    pub first: Address,
    pub end: Address,
}

pub struct Builder {
    machine: Machine,
    seed: Seeds,
    ports: Ports,
    frames: Address,
    tokens: Address,
    output: Address,
    frame_count: usize,
    output_count: usize,
    call_entry: Address,
    halt: Address,
    terminal_source: Address,
}
impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
impl Builder {
    pub fn new() -> Self {
        let machine = Machine::default();
        let root = machine.states.root();
        let ports = Ports::new(&root.provision(b"ports"));
        let halt = root.provision(b"halt");
        halt.terminate();
        let terminal_source = root.provision(b"terminal-source");
        terminal_source.terminate();
        ports.parent.point_to(&halt);
        let mut seed = Seeds {
            instructions: root.provision(b"seed-code"),
            constants: root.provision(b"seed-values"),
            instruction_count: 0,
            constant_count: 0,
        };
        // The second result has arrived. Bind the writable endpoint, connect it, and return.
        let completed = seed.chain(
            vec![
                (
                    Through(ports.bound_end_slot.clone()),
                    Cell(ports.link_target.clone()),
                ),
                (
                    Cell(ports.first.clone()),
                    Through(ports.link_target.clone()),
                ),
                (
                    Through(ports.bound_first_slot.clone()),
                    Cell(ports.first.clone()),
                ),
                (
                    Through(ports.parent_slot.clone()),
                    Cell(ports.parent.clone()),
                ),
            ],
            Continuation::Cell(ports.parent.clone()),
        );
        let completed_value = seed.value(&completed);
        let left_returned = seed.chain(
            vec![
                (
                    Cell(ports.first.clone()),
                    Through(ports.bound_first_slot.clone()),
                ),
                (
                    Cell(ports.end.clone()),
                    Through(ports.bound_end_slot.clone()),
                ),
                (Cell(completed_value), Through(ports.phase_slot.clone())),
                (Cell(ports.context.clone()), Cell(ports.parent.clone())),
            ],
            Continuation::Through(ports.right_slot.clone()),
        );
        let left_returned_value = seed.value(&left_returned);
        let call_entry = seed.chain(
            vec![
                (
                    Cell(ports.parent.clone()),
                    Through(ports.parent_slot.clone()),
                ),
                (Cell(left_returned_value), Through(ports.phase_slot.clone())),
                (Cell(ports.context.clone()), Cell(ports.parent.clone())),
            ],
            Continuation::Through(ports.left_slot.clone()),
        );
        let frames = root.provision(b"contexts");
        let tokens = root.provision(b"literal-tokens");
        let output = machine.output.root();
        Self {
            machine,
            seed,
            ports,
            frames,
            tokens,
            output,
            frame_count: 0,
            output_count: 0,
            call_entry,
            halt,
            terminal_source,
        }
    }
    fn frame(&mut self) -> Address {
        let frame = self.frames.provision(&self.frame_count.to_be_bytes());
        self.frame_count += 1;
        frame
    }
    fn location(&mut self) -> Address {
        let node = self.output.provision(&self.output_count.to_be_bytes());
        self.output_count += 1;
        node
    }
    fn expose(&mut self, context: &Address, phase: &Address) -> Address {
        let phase_cell = context.provision(b"phase");
        phase_cell.point_to(phase);
        let mut moves = Vec::new();
        for (field, register) in [
            (b"parent".as_slice(), &self.ports.parent_slot),
            (b"phase", &self.ports.phase_slot),
            (b"left", &self.ports.left_slot),
            (b"right", &self.ports.right_slot),
            (b"bound-first", &self.ports.bound_first_slot),
            (b"bound-end", &self.ports.bound_end_slot),
        ] {
            let value = self.seed.value(&context.provision(field));
            moves.push((Cell(value), Cell(register.clone())));
        }
        let own_entry_value = self.seed.value(&self.halt); // patched as seed data before execution
        moves.push((
            Cell(own_entry_value.clone()),
            Cell(self.ports.context.clone()),
        ));
        let entry = self.seed.chain(moves, Continuation::Cell(phase_cell));
        own_entry_value.point_to(&entry);
        entry
    }
    pub fn literal(&mut self, bytes: &[u8]) -> Application {
        let context = self.frame();
        let nodes: Vec<_> = (0..bytes.len()).map(|_| self.location()).collect();
        let end = self.location();
        let first = nodes.first().unwrap_or(&end).clone();
        let mut moves = Vec::new();
        for (index, byte) in bytes.iter().enumerate() {
            let token = self.tokens.provision(&[*byte]);
            let token_value = self.seed.value(&token);
            let successor = self.seed.value(nodes.get(index + 1).unwrap_or(&end));
            moves.push((Cell(successor), Cell(nodes[index].clone())));
            moves.push((Cell(token_value), Cell(nodes[index].provision(b"token"))));
        }
        moves.push((Cell(self.terminal_source.clone()), Cell(end.clone())));
        let first_value = self.seed.value(&first);
        let end_value = self.seed.value(&end);
        moves.push((Cell(first_value), Cell(self.ports.first.clone())));
        moves.push((Cell(end_value), Cell(self.ports.end.clone())));
        let literal_entry = self
            .seed
            .chain(moves, Continuation::Cell(self.ports.parent.clone()));
        let entry = self.expose(&context, &literal_entry);
        Application {
            entry,
            context,
            first,
            end,
        }
    }
    /// A supplied binary application. Loading binds only operand entry addresses, not their values.
    pub fn join(&mut self, left: Application, right: Application) -> Application {
        let context = self.frame();
        context.provision(b"left").point_to(&left.entry);
        context.provision(b"right").point_to(&right.entry);
        let phase = self.call_entry.clone();
        let entry = self.expose(&context, &phase);
        Application {
            entry,
            context,
            first: left.first,
            end: right.end,
        }
    }
    /// Discard the host assembler. Only the machine plus readout handles survive execution.
    pub fn finish(mut self, root: Application) -> Evaluation {
        self.ports.parent.point_to(&self.halt);
        self.machine.enter(&root.entry);
        Evaluation {
            machine: self.machine,
            first: self.ports.first,
            end: self.ports.end,
            halt: self.halt,
            root_context: root.context,
        }
    }
}

pub struct Evaluation {
    pub machine: Machine,
    first: Address,
    end: Address,
    halt: Address,
    pub root_context: Address,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvaluationError {
    Machine(Fault),
    Limit,
    UnexpectedTerminal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionError {
    Reference(Fault),
    MissingToken,
    Limit,
}

impl Evaluation {
    pub fn execute(&mut self, limit: usize) -> Result<u64, EvaluationError> {
        for _ in 0..limit {
            if self.machine.cycle().map_err(EvaluationError::Machine)? == Outcome::Halted {
                let stopped = self.machine.current().map_err(EvaluationError::Machine)?;
                return if stopped.same(&self.halt) {
                    Ok(self.machine.cycles)
                } else {
                    Err(EvaluationError::UnexpectedTerminal)
                };
            }
        }
        Err(EvaluationError::Limit)
    }
    pub fn first(&self) -> Result<Address, Fault> {
        self.first.follow()
    }
    pub fn end(&self) -> Result<Address, Fault> {
        self.end.follow()
    }
    pub fn halt(&self) -> Address {
        self.halt.clone()
    }
    /// Observation only: walks computed output links, never joins or evaluates operands.
    pub fn inspect(&self, limit: usize) -> Result<Vec<u8>, InspectionError> {
        self.inspect_view(
            &self.first().map_err(InspectionError::Reference)?,
            None,
            limit,
        )
    }
    pub fn inspect_view(
        &self,
        first: &Address,
        end: Option<&Address>,
        limit: usize,
    ) -> Result<Vec<u8>, InspectionError> {
        let mut node = first.clone();
        let mut result = Vec::new();
        for _ in 0..limit {
            if end.is_some_and(|boundary| node.same(boundary)) || node.is_terminal() {
                return Ok(result);
            }
            if let Some(cell) = node.child(b"token") {
                let token = cell.follow().map_err(InspectionError::Reference)?;
                result.extend_from_slice(token.token().ok_or(InspectionError::MissingToken)?);
            }
            node = node.follow().map_err(InspectionError::Reference)?;
        }
        Err(InspectionError::Limit)
    }
}
