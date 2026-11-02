//! Supplied physical-cycle/E-S lattice prototype, not an ARC learner.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{source_field, target_field};
    fn input(s: &Grids) -> Grid {
        let source = s
            .source(b"cyclic-unit", b"declared sparse cycle fixture")
            .unwrap();
        s.admit(
            b"cyclic-unit",
            crate::arc_grids::Side::TrainingInput,
            &[
                vec![Some(2), Some(8), Some(3)],
                vec![Some(0); 3],
                vec![Some(0); 3],
            ],
            &source,
        )
        .unwrap()
    }
    #[test]
    fn copy_source_target_next_and_phase_condition_remain_native() {
        for case in 0..4 {
            let s = Grids::new();
            let mut m = SparseRewrite::new(&s);
            let input = input(&s);
            let scope = m.root.provision(b"copy-mutation");
            let source = input.cell(0, 0).unwrap().child(b"value").unwrap();
            let alternate = input.cell(0, 1).unwrap().child(b"value").unwrap();
            let target = scope.provision(b"target");
            let other = scope.provision(b"other-target");
            let mut tape = m
                .reference_copy_plan(
                    &input,
                    scope.clone(),
                    vec![(source.clone(), target.clone())],
                )
                .unwrap();
            let state = tape.machine.current().unwrap();
            match case {
                0 => source_field(&state).unwrap().point_to(&alternate),
                1 => target_field(&state).unwrap().point_to(&other),
                2 => {
                    let continuation = crate::next_field(&state).unwrap().follow().unwrap();
                    target_field(&state).unwrap().point_to(&continuation);
                }
                _ => {
                    let bg = input
                        .cell(2, 2)
                        .unwrap()
                        .child(b"value")
                        .unwrap()
                        .follow()
                        .unwrap();
                    let p = m.cyclic_program(&bg, 3).unwrap();
                    let phases = p.root.child(b"phases").unwrap().inspect_children();
                    let slot = scope.provision(b"phase-condition");
                    slot.point_to(&phases[0]);
                    assert!(
                        m.cyclic_select(&slot, &phases, &scope.provision(b"before"))
                            .unwrap()
                            .same(&phases[0])
                    );
                    slot.point_to(&phases[1]);
                    assert!(
                        m.cyclic_select(&slot, &phases, &scope.provision(b"after"))
                            .unwrap()
                            .same(&phases[1])
                    );
                }
            }
            if case == 2 {
                assert!(tape.execute(2).is_err());
                assert!(tape.commit_native().is_err());
                assert!(scope.child(b"committed").is_none());
            } else {
                tape.execute(1).unwrap();
                tape.commit_native().unwrap();
                if case == 0 {
                    assert!(target.follow().unwrap().same(&alternate.follow().unwrap()));
                }
                if case == 1 {
                    assert!(target.follow().is_err());
                    assert!(other.follow().unwrap().same(&source.follow().unwrap()));
                }
            }
        }
    }
    #[test]
    fn precharged_single_copy_has_identical_native_bytes_and_global_observation_progress() {
        let s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let input = input(&s);
        let source = input.cell(0, 0).unwrap().child(b"value").unwrap();
        let a = m.root.provision(b"r0");
        let b = m.root.provision(b"r1");
        let mut regular = m
            .reference_copy_plan(
                &input,
                a.clone(),
                vec![(source.clone(), a.provision(b"target"))],
            )
            .unwrap();
        m.guard(256 * 640).unwrap();
        m.rectangle_observe().unwrap();
        let mut precharged = m
            .precharged_reference_copy(&input, b.clone(), source, b.provision(b"target"))
            .unwrap();
        assert_eq!(
            crate::region_witness::RegionWitness::storage_charge(&a),
            crate::region_witness::RegionWitness::storage_charge(&b)
        );
        regular.execute(1).unwrap();
        precharged.execute(1).unwrap();
        regular.commit_native().unwrap();
        precharged.commit_native().unwrap();
        assert_eq!(
            crate::region_witness::RegionWitness::storage_charge(&a),
            crate::region_witness::RegionWitness::storage_charge(&b)
        );
        assert!(
            a.child(b"target")
                .unwrap()
                .follow()
                .unwrap()
                .same(&b.child(b"target").unwrap().follow().unwrap())
        );
        assert_eq!(m.rectangle_observations, 1);
    }
    #[test]
    fn changed_input_cannot_reuse_knownness_and_partial_tapes_do_not_commit() {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let input = input(&s);
        let bg = input
            .cell(2, 2)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap();
        let p = m.cyclic_program(&bg, 3).unwrap();
        let old = m.cyclic_lattice(&mut s, &input, &p).unwrap();
        let snapshot = old.inspect().unwrap();
        input
            .cell(0, 2)
            .unwrap()
            .child(b"value")
            .unwrap()
            .point_to(&m.unknown);
        assert!(m.cyclic_lattice(&mut s, &input, &p).is_err());
        assert_eq!(old.inspect().unwrap(), snapshot);
        let root = m.root.provision(b"partial-tape");
        let source = input.cell(0, 0).unwrap().child(b"value").unwrap();
        let mut tape = m
            .reference_copy_plan(
                &input,
                root.clone(),
                vec![
                    (source.clone(), root.provision(b"one")),
                    (source, root.provision(b"two")),
                ],
            )
            .unwrap();
        // Bounded partial execution reports Incomplete; it is resumable but
        // cannot produce a committed result until every planned state ran.
        assert!(matches!(tape.execute(1), Err(Error::Incomplete)));
        assert!(tape.commit_native().is_err());
        tape.execute(1).unwrap();
        tape.commit_native().unwrap();
    }
}

use super::*;
#[derive(Clone)]
pub struct CyclicProgram {
    pub root: Address,
    owner: Address,
}
pub(super) struct CyclicProjection {
    pub proof: Address,
    pub rows: Vec<Address>,
    input: Grid,
    owners: Vec<Address>,
}
impl CyclicProjection {
    pub(super) fn export(self, s: &mut Grids) -> Result<Grid, Error> {
        let out = s.derived(&self.input, &self.proof, &self.rows)?;
        let mut owners = self.owners;
        owners.push(self.proof);
        Ok(out.retain_owners(owners))
    }
}
impl SparseRewrite {
    pub fn cyclic_program(
        &mut self,
        background: &Address,
        length: usize,
    ) -> Result<CyclicProgram, Error> {
        if length == 0 || length > 900 {
            return Err(Error::NotReady);
        }
        self.guard((length * 32 + 128) * 640)?;
        let p = self
            .root
            .provision(b"cyclic-programs")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        p.provision(b"background-value").point_to(background);
        let nodes: Vec<_> = (0..length)
            .map(|i| p.provision(b"phases").provision(&i.to_be_bytes()))
            .collect();
        let mut next = nodes[0].clone();
        for node in nodes.iter().rev() {
            node.provision(b"east-next").point_to(&next);
            node.provision(b"south-next").point_to(&next);
            next = node.clone();
        }
        p.provision(b"start-phase").point_to(&nodes[0]);
        Ok(CyclicProgram {
            root: p,
            owner: self.root.clone(),
        })
    }
    fn cyclic_select(
        &mut self,
        slot: &Address,
        phases: &[Address],
        proof: &Address,
    ) -> Result<Address, Error> {
        let actual = slot.follow()?;
        let mut selected = None;
        for phase in phases {
            self.rectangle_observe()?;
            let (tag, fact) = self.context_equality_fact(&actual, phase)?;
            proof.provision(&phase.identity_key()).point_to(&fact);
            if tag.same(&self.yes) {
                if selected.is_some() {
                    return Err(Error::Incomplete);
                }
                selected = Some(phase.clone());
            }
        }
        selected.ok_or(Error::Incomplete)
    }
    fn cyclic_copy(
        &mut self,
        input: &Grid,
        scope: Address,
        source: Address,
        target: Address,
    ) -> Result<(), Error> {
        self.rectangle_observe()?;
        let mut tape = self.precharged_reference_copy(input, scope, source, target)?;
        tape.execute(tape.budget)?;
        tape.commit_native()?;
        Ok(())
    }
    pub fn cyclic_lattice(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &CyclicProgram,
    ) -> Result<Grid, Error> {
        self.cyclic_projection(input, program)?.export(s)
    }
    pub(super) fn cyclic_projection(
        &mut self,
        input: &Grid,
        program: &CyclicProgram,
    ) -> Result<CyclicProjection, Error> {
        if !input.owner.same(&self.space) || !program.owner.same(&self.root) {
            return Err(Error::NotReady);
        }
        let phases = field(&program.root, b"phases")?.inspect_children();
        if phases.is_empty() || phases.len() > 900 {
            return Err(Error::Incomplete);
        }
        self.guard((128 * (input.height() * input.width() * 6 + phases.len() * 4) + 512) * 640)?;
        let root = self
            .root
            .provision(b"cyclic-queries")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        root.provision(b"input").point_to(&input.anchor);
        root.provision(b"program").point_to(&program.root);
        let supplied = root.provision(b"supplied-at-bind");
        for key in [b"background-value".as_slice(), b"start-phase"] {
            supplied
                .provision(key)
                .point_to(&field(&program.root, key)?.follow()?);
        }
        for phase in &phases {
            let at = supplied
                .provision(b"phases")
                .provision(&phase.identity_key());
            at.provision(b"phase").point_to(phase);
            for key in [b"east-next".as_slice(), b"south-next"] {
                at.provision(key).point_to(&field(phase, key)?.follow()?);
            }
        }
        // Fresh guards: cached knownness from an earlier invocation cannot
        // certify input slots that have since changed to unknown.
        let mut guards = Vec::new();
        for y in 0..input.height() {
            for x in 0..input.width() {
                guards.push((field(&input.cell(y, x)?, b"value")?, self.unknown.clone()));
            }
        }
        let (known, _) = self.run_set_count(root.provision(b"known-empty-set"), &guards, input)?;
        let value = field(&known, b"value")?.follow()?;
        let unavailable = self
            .root
            .provision(b"native-components")
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let (tag, fact) = self.property_knownness(&known, &value, &unavailable)?;
        root.provision(b"all-input-known").point_to(&fact);
        if !tag.same(&self.yes) {
            return Err(Error::Incomplete);
        }
        let bg = field(&program.root, b"background-value")?.follow()?;
        let mut rows = Vec::new();
        for y in 0..input.height() {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..input.width() {
                let at = row.provision(b"cells").provision(&x.to_be_bytes());
                at.provision(b"before").point_to(&input.cell(y, x)?);
                at.provision(b"input-value-at-bind")
                    .point_to(&field(&input.cell(y, x)?, b"value")?.follow()?);
                let target = at.provision(b"phase");
                if y == 0 && x == 0 {
                    self.cyclic_select(
                        &field(&program.root, b"start-phase")?,
                        &phases,
                        &at.provision(b"initial-membership"),
                    )?;
                    self.cyclic_copy(
                        input,
                        at.provision(b"initial-copy"),
                        field(&program.root, b"start-phase")?,
                        target.clone(),
                    )?;
                } else {
                    for (parent, port) in [
                        (
                            if x > 0 {
                                Some(field(&field(&row, b"cells")?, &(x - 1).to_be_bytes())?)
                            } else {
                                None
                            },
                            b"east-next".as_slice(),
                        ),
                        (
                            if y > 0 {
                                Some(field(&field(&rows[y - 1], b"cells")?, &x.to_be_bytes())?)
                            } else {
                                None
                            },
                            b"south-next".as_slice(),
                        ),
                    ] {
                        if let Some(parent) = parent {
                            let producer = at.provision(port);
                            let phase = self.cyclic_select(
                                &field(&parent, b"phase")?,
                                &phases,
                                &producer.provision(b"source-phase-membership"),
                            )?;
                            let source = field(&phase, port)?;
                            self.cyclic_select(
                                &source,
                                &phases,
                                &producer.provision(b"successor-membership"),
                            )?;
                            let candidate = producer.provision(b"phase");
                            self.cyclic_copy(
                                input,
                                producer.provision(b"native-copy"),
                                source,
                                candidate.clone(),
                            )?;
                            if target.follow().is_ok() {
                                let (tag, fact) = self.context_equality_fact(
                                    &target.follow()?,
                                    &candidate.follow()?,
                                )?;
                                producer.provision(b"parent-agreement").point_to(&fact);
                                if !tag.same(&self.yes) {
                                    root.provision(b"inconsistent-parents").point_to(&at);
                                    return Err(Error::Incomplete);
                                }
                            } else {
                                self.cyclic_copy(
                                    input,
                                    producer.provision(b"commit-phase"),
                                    candidate,
                                    target.clone(),
                                )?;
                            }
                        }
                    }
                }
            }
            rows.push(row);
        }
        // Bind all input evidence, never a decoded phase index or teacher colour.
        for row in &rows {
            for at in field(row, b"cells")?.inspect_children() {
                self.rectangle_observe()?;
                let before = field(&at, b"before")?.follow()?;
                let slot = field(&before, b"value")?;
                let value = slot.follow()?;
                let (tag, fact) = self.context_equality_fact(&value, &bg)?;
                at.provision(b"native-background-equality").point_to(&fact);
                if tag.same(&self.yes) {
                    continue;
                }
                let phase = self.cyclic_select(
                    &field(&at, b"phase")?,
                    &phases,
                    &at.provision(b"binding-phase-membership"),
                )?;
                let binding = root.provision(b"bindings").provision(&phase.identity_key());
                let source = binding.provision(b"source-value-slot");
                if let Ok(old) = source.follow() {
                    let (tag, fact) = self.context_equality_fact(&old.follow()?, &value)?;
                    at.provision(b"native-binding-agreement").point_to(&fact);
                    if !tag.same(&self.yes) {
                        root.provision(b"conflicting-input-binding").point_to(&at);
                        return Err(Error::Incomplete);
                    }
                } else {
                    let argument = at.provision(b"input-slot-reference");
                    argument.point_to(&slot);
                    self.cyclic_copy(
                        input,
                        at.provision(b"bind-original-source"),
                        argument,
                        source,
                    )?;
                }
                binding.provision(b"phase").point_to(&phase);
            }
        }
        // Every phase needs a binding even if a small output happens not to visit it.
        for phase in &phases {
            let _ = field(
                &root.provision(b"bindings").provision(&phase.identity_key()),
                b"source-value-slot",
            )?
            .follow()?;
        }
        for row in &rows {
            for at in field(row, b"cells")?.inspect_children() {
                let phase = self.cyclic_select(
                    &field(&at, b"phase")?,
                    &phases,
                    &at.provision(b"emission-phase-membership"),
                )?;
                let source = field(
                    &field(&root, b"bindings")?
                        .child(&phase.identity_key())
                        .ok_or(Error::Incomplete)?,
                    b"source-value-slot",
                )?
                .follow()?;
                at.provision(b"emitted-source-slot").point_to(&source);
                self.cyclic_copy(
                    input,
                    at.provision(b"native-emission"),
                    source,
                    at.provision(b"value"),
                )?;
            }
        }
        self.guard(0)?;
        root.provision(b"committed").terminate();
        Ok(CyclicProjection {
            proof: root,
            rows,
            input: input.clone(),
            owners: self.owned_roots().to_vec(),
        })
    }
}
