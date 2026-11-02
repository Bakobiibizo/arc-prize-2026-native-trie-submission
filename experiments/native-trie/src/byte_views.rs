//! A fixed-width, bit-anchored byte-view probe, expressed as native copy programs.
//! Input transport records have one uniform executable header; no input-specific decoder is emitted.
use crate::{Address, Fault, Machine, Outcome, Trie, install, source_field, target_field};

struct Assembly {
    owner: Address,
    values: Address,
    serial: usize,
}
impl Assembly {
    fn constant(&mut self, target: &Address) -> Address {
        let at = self.values.provision(&self.serial.to_be_bytes());
        self.serial += 1;
        at.point_to(target);
        at
    }
    fn state(&mut self) -> Address {
        let at = self.owner.provision(&self.serial.to_be_bytes());
        self.serial += 1;
        at
    }
    fn action(&mut self, source: &Address, target: &Address, next: &Address) -> Address {
        let state = self.state();
        install(&state, source, target, next);
        state
    }
    fn chain(&mut self, copies: Vec<(Address, Address)>, next_cell: Address) -> Address {
        let mut next = next_cell;
        let mut start = None;
        for (source, target) in copies.into_iter().rev() {
            let at = self.action(&source, &target, &next);
            next = self.constant(&at);
            start = Some(at);
        }
        start.expect("copy chain cannot be empty")
    }
}

pub struct Statuses {
    pub byte: Address,
    pub installed: Address,
    pub unknown: Address,
    pub eof: Address,
    pub truncated: Address,
    pub needs_reconciliation: Address,
}
struct Ports {
    input: Address,
    prefix: Address,
    zero: Address,
    one: Address,
    bit: Address,
    at_end: Address,
    slot: Address,
    choice: Address,
    candidate: Address,
    on_unknown: Address,
    on_present: Address,
}

struct Layout<'a> {
    ports: &'a Ports,
    anchor: &'a Address,
    unknown: &'a Address,
    eof: &'a Address,
    truncated: &'a Address,
    consult: &'a Address,
}

pub struct BitInput {
    pub first: Address,
    pub payloads: Vec<Address>, // transport/debug handles only; never consulted by cycle()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeError {
    Machine(Fault),
    Limit,
    UnexpectedTerminal,
    InvalidStatus,
    NotCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupError {
    MissingPath,
    MissingView,
    Unrecorded,
    Reference(Fault),
}

/// An observed result of a completed probe call. The fields are actual output trie cells.
pub struct ByteProbe {
    pub machine: Machine,
    pub representations: Trie,
    pub statuses: Statuses,
    ports: Ports,
    pub result: Address,
    pub anchor: Address,
    pub status: Address,
    halt: Address,
    no_view: Address,
    unknown_action: Address,
    zero_symbol: Address,
    one_symbol: Address,
    end_record: Address,
    read_entry: Address,
    teach_entry: Address,
    inputs: Address,
    observations: Address,
    input_serial: usize,
    observation_serial: usize,
}

impl Default for ByteProbe {
    fn default() -> Self {
        Self::new()
    }
}
impl ByteProbe {
    pub fn new() -> Self {
        let machine = Machine::default();
        let root = machine.states.root();
        let p = root.provision(b"byte-working");
        let o = machine.output.root();
        let result = o.provision(b"byte-view");
        let anchor = o.provision(b"bit-address");
        let status = o.provision(b"status");
        let halt = root.provision(b"byte-return");
        halt.terminate();
        let no_view = root.provision(b"no-view");
        let statuses = Statuses {
            byte: root.provision(b"byte-emitted"),
            installed: root.provision(b"view-installed"),
            unknown: root.provision(b"unknown-view"),
            eof: root.provision(b"input-ended"),
            truncated: root.provision(b"partial-byte"),
            needs_reconciliation: root.provision(b"admission-deferred"),
        };
        let ports = Ports {
            input: p.provision(b"input"),
            prefix: p.provision(b"prefix"),
            zero: p.provision(b"zero-branch"),
            one: p.provision(b"one-branch"),
            bit: p.provision(b"bit"),
            at_end: p.provision(b"input-end-case"),
            slot: p.provision(b"view-slot"),
            choice: p.provision(b"choice"),
            candidate: p.provision(b"lesson-view"),
            on_unknown: p.provision(b"unknown-continuation"),
            on_present: p.provision(b"present-continuation"),
        };
        let mut a = Assembly {
            owner: root.provision(b"byte-control"),
            values: root.provision(b"byte-constants"),
            serial: 0,
        };
        let halt_cell = a.constant(&halt);
        let success_value = a.constant(&statuses.byte);
        let success = a.action(&success_value, &status, &halt_cell);
        let eof_value = a.constant(&statuses.eof);
        let eof = a.action(&eof_value, &status, &halt_cell);
        let truncated_value = a.constant(&statuses.truncated);
        let truncated = a.action(&truncated_value, &status, &halt_cell);
        let deferred_value = a.constant(&statuses.needs_reconciliation);
        let deferred = a.action(&deferred_value, &status, &halt_cell);
        let unknown_value = a.constant(&statuses.unknown);
        let unknown_action = a.action(&unknown_value, &status, &ports.on_unknown);
        // Prepare the backlink BEFORE publishing a view in the bit address's forward slot.
        // An interrupted multi-copy admission may leave a pending candidate, never a broken held view.
        let installed_value = a.constant(&statuses.installed);
        let installed = a.action(&installed_value, &status, &halt_cell);
        let installed_cell = a.constant(&installed);
        let forward = a.action(&ports.candidate, &p, &installed_cell);
        let forward_cell = a.constant(&forward);
        let patch_forward = a.action(&ports.slot, &target_field(&forward).unwrap(), &forward_cell);
        let patch_forward_cell = a.constant(&patch_forward);
        let back = a.action(&anchor, &p, &patch_forward_cell);
        let back_cell = a.constant(&back);
        let teach = a.action(&ports.candidate, &target_field(&back).unwrap(), &back_cell);
        // Read/admit dispatch goes through the contents of the selected view slot.
        let consult = a.action(&p, &ports.choice, &ports.choice);
        let consult_cell = a.constant(&consult);
        let consult_patch = a.action(&ports.slot, &source_field(&consult).unwrap(), &consult_cell);
        // Advance the input cursor by copying the input record's own next link.
        let advance = a.action(&p, &ports.input, &ports.prefix);
        let advance_cell = a.constant(&advance);
        let advance_patch = a.action(
            &ports.input,
            &source_field(&advance).unwrap(),
            &advance_cell,
        );
        let advance_patch_cell = a.constant(&advance_patch);
        let zero_action = a.action(&ports.zero, &ports.prefix, &advance_patch_cell);
        let one_action = a.action(&ports.one, &ports.prefix, &advance_patch_cell);
        let end_record = a.action(&ports.at_end, &ports.choice, &ports.choice);
        end_record.point_to(&end_record);
        let representations = Trie::default();
        let bits = representations.root();
        let layout = Layout {
            ports: &ports,
            anchor: &anchor,
            unknown: &unknown_action,
            eof: &eof,
            truncated: &truncated,
            consult: &consult_patch,
        };
        Self::prefix(&bits, 0, &mut a, &layout);
        // The two actual one-bit addresses also supply input bit values. Occurrences
        // reference them instead of duplicating a value node for every supplied bit.
        let zero_symbol = bits.child(&[0]).unwrap();
        zero_symbol.point_to(&zero_action);
        let one_symbol = bits.child(&[1]).unwrap();
        one_symbol.point_to(&one_action);
        let unknown_halt = a.constant(&halt);
        let read_present = a.constant(&success);
        let teach_unknown = a.constant(&teach);
        let teach_present = a.constant(&deferred);
        let root_value = a.constant(&bits);
        let absent_value = a.constant(&no_view);
        let read_entry = a.chain(
            vec![
                (unknown_halt, ports.on_unknown.clone()),
                (read_present, ports.on_present.clone()),
                (absent_value.clone(), result.clone()),
                (absent_value.clone(), anchor.clone()),
                (root_value.clone(), ports.prefix.clone()),
            ],
            root_value.clone(),
        );
        let teach_entry = a.chain(
            vec![
                (teach_unknown, ports.on_unknown.clone()),
                (teach_present, ports.on_present.clone()),
                (absent_value.clone(), result.clone()),
                (absent_value, anchor.clone()),
                (root_value.clone(), ports.prefix.clone()),
            ],
            root_value,
        );
        let inputs = root.provision(b"supplied-bits");
        let observations = root.provision(b"shown-byte-pairs");
        Self {
            machine,
            representations,
            statuses,
            ports,
            result,
            anchor,
            status,
            halt,
            no_view,
            unknown_action,
            zero_symbol,
            one_symbol,
            end_record,
            read_entry,
            teach_entry,
            inputs,
            observations,
            input_serial: 0,
            observation_serial: 0,
        }
    }

    fn prefix(at: &Address, depth: usize, a: &mut Assembly, layout: &Layout<'_>) {
        let Layout {
            ports,
            anchor,
            unknown,
            eof,
            truncated,
            consult,
        } = layout;
        if depth == 8 {
            let slot = at.provision(b"holds").provision(b"raw-byte-msb-first");
            slot.point_to(unknown);
            let slot_value = a.constant(&slot);
            let anchor_value = a.constant(at);
            let consult_value = a.constant(consult);
            let record = a.action(&anchor_value, anchor, &consult_value);
            let record_value = a.constant(&record);
            install(at, &slot_value, &ports.slot, &record_value);
        } else {
            let zero = at.provision(&[0]);
            let one = at.provision(&[1]);
            Self::prefix(&zero, depth + 1, a, layout);
            Self::prefix(&one, depth + 1, a, layout);
            let end_case = a.constant(if depth == 0 { eof } else { truncated });
            let end_step = a.action(&end_case, &ports.at_end, &ports.input);
            let end_step_value = a.constant(&end_step);
            let one_value = a.constant(&one);
            let one_step = a.action(&one_value, &ports.one, &end_step_value);
            let one_step_value = a.constant(&one_step);
            let zero_value = a.constant(&zero);
            install(at, &zero_value, &ports.zero, &one_step_value);
        }
    }

    /// Input encoding only: one identical header per supplied bit, linked data records, no byte calculation.
    pub fn load_bits(&mut self, bits: &[bool]) -> BitInput {
        let stream = self.inputs.provision(&self.input_serial.to_be_bytes());
        self.input_serial += 1;
        let records: Vec<_> = (0..bits.len())
            .map(|i| stream.provision(&i.to_be_bytes()))
            .collect();
        let mut payloads = Vec::with_capacity(bits.len());
        for (i, bit) in bits.iter().enumerate() {
            let symbol = if *bit {
                &self.one_symbol
            } else {
                &self.zero_symbol
            };
            records[i].point_to(records.get(i + 1).unwrap_or(&self.end_record));
            install(&records[i], symbol, &self.ports.bit, &self.ports.bit);
            payloads.push(source_field(&records[i]).unwrap());
        }
        BitInput {
            first: records.first().unwrap_or(&self.end_record).clone(),
            payloads,
        }
    }
    /// External stream arrival. Subsequent decode calls advance only by trie programs.
    pub fn attach(&mut self, input: &BitInput) {
        self.ports.input.point_to(&input.first);
    }
    /// Ordinary trie addressing, not the retained per-bit copy program. Input is a complete
    /// caller-delimited address, not an unframed bitstream. Tokens are never numerically decoded.
    pub fn locate_view<'a>(
        &self,
        tokens: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Address, LookupError> {
        let anchor = self
            .representations
            .root()
            .locate(tokens)
            .ok_or(LookupError::MissingPath)?;
        self.view_at(&anchor)
    }
    /// A supplied native anchor needs no replay of its path. Resolve the reference in the trie.
    pub fn view_at(&self, anchor: &Address) -> Result<Address, LookupError> {
        let slot = anchor
            .child(b"holds")
            .and_then(|holds| holds.child(b"raw-byte-msb-first"))
            .ok_or(LookupError::MissingView)?;
        let view = slot.follow().map_err(LookupError::Reference)?;
        if view.same(&self.unknown_action) {
            Err(LookupError::Unrecorded)
        } else {
            Ok(view)
        }
    }
    /// Optional construction execution, AFTER resolution. One copy emits the stored view address.
    /// This mode writes `result`, not the retained stream reader's status/anchor receipt fields.
    pub fn start_view(&mut self, view: &Address) {
        self.ports.on_present.point_to(&self.halt);
        self.machine.enter(view);
    }
    pub fn start_read(&mut self) {
        self.machine.enter(&self.read_entry);
    }
    /// Encode a shown value/source, not an inferred result. Admission is done by the trie program.
    pub fn start_lesson(&mut self, input: &BitInput, value: u8, source: &[u8]) -> Address {
        let source_id = self.observations.provision(source);
        let observation = source_id.provision(&self.observation_serial.to_be_bytes());
        self.observation_serial += 1;
        let view = observation.provision(&[value]);
        observation
            .provision(b"input-occurrence")
            .point_to(&input.first);
        observation.provision(b"source-id").point_to(&source_id);
        view.provision(b"lesson").point_to(&observation);
        view.provision(b"construction").point_to(&self.read_entry);
        let self_value = view.provision(b"self");
        self_value.point_to(&view);
        install(&view, &self_value, &self.result, &self.ports.on_present);
        self.ports.candidate.point_to(&view);
        self.attach(input);
        self.machine.enter(&self.teach_entry);
        view
    }
    /// At most `limit` advancing copies. A final terminal observation performs no extra copy.
    pub fn execute(&mut self, limit: usize) -> Result<u64, ProbeError> {
        for _ in 0..limit {
            if self.machine.cycle().map_err(ProbeError::Machine)? == Outcome::Halted {
                let at = self.machine.current().map_err(ProbeError::Machine)?;
                return if at.same(&self.halt) {
                    Ok(self.machine.cycles)
                } else {
                    Err(ProbeError::UnexpectedTerminal)
                };
            }
        }
        let at = self.machine.current().map_err(ProbeError::Machine)?;
        if at.same(&self.halt) {
            Ok(self.machine.cycles)
        } else if at.is_terminal() {
            Err(ProbeError::UnexpectedTerminal)
        } else {
            Err(ProbeError::Limit)
        }
    }
    pub fn status_is(&self, status: &Address) -> bool {
        self.machine.current().is_ok_and(|at| at.same(&self.halt))
            && self.status.follow().is_ok_and(|found| found.same(status))
    }
    /// Observation only. No mapping arithmetic or model lookup is performed here.
    pub fn emitted_byte(&self) -> Result<Option<u8>, ProbeError> {
        if !self.machine.current().is_ok_and(|at| at.same(&self.halt)) {
            return Err(ProbeError::NotCompleted);
        }
        if !self.status_is(&self.statuses.byte) {
            return Ok(None);
        }
        let view = self.result.follow().map_err(ProbeError::Machine)?;
        let label = view.token().ok_or(ProbeError::InvalidStatus)?;
        if label.len() != 1 {
            return Err(ProbeError::InvalidStatus);
        }
        Ok(Some(label[0]))
    }
    pub fn bit_anchor(&self) -> Result<Option<Address>, ProbeError> {
        if !self.machine.current().is_ok_and(|at| at.same(&self.halt)) {
            return Err(ProbeError::NotCompleted);
        }
        let anchor = self.anchor.follow().map_err(ProbeError::Machine)?;
        Ok(if anchor.same(&self.no_view) {
            None
        } else {
            Some(anchor)
        })
    }
    /// Debug validation only, not used to choose a decoded result or execute an input.
    pub fn known_anchor(&self, bits: &[bool]) -> Option<Address> {
        let mut at = self.representations.root();
        for bit in bits {
            at = at.child(&[u8::from(*bit)])?;
        }
        Some(at)
    }
    pub fn unknown_marker(&self) -> Address {
        self.unknown_action.clone()
    }
}
