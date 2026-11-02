//! Supplied ordered rectangle-copy prototype. Role fields bind physical
//! topology; stored source/target/next instructions perform every actual write.
use super::*;
use crate::{Fault, Machine, install, next_field, source_field, target_field};
use std::rc::Rc;
fn observe(slot: &Address, source: &Address) {
    match source.follow() {
        Ok(a) => slot.point_to(&a),
        Err(Fault::TerminalUsedAsReference) => slot.terminate(),
        Err(e) => {
            slot.provision(b"read-fault")
                .provision(format!("{e:?}").as_bytes())
                .terminate();
        }
    }
}
pub struct RectangleCopyWitness {
    pub root: Address,
    pub machine: Machine,
    pub budget: usize,
    halt: Address,
    complete: bool,
    failed: bool,
    precharged: bool,
    planned: std::collections::BTreeSet<Vec<u8>>,
    visited: std::collections::BTreeSet<Vec<u8>>,
    input: Grid,
    rows: Vec<Address>,
    program: Address,
    _rectangles: Vec<Rc<RectangleProjection>>,
    _owners: Vec<Address>,
}
impl RectangleCopyWitness {
    pub fn execute(&mut self, limit: usize) -> Result<u64, Error> {
        if self.complete || self.failed {
            return Err(Error::NotReady);
        }
        for rectangle in &self._rectangles {
            rectangle.check_bindings()?;
        }
        for _ in 0..limit {
            if !self.precharged
                && self.machine.cycles.is_multiple_of(128)
                && self
                    ._owners
                    .iter()
                    .map(crate::region_witness::RegionWitness::storage_charge)
                    .sum::<usize>()
                    .saturating_add((128 * 160 + 900 * 24) * 640)
                    > 286_720_000
            {
                return Err(Error::WorkspaceLimit);
            }
            let current = match self.machine.current() {
                Ok(v) => v,
                Err(e) => {
                    self.failed = true;
                    self.checkpoint();
                    return Err(e.into());
                }
            };
            let identity = current.identity_key().to_vec();
            if !self.planned.contains(&identity) || !self.visited.insert(identity) {
                self.failed = true;
                self.checkpoint();
                return Err(Error::Incomplete);
            }
            let step = self
                .root
                .provision(b"observed-steps")
                .provision(&self.machine.cycles.to_be_bytes());
            step.provision(b"state").point_to(&current);
            let source = source_field(&current).ok_or(Error::Incomplete)?;
            let target = target_field(&current).ok_or(Error::Incomplete)?;
            let next = next_field(&current).ok_or(Error::Incomplete)?;
            observe(&step.provision(b"actual-source-slot"), &source);
            observe(&step.provision(b"actual-target-slot"), &target);
            observe(&step.provision(b"next-source-slot-before"), &next);
            if let Ok(s) = source.follow() {
                observe(&step.provision(b"copied-value"), &s);
            }
            let actual_target = target.follow().ok();
            if let Some(t) = &actual_target {
                observe(&step.provision(b"target-value-before"), t);
            }
            if let Err(e) = self.machine.cycle() {
                self.failed = true;
                self.checkpoint();
                return Err(e.into());
            }
            if let Some(t) = &actual_target {
                observe(&step.provision(b"target-value-after"), t);
            }
            let at = match self.machine.current() {
                Ok(v) => v,
                Err(e) => {
                    self.failed = true;
                    self.checkpoint();
                    return Err(e.into());
                }
            };
            step.provision(b"actual-next-state").point_to(&at);
            if at.same(&self.halt) {
                if self.machine.cycles != self.budget as u64 || self.visited != self.planned {
                    self.failed = true;
                    self.checkpoint();
                    return Err(Error::Incomplete);
                }
                self.complete = true;
                self.checkpoint();
                return Ok(self.machine.cycles);
            }
        }
        self.checkpoint();
        Err(Error::Incomplete)
    }
    pub(super) fn commit_native(&self) -> Result<Address, Error> {
        if !self.complete || self.failed {
            return Err(Error::Incomplete);
        }
        self.root.provision(b"committed").terminate();
        Ok(self.root.clone())
    }
    fn checkpoint(&self) {
        self.root
            .provision(b"attempt-copies")
            .provision(&self.machine.cycles.to_be_bytes());
    }
    pub fn record(self, s: &mut Grids) -> Result<Grid, Error> {
        self.checkpoint();
        if !self.complete || self.failed {
            return Err(Error::Incomplete);
        }
        let output = s.derived(&self.input, &self.program, &self.rows)?;
        self.root.provision(b"committed").terminate();
        let mut owners = self._owners;
        owners.push(self.root);
        Ok(output.retain_owners(owners))
    }
}
impl SparseRewrite {
    // Shared native tape executor for explicitly addressed reference copies.
    // It retains every instruction/observation, unlike the transient emit helper.
    #[cfg(test)]
    pub(super) fn reference_copy_plan(
        &mut self,
        input: &Grid,
        root: Address,
        writes: Vec<(Address, Address)>,
    ) -> Result<RectangleCopyWitness, Error> {
        self.reference_copy_plan_inner(input, root, writes, false)
    }
    // The one-write cyclic caller precharges its entire frame and invokes the
    // MODEL-wide 128-observation guard before each copy, not a reset PID/tape counter.
    pub(super) fn precharged_reference_copy(
        &mut self,
        input: &Grid,
        root: Address,
        source: Address,
        target: Address,
    ) -> Result<RectangleCopyWitness, Error> {
        self.reference_copy_plan_inner(input, root, vec![(source, target)], true)
    }
    fn reference_copy_plan_inner(
        &mut self,
        input: &Grid,
        root: Address,
        writes: Vec<(Address, Address)>,
        precharged: bool,
    ) -> Result<RectangleCopyWitness, Error> {
        if !input.owner.same(&self.space) || writes.is_empty() || (precharged && writes.len() != 1)
        {
            return Err(Error::NotReady);
        }
        if !precharged {
            self.guard((writes.len() * 128 + 128) * 640)?;
        }
        let execution = root.provision(b"execution");
        let machine = Machine::default();
        execution
            .provision(b"machine")
            .publish_prepared_child(&machine.pointer());
        let halt = execution.provision(b"halt");
        halt.terminate();
        let mut next = halt.clone();
        let budget = writes.len();
        for (i, (source, target)) in writes.into_iter().enumerate().rev() {
            let state = execution
                .provision(b"construction")
                .provision(&i.to_be_bytes());
            let arg = state.provision(b"continuation");
            arg.point_to(&next);
            install(&state, &source, &target, &arg);
            let w = root.provision(b"writes").provision(&i.to_be_bytes());
            w.provision(b"source-slot-at-bind").point_to(&source);
            w.provision(b"target-slot-at-bind").point_to(&target);
            w.provision(b"instruction").point_to(&state);
            next = state;
        }
        machine.pointer().point_to(&next);
        Ok(RectangleCopyWitness {
            root: root.clone(),
            machine,
            budget,
            halt,
            complete: false,
            failed: false,
            planned: field(&execution, b"construction")?
                .inspect_children()
                .iter()
                .map(|a| a.identity_key().to_vec())
                .collect(),
            visited: std::collections::BTreeSet::new(),
            input: input.clone(),
            rows: Vec::new(),
            precharged,
            program: root,
            _rectangles: Vec::new(),
            _owners: self.owned_roots().to_vec(),
        })
    }
    pub fn rectangle_copy_program(
        &mut self,
        sources: &[Rc<RectangleProjection>],
        background: &Address,
        initial: &[u8],
        next: &[u8],
    ) -> Result<Address, Error> {
        let roles = [b"NW".as_slice(), b"NE", b"SW", b"SE"];
        if sources.is_empty()
            || !roles.contains(&initial)
            || !roles.contains(&next)
            || sources.iter().any(|p| !p.owner.same(&self.root))
        {
            return Err(Error::NotReady);
        }
        self.guard((sources.len() * 16 + 128) * 640)?;
        let p = self
            .root
            .provision(b"rectangle-copy-programs")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        p.provision(b"owner").point_to(&self.root);
        p.provision(b"background-value").point_to(background);
        for (k, r) in [
            (b"initial-corner".as_slice(), initial),
            (b"next-corner", next),
        ] {
            p.provision(k)
                .point_to(&self.root.provision(b"rectangle-corner-keys").provision(r));
        }
        for (i, source) in sources.iter().enumerate() {
            p.provision(b"source-rectangles")
                .provision(&i.to_be_bytes())
                .point_to(&source.proof);
        }
        Ok(p)
    }
    pub fn prepare_rectangle_copy(
        &mut self,
        input: &Grid,
        program: &Address,
        sources: Vec<Rc<RectangleProjection>>,
    ) -> Result<RectangleCopyWitness, Error> {
        if !input.owner.same(&self.space)
            || !field(program, b"owner")?.follow()?.same(&self.root)
            || sources
                .iter()
                .any(|p| !p.owner.same(&self.root) || !p.input.anchor.same(&input.anchor))
        {
            return Err(Error::NotReady);
        }
        let selected = field(program, b"source-rectangles")?.inspect_children();
        let mut ordered = Vec::new();
        let valid = self
            .root
            .provision(b"rectangle-tags")
            .provision(b"rectangle");
        for slot in selected {
            let proof = slot.follow()?;
            let Some(p) = sources.iter().find(|p| p.proof.same(&proof)) else {
                return Err(Error::Incomplete);
            };
            if !field(&p.proof, b"status")?.follow()?.same(&valid)
                || !p.proof.child(b"committed").is_some_and(|n| n.is_terminal())
            {
                return Err(Error::Incomplete);
            }
            let original = field(
                &field(&p.proof, b"original-count-proof")?.follow()?,
                b"value",
            )?
            .follow()?;
            let covered = field(
                &field(&p.proof, b"covered-count-proof")?.follow()?,
                b"value",
            )?
            .follow()?;
            if !self.context_equal(&original, &covered)? {
                return Err(Error::Incomplete);
            }
            p.check_bindings()?;
            ordered.push(p.clone());
        }
        if ordered.is_empty() {
            return Err(Error::Incomplete);
        }
        let tiles = ordered.iter().try_fold(0usize, |n, p| {
            let rows = field(&p.proof, b"source-rows")?.inspect_children();
            Ok::<_, Error>(
                n + rows
                    .iter()
                    .map(|r| r.child(b"cells").map_or(0, |c| c.inspect_children().len()))
                    .sum::<usize>(),
            )
        })?;
        self.guard((128 * (input.height() * input.width() + tiles) + 512) * 640)?;
        let root = self
            .root
            .provision(b"rectangle-copy-queries")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        root.provision(b"input").point_to(&input.anchor);
        root.provision(b"program").point_to(program);
        let mut rows = Vec::new();
        let mut cells = Vec::new();
        for y in 0..input.height() {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            let mut line = Vec::new();
            for x in 0..input.width() {
                self.rectangle_observe()?;
                let c = row.provision(b"cells").provision(&x.to_be_bytes());
                c.provision(b"before").point_to(&input.cell(y, x)?);
                c.provision(b"value");
                line.push(c);
            }
            rows.push(row);
            cells.push(line);
        }
        for y in 0..input.height() {
            for x in 0..input.width() {
                for (port, at) in [
                    (b"north".as_slice(), y.checked_sub(1).map(|ny| (ny, x))),
                    (b"south", (y + 1 < input.height()).then_some((y + 1, x))),
                    (b"west", x.checked_sub(1).map(|nx| (y, nx))),
                    (b"east", (x + 1 < input.width()).then_some((y, x + 1))),
                ] {
                    cells[y][x].provision(port).point_to(
                        &at.map_or_else(|| self.outside.clone(), |(yy, xx)| cells[yy][xx].clone()),
                    );
                }
            }
        }
        let corners = root.provision(b"initial-destination-corners");
        for (k, y, x) in [
            (b"NW".as_slice(), 0, 0),
            (b"NE", 0, input.width() - 1),
            (b"SW", input.height() - 1, 0),
            (b"SE", input.height() - 1, input.width() - 1),
        ] {
            corners.provision(k).point_to(&cells[y][x]);
        }
        let initial = field(program, b"initial-corner")?.follow()?;
        root.provision(b"initial-corner-at-bind").point_to(&initial);
        let mut dest = field(&corners, initial.token().ok_or(Error::Incomplete)?)?.follow()?;
        let next_role = field(program, b"next-corner")?.follow()?;
        root.provision(b"next-corner-at-bind").point_to(&next_role);
        let bg = field(program, b"background-value")?;
        root.provision(b"background-source-slot").point_to(&bg);
        let mut writes = Vec::new();
        for line in &cells {
            for c in line {
                writes.push((bg.clone(), field(c, b"value")?, None));
            }
        }
        for (rect_index, p) in ordered.iter().enumerate() {
            let placement = root
                .provision(b"placements")
                .provision(&rect_index.to_be_bytes());
            placement.provision(b"source-proof").point_to(&p.proof);
            placement.provision(b"destination-top-left").point_to(&dest);
            let start = dest.clone();
            let mut row_start = dest.clone();
            let mut placed = Vec::new();
            for source_row in field(&p.proof, b"source-rows")?.inspect_children() {
                let mut at = row_start.clone();
                let mut line = Vec::new();
                for source_cell in field(&source_row, b"cells")?.inspect_children() {
                    self.rectangle_observe()?;
                    let source = field(&source_cell, b"source-value-slot")?.follow()?;
                    let original = field(&source_cell, b"source-cell")?.follow()?;
                    if !field(&original, b"value")?.same(&source) {
                        return Err(Error::Incomplete);
                    }
                    let Some(target) = at.child(b"value") else {
                        placement.provision(b"outside-destination").point_to(&at);
                        return Err(Error::Incomplete);
                    };
                    writes.push((source, target, Some(p.proof.clone())));
                    line.push(at.clone());
                    at = field(&at, b"east")?.follow()?;
                }
                if line.is_empty() {
                    return Err(Error::Incomplete);
                }
                placed.push(line);
                row_start = field(&row_start, b"south")?.follow()?;
            }
            let first = placed.first().ok_or(Error::Incomplete)?;
            let last = placed.last().ok_or(Error::Incomplete)?;
            let corner = placement.provision(b"destination-corners");
            for (k, c) in [
                (b"NW".as_slice(), start),
                (b"NE", first.last().ok_or(Error::Incomplete)?.clone()),
                (b"SW", last.first().ok_or(Error::Incomplete)?.clone()),
                (b"SE", last.last().ok_or(Error::Incomplete)?.clone()),
            ] {
                corner.provision(k).point_to(&c);
            }
            dest = field(&corner, next_role.token().ok_or(Error::Incomplete)?)?.follow()?;
        }
        let execution = root.provision(b"execution");
        let machine = Machine::default();
        execution
            .provision(b"machine")
            .publish_prepared_child(&machine.pointer());
        let halt = execution.provision(b"halt");
        halt.terminate();
        let mut next = halt.clone();
        let budget = writes.len();
        for (i, (source, target, producer)) in writes.into_iter().enumerate().rev() {
            let state = execution
                .provision(b"construction")
                .provision(&i.to_be_bytes());
            let arg = state.provision(b"continuation");
            arg.point_to(&next);
            install(&state, &source, &target, &arg);
            let write = root.provision(b"writes").provision(&i.to_be_bytes());
            write.provision(b"source-slot-at-bind").point_to(&source);
            write.provision(b"target-slot-at-bind").point_to(&target);
            write.provision(b"instruction").point_to(&state);
            if let Some(p) = producer {
                write.provision(b"rectangle-proof").point_to(&p);
            }
            next = state;
        }
        machine.pointer().point_to(&next);
        self.guard(0)?;
        Ok(RectangleCopyWitness {
            root,
            machine,
            budget,
            halt,
            complete: false,
            failed: false,
            planned: field(&execution, b"construction")?
                .inspect_children()
                .iter()
                .map(|a| a.identity_key().to_vec())
                .collect(),
            visited: std::collections::BTreeSet::new(),
            input: input.clone(),
            rows,
            precharged: false,
            program: program.clone(),
            _rectangles: sources,
            _owners: self.owned_roots().to_vec(),
        })
    }
}
