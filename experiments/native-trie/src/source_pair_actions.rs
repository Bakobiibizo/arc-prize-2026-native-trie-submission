//! NATIVE-SOURCE-PAIR-ACTIONS-PROTOCOL: learn input address roles and copy
//! actual bound target slots. No colour payload matcher or fixed marker layout.
use super::source_groups::{candidate, candidates, frames};
use super::*;
fn keys(role: &Address, which: &[u8]) -> Result<(Address, Address), Error> {
    let p = field(role, which)?;
    Ok((
        field(&p, b"row-key")?.follow()?,
        field(&p, b"column-key")?.follow()?,
    ))
}
fn role_cell(input: &Address, role: &Address, which: &[u8]) -> Result<Option<Address>, Error> {
    let (y, x) = keys(role, which)?;
    Ok(input
        .child(b"row-data")
        .and_then(|a| a.child(y.token()?))
        .and_then(|a| a.child(b"cells"))
        .and_then(|a| a.child(x.token()?)))
}
fn role_pair(lesson: &GroupLesson, role: &Address, which: &[u8]) -> Result<Option<Address>, Error> {
    let (y, x) = keys(role, which)?;
    Ok(lesson
        .evidence
        .child(b"pairs")
        .and_then(|a| a.child(y.token()?))
        .and_then(|a| a.child(x.token()?)))
}
fn pairs(lesson: &GroupLesson) -> Result<Vec<Address>, Error> {
    let mut out = Vec::new();
    for row in field(&lesson.evidence, b"pairs")?.inspect_children() {
        out.extend(row.inspect_children());
    }
    Ok(out)
}
impl SparseRewrite {
    fn pair_observe(&mut self) -> Result<(), Error> {
        if self.source_pair_observations.is_multiple_of(128) {
            self.guard((128 * 160 + 900 * 24) * 640)?;
        }
        self.source_pair_observations += 1;
        Ok(())
    }
    fn install_pair_role(&self, rule: &Address, which: &[u8], cell: &Address) -> Result<(), Error> {
        let row = field(cell, b"source-row")?.follow()?;
        let p = rule.provision(which);
        for (k, node) in [(b"row-key".as_slice(), row), (b"column-key", cell.clone())] {
            let token = node.token().ok_or(Error::Incomplete)?;
            let key = self
                .root
                .provision(b"source-pair-coordinate-keys")
                .provision(token);
            key.terminate();
            p.provision(k).point_to(&key);
        }
        Ok(())
    }
    fn validate_source_pair(
        &mut self,
        lessons: &[GroupLesson],
        rule: &Address,
    ) -> Result<bool, Error> {
        let unknown = self.unknown.clone();
        for d in lessons {
            let at = rule
                .provision(b"demonstrations")
                .provision(&d.evidence.identity_key());
            let Some(head) = role_cell(&d.input.anchor, rule, b"head")? else {
                at.provision(b"missing-head").terminate();
                return Ok(false);
            };
            let Some(target) = role_cell(&d.input.anchor, rule, b"target")? else {
                at.provision(b"missing-target").terminate();
                return Ok(false);
            };
            let hv = field(&head, b"value")?.follow()?;
            let tv = field(&target, b"value")?.follow()?;
            at.provision(b"head").point_to(&head);
            at.provision(b"target").point_to(&target);
            at.provision(b"head-value-at-bind").point_to(&hv);
            at.provision(b"target-value-at-bind").point_to(&tv);
            if self.context_equal(&hv, &unknown)? || self.context_equal(&tv, &unknown)? {
                at.provision(b"unknown-role").terminate();
                return Ok(false);
            }
            for which in [b"head".as_slice(), b"target"] {
                let Some(pair) = role_pair(d, rule, which)? else {
                    return Ok(false);
                };
                let before = field(&field(&pair, b"before")?.follow()?, b"value")?.follow()?;
                let after = field(&field(&pair, b"after")?.follow()?, b"value")?.follow()?;
                let (tag, fact) = self.context_equality_fact(&before, &after)?;
                at.provision(which)
                    .provision(b"unchanged-fact")
                    .point_to(&fact);
                if !tag.same(&self.yes) {
                    at.provision(b"changed-role-contradiction").point_to(&pair);
                    return Ok(false);
                }
            }
            for pair in pairs(d)? {
                self.pair_observe()?;
                let before = field(&pair, b"before")?.follow()?;
                if before.same(&head) || before.same(&target) {
                    continue;
                }
                let value = field(&before, b"value")?.follow()?;
                let (tag, fact) = self.context_equality_fact(&value, &hv)?;
                if tag.same(&self.yes) {
                    let after = field(&field(&pair, b"after")?.follow()?, b"value")?.follow()?;
                    let (ok, proof) = self.context_equality_fact(&after, &tv)?;
                    let observed = at
                        .provision(b"matching-data")
                        .provision(&pair.identity_key());
                    observed.provision(b"pair").point_to(&pair);
                    observed.provision(b"source-equality").point_to(&fact);
                    observed.provision(b"target-equality").point_to(&proof);
                    if !ok.same(&self.yes) {
                        at.provision(b"data-contradiction").point_to(&observed);
                        return Ok(false);
                    }
                }
            }
        }
        rule.provision(b"all-demonstrations-valid").terminate();
        Ok(true)
    }
    fn source_pair_selected(
        &mut self,
        before: &Address,
        bound: &[(Address, Address, Address)],
        at: &Address,
    ) -> Result<Selection, Error> {
        let selected = at.provision(b"selected-actions");
        let value = field(before, b"value")?.follow()?;
        for (rule, head, target) in bound {
            self.pair_observe()?;
            let observed = at
                .provision(b"rule-predicates")
                .provision(&rule.identity_key());
            observed.provision(b"rule").point_to(rule);
            observed.provision(b"bound-head").point_to(head);
            observed.provision(b"bound-target").point_to(target);
            if before.same(head) || before.same(target) {
                observed.provision(b"own-role-excluded").point_to(before);
                continue;
            }
            let hv = field(head, b"value")?.follow()?;
            let (tag, fact) = self.context_equality_fact(&value, &hv)?;
            observed
                .provision(b"native-source-equality")
                .point_to(&fact);
            if tag.same(&self.yes) {
                let slot = field(target, b"value")?;
                records::record(&selected, b"action", &slot, &observed)?;
            }
        }
        Ok(records::resolve(&selected, b"action")?)
    }
    fn learn_source_pairs(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        let assembly = self
            .root
            .provision(b"source-pair-assemblies")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        for d in lessons {
            assembly
                .provision(b"lessons")
                .provision(&d.evidence.identity_key())
                .point_to(&d.evidence);
            for group in field(&d.evidence, b"groups")?.inspect_children() {
                if matches!(
                    records::resolve(&field(&group, b"targets")?, b"literal")?,
                    Selection::Ambiguous { .. }
                ) {
                    assembly
                        .provision(b"fixed-source-needs-multiple-targets")
                        .point_to(&group);
                    return Ok(None);
                }
            }
        }
        let first = &lessons[0];
        let unchanged = field(&first.evidence, b"unchanged")?;
        let mut valid = Vec::new();
        for source in candidates(&field(&first.evidence, b"changed")?, b"source-colour") {
            let colour = source.follow()?;
            let changed = frames(Some(source))?;
            let Some(example) = changed.first() else {
                continue;
            };
            let desired = field(&field(example, b"after")?.follow()?, b"value")?.follow()?;
            let heads = frames(candidate(&unchanged, b"source-colour", &colour))?;
            if let Some(second) = heads.get(1) {
                // A changed source must differ from its required target. Only the
                // head can exempt an unchanged occurrence of that source; a second
                // one is therefore a negative witness for EVERY candidate pair.
                let witness = assembly.provision(b"two-unchanged-source-occurrences");
                for (k, pair) in [(b"first".as_slice(), &heads[0]), (b"second", second)] {
                    let v = field(&field(pair, b"before")?.follow()?, b"value")?.follow()?;
                    let (tag, fact) = self.context_equality_fact(&v, &colour)?;
                    if !tag.same(&self.yes) {
                        return Err(Error::Incomplete);
                    }
                    witness.provision(k).point_to(pair);
                    witness
                        .provision(k)
                        .provision(b"native-source-equality")
                        .point_to(&fact);
                }
                let (tag, fact) = self.context_equality_fact(&colour, &desired)?;
                if tag.same(&self.yes) {
                    return Err(Error::Incomplete);
                }
                witness
                    .provision(b"positive-source-target-difference")
                    .point_to(&fact);
                witness.provision(b"required-positive").point_to(example);
                return Ok(None);
            }
            for hp in heads {
                for tp in frames(candidate(&unchanged, b"source-colour", &desired))? {
                    self.pair_observe()?;
                    let head = field(&hp, b"before")?.follow()?;
                    let target = field(&tp, b"before")?.follow()?;
                    let hr = field(&head, b"source-row")?.follow()?;
                    let tr = field(&target, b"source-row")?.follow()?;
                    let rule = assembly
                        .provision(b"candidates")
                        .provision(hr.token().ok_or(Error::Incomplete)?)
                        .provision(head.token().ok_or(Error::Incomplete)?)
                        .provision(tr.token().ok_or(Error::Incomplete)?)
                        .provision(target.token().ok_or(Error::Incomplete)?);
                    if rule.child(b"evaluated").is_some() {
                        continue;
                    }
                    self.install_pair_role(&rule, b"head", &head)?;
                    self.install_pair_role(&rule, b"target", &target)?;
                    rule.provision(b"seed-positive").point_to(example);
                    let accepted = self.validate_source_pair(lessons, &rule)?;
                    rule.provision(b"evaluated").terminate();
                    if accepted {
                        valid.push(rule)
                    }
                }
            }
        }
        if valid.is_empty() {
            return Ok(None);
        }
        // No subset/program Cartesian search: the entire independently valid body must
        // satisfy ALL cells with a unique PHYSICAL action source slot.
        for d in lessons {
            let mut bound = Vec::new();
            for rule in &valid {
                let h = role_cell(&d.input.anchor, rule, b"head")?.ok_or(Error::Incomplete)?;
                let t = role_cell(&d.input.anchor, rule, b"target")?.ok_or(Error::Incomplete)?;
                bound.push((rule.clone(), h, t));
            }
            for pair in pairs(d)? {
                let before = field(&pair, b"before")?.follow()?;
                let at = assembly
                    .provision(b"whole-body-validation")
                    .provision(&pair.identity_key());
                at.provision(b"pair").point_to(&pair);
                let selected = self.source_pair_selected(&before, &bound, &at)?;
                let source = match selected {
                    Selection::Unique { target, .. } => target,
                    Selection::Unknown => field(&before, b"value")?,
                    _ => {
                        assembly
                            .provision(b"ambiguous-physical-action-sources")
                            .point_to(&at);
                        return Ok(None);
                    }
                };
                let after = field(&field(&pair, b"after")?.follow()?, b"value")?.follow()?;
                let (tag, fact) = self.context_equality_fact(&source.follow()?, &after)?;
                at.provision(b"native-output-equality").point_to(&fact);
                if !tag.same(&self.yes) {
                    assembly
                        .provision(b"uncovered-or-wrong-action")
                        .point_to(&at);
                    return Ok(None);
                }
            }
        }
        let program = assembly.provision(b"program");
        program.provision(b"source-pair-program").terminate();
        let groups = program.provision(b"groups");
        for (i, rule) in valid.iter().enumerate() {
            groups.provision(&i.to_be_bytes()).point_to(rule);
        }
        assembly.provision(b"all-demonstrations-valid").terminate();
        self.guard(0)?;
        Ok(Some(GroupProgram {
            root: program,
            owner: self.root.clone(),
        }))
    }
    pub fn agree_source_pair_actions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        let prior = self.agree_relative_actions(lessons)?;
        let induced = self.learn_source_pairs(lessons)?;
        let choice = self
            .root
            .provision(b"source-pair-construction-evidence")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        if let Some(program) = &prior {
            choice.provision(b"prior-candidate").point_to(&program.root);
        }
        if let Some(program) = &induced {
            choice
                .provision(b"induced-candidate")
                .point_to(&program.root);
        }
        let selected = match (prior, induced) {
            (Some(a), Some(b)) if b.groups() < a.groups() => Some(b),
            (Some(a), _) => Some(a),
            (_, b) => b,
        };
        if let Some(program) = &selected {
            choice.provision(b"selected").point_to(&program.root);
            choice
                .provision(b"minimum-operation-count-policy")
                .terminate();
        }
        Ok(selected)
    }
    pub fn source_pair_applicable(
        &mut self,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<bool, Error> {
        if program
            .root
            .child(b"episode-transduction-program")
            .is_some()
        {
            if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
                return Err(Error::NotReady);
            }
            let values = (0..input.height())
                .flat_map(|y| (0..input.width()).map(move |x| (y, x)))
                .map(|(y, x)| Ok(field(&input.cell(y, x)?, b"value")?.follow()?))
                .collect::<Result<Vec<_>, Error>>()?;
            let applications = self.episode_transducer.applications;
            return Ok(self
                .episode_transducer
                .transduce_grid(input.height(), input.width(), &values)
                .is_ok()
                && self.episode_transducer.applications > applications);
        }
        if program.root.child(b"intrinsic-addressed-program").is_some() {
            if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
                return Err(Error::NotReady);
            }
            return self
                .intrinsic_actions
                .applicable(input, &self.outside, &self.unknown)
                .map_err(|error| match error {
                    crate::intrinsic_actions::ActionError::Shape => Error::Shape,
                    crate::intrinsic_actions::ActionError::Grid(error) => Error::Grid(error),
                    crate::intrinsic_actions::ActionError::Route(_) => Error::Incomplete,
                });
        }
        if program.root.child(b"cyclic-lattice-program").is_some() {
            return self.cyclic_applicable(input, program);
        }
        if program.root.child(b"ordered-object-program").is_some() {
            return self.ordered_applicable(input, program);
        }
        if program.root.child(b"source-pair-program").is_none() {
            return Ok(true);
        }
        let unknown = self.unknown.clone();
        for w in field(&program.root, b"groups")?.inspect_children() {
            let rule = w.follow()?;
            for which in [b"head".as_slice(), b"target"] {
                let Some(cell) = role_cell(&input.anchor, &rule, which)? else {
                    return Ok(false);
                };
                if self.context_equal(&field(&cell, b"value")?.follow()?, &unknown)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    pub(super) fn apply_source_pairs(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<Grid, Error> {
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        if !self.source_pair_applicable(input, program)? {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        self.frozen = true;
        let root = self
            .root
            .provision(b"group-queries")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        root.provision(b"input").point_to(&input.anchor);
        root.provision(b"program").point_to(&program.root);
        let mut bound = Vec::new();
        for w in field(&program.root, b"groups")?.inspect_children() {
            let rule = w.follow()?;
            let h = role_cell(&input.anchor, &rule, b"head")?.ok_or(Error::Incomplete)?;
            let t = role_cell(&input.anchor, &rule, b"target")?.ok_or(Error::Incomplete)?;
            let snapshot = root
                .provision(b"consumed-groups")
                .provision(&rule.identity_key());
            snapshot.provision(b"head").point_to(&h);
            snapshot.provision(b"target").point_to(&t);
            snapshot
                .provision(b"head-value-at-bind")
                .point_to(&field(&h, b"value")?.follow()?);
            snapshot
                .provision(b"target-value-at-bind")
                .point_to(&field(&t, b"value")?.follow()?);
            bound.push((rule, h, t));
        }
        let mut rows = Vec::new();
        for y in 0..input.height() {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let cell = row.provision(b"cells").provision(&x.to_be_bytes());
                cell.provision(b"before").point_to(&before);
                let selected = self.source_pair_selected(&before, &bound, &cell)?;
                let source = match selected {
                    Selection::Unique { target, .. } => target,
                    Selection::Unknown => field(&before, b"value")?,
                    _ => return Err(Error::Incomplete),
                };
                self.emit(&source, &cell.provision(b"value"))?;
                cell.provision(b"emitted-source-slot").point_to(&source);
                cell.provision(b"written-value")
                    .point_to(&field(&cell, b"value")?.follow()?);
            }
            rows.push(row);
        }
        self.guard(0)?;
        Ok(s.derived(input, &program.root, &rows)?)
    }
}
