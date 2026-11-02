//! Learn path-membership selectors and input-neighbour copy roles from ALL I/O.
use super::action_partitions::Row;
use super::source_groups::{candidates, frames};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
fn copy_parameters(source: &Address, target: &Address) -> Result<(), Error> {
    target
        .provision(b"action-port")
        .point_to(&field(source, b"action-port")?.follow()?);
    target
        .provision(b"schema-binding")
        .point_to(&field(source, b"schema-binding")?.follow()?);
    if source.child(b"always").is_some() {
        target.provision(b"always").terminate();
    } else {
        target
            .provision(b"expected-membership")
            .point_to(&field(source, b"expected-membership")?.follow()?);
        if source.child(b"expected-action-source").is_some() {
            target.provision(b"expected-action-source").terminate();
        }
        for step in field(source, b"step-word")?.inspect_children() {
            target
                .provision(b"step-word")
                .provision(step.token().ok_or(Error::Incomplete)?)
                .point_to(&step.follow()?);
        }
    }
    Ok(())
}
impl SparseRewrite {
    fn validate_path_rule(
        &mut self,
        rows: &[Row],
        rule: &Address,
        at: &Address,
    ) -> Result<bool, Error> {
        for row in rows {
            self.path_observe()?;
            let before = field(&row.pair, b"before")?.follow()?;
            let cell = at
                .provision(b"constraints")
                .provision(&row.pair.identity_key());
            cell.provision(b"pair").point_to(&row.pair);
            let mut source = field(&before, b"value")?;
            if let Some(action) = self.copy_action_slot(&before, rule)? {
                let tag = self.path_predicate(
                    &row.input,
                    &before,
                    rule,
                    &action,
                    &cell.provision(b"selector"),
                )?;
                cell.provision(b"native-selector-result").point_to(&tag);
                if tag.same(&self.yes) {
                    source = action;
                }
            } else {
                cell.provision(b"action-unavailable").terminate();
            }
            let after = field(&field(&row.pair, b"after")?.follow()?, b"value")?.follow()?;
            let (tag, fact) = self.context_equality_fact(&source.follow()?, &after)?;
            cell.provision(b"actual-source-slot").point_to(&source);
            cell.provision(b"native-output-equality").point_to(&fact);
            if !tag.same(&self.yes) {
                at.provision(b"contradiction").point_to(&cell);
                return Ok(false);
            }
        }
        at.provision(b"all-observations-valid").terminate();
        Ok(true)
    }
    fn learn_path_copy(&mut self, lessons: &[GroupLesson]) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        self.path_upper.set(0);
        self.path_guard((128 + lessons.len() * 4) * 640)?;
        let assembly = self
            .root
            .provision(b"path-copy-assemblies")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        let mut rows = Vec::new();
        let mut bycolour: BTreeMap<_, (Address, Vec<Row>)> = BTreeMap::new();
        for d in lessons {
            assembly
                .provision(b"lessons")
                .provision(&d.evidence.identity_key())
                .point_to(&d.evidence);
            let mut changed = BTreeSet::new();
            for source in candidates(&field(&d.evidence, b"changed")?, b"source-colour") {
                for pair in frames(Some(source))? {
                    changed.insert(pair.identity_key());
                }
            }
            for row in field(&d.evidence, b"pairs")?.inspect_children() {
                for pair in row.inspect_children() {
                    let value = field(&field(&pair, b"before")?.follow()?, b"value")?.follow()?;
                    let r = Row {
                        input: d.input.clone(),
                        changed: changed.contains(&pair.identity_key()),
                        pair,
                    };
                    bycolour
                        .entry(value.identity_key())
                        .or_insert_with(|| (value, Vec::new()))
                        .1
                        .push(r.clone());
                    rows.push(r);
                }
            }
        }
        if !rows.iter().any(|r| r.changed) {
            return Ok(None);
        }
        let mut best: Option<GroupProgram> = None;
        for port in PORTS {
            let port_key = self.root.provision(b"path-copy-port-keys").provision(port);
            let proposal = assembly.provision(b"action-alternatives").provision(port);
            let test_rule = proposal.provision(b"action-rule");
            test_rule.provision(b"action-port").point_to(&port_key);
            let mut valid_action = true;
            for row in rows.iter().filter(|r| r.changed) {
                self.path_observe()?;
                let before = field(&row.pair, b"before")?.follow()?;
                let Some(action) = self.copy_action_slot(&before, &test_rule)? else {
                    proposal
                        .provision(b"missing-positive-action")
                        .point_to(&row.pair);
                    valid_action = false;
                    break;
                };
                let desired = field(&field(&row.pair, b"after")?.follow()?, b"value")?.follow()?;
                let (tag, fact) = self.context_equality_fact(&action.follow()?, &desired)?;
                if !tag.same(&self.yes) {
                    proposal
                        .provision(b"positive-action-contradiction")
                        .point_to(&row.pair);
                    proposal
                        .provision(b"native-action-equality")
                        .point_to(&fact);
                    valid_action = false;
                    break;
                }
            }
            if !valid_action {
                continue;
            }
            self.path_guard(256 * 640)?;
            let mut schemata = Vec::new();
            let always = proposal.provision(b"schemata").provision(b"always");
            always.provision(b"always").terminate();
            always.provision(b"action-port").point_to(&port_key);
            always.provision(b"schema-binding").point_to(&always);
            schemata.push(always);
            let tags = self
                .root
                .provision(b"native-path-memberships")
                .provision(b"set-membership-tags");
            for (label, expected_action, word) in [
                (
                    b"self-se".as_slice(),
                    false,
                    vec![b"south".as_slice(), b"east"],
                ),
                (b"action-s", true, vec![b"south".as_slice()]),
            ] {
                for expected in [tags.provision(b"present"), tags.provision(b"absent")] {
                    let schema = proposal
                        .provision(b"schemata")
                        .provision(label)
                        .provision(expected.token().ok_or(Error::Incomplete)?);
                    schema.provision(b"action-port").point_to(&port_key);
                    schema.provision(b"schema-binding").point_to(&schema);
                    schema.provision(b"expected-membership").point_to(&expected);
                    if expected_action {
                        schema.provision(b"expected-action-source").terminate();
                    }
                    for (i, p) in word.iter().enumerate() {
                        let key = self.root.provision(b"path-copy-port-keys").provision(p);
                        schema
                            .provision(b"step-word")
                            .provision(&i.to_be_bytes())
                            .point_to(&key);
                    }
                    schemata.push(schema);
                }
            }
            let mut selected = Vec::new();
            let mut valid_sources = true;
            for (colour, subset) in bycolour
                .values()
                .filter(|(_, r)| r.iter().any(|x| x.changed))
            {
                let mut chosen = None;
                for schema in &schemata {
                    self.path_guard(128 * 640)?;
                    let rule = proposal
                        .provision(b"source-alternatives")
                        .provision(&colour.identity_key())
                        .provision(&schema.identity_key());
                    rule.provision(b"source-colour").point_to(colour);
                    copy_parameters(schema, &rule)?;
                    if self.validate_path_rule(subset, &rule, &rule.provision(b"validation"))? {
                        chosen = Some(rule);
                        break;
                    }
                }
                let Some(rule) = chosen else {
                    proposal
                        .provision(b"no-selector-for-source")
                        .point_to(colour);
                    valid_sources = false;
                    break;
                };
                selected.push(rule);
            }
            if !valid_sources {
                continue;
            }
            let mut grouped: BTreeMap<_, Vec<Address>> = BTreeMap::new();
            for rule in selected {
                grouped
                    .entry(field(&rule, b"schema-binding")?.follow()?.identity_key())
                    .or_default()
                    .push(rule);
            }
            let mut body = Vec::new();
            for (_, same) in grouped {
                if same.len() < 2 {
                    body.extend(same);
                    continue;
                }
                self.path_guard(256 * 640)?;
                let merged = proposal
                    .provision(b"generalized-sources")
                    .provision(&field(&same[0], b"schema-binding")?.follow()?.identity_key());
                copy_parameters(&same[0], &merged)?;
                for rule in &same {
                    let colour = field(rule, b"source-colour")?.follow()?;
                    merged
                        .provision(b"positive-source-bindings")
                        .provision(&colour.identity_key())
                        .point_to(&colour);
                    merged
                        .provision(b"positive-source-rules")
                        .provision(&rule.identity_key())
                        .point_to(rule);
                }
                let excluded = merged.provision(b"excluded-source-colours");
                for (colour, subset) in bycolour.values() {
                    let at = merged
                        .provision(b"all-source-validation")
                        .provision(&colour.identity_key());
                    if !self.validate_path_rule(subset, &merged, &at)? {
                        excluded.provision(&colour.identity_key()).point_to(colour);
                    }
                }
                body.push(merged);
            }
            let program = proposal.provision(b"program");
            program.provision(b"path-copy-program").terminate();
            for (i, rule) in body.iter().enumerate() {
                program
                    .provision(b"groups")
                    .provision(&i.to_be_bytes())
                    .point_to(rule);
            }
            let p = GroupProgram {
                root: program,
                owner: self.root.clone(),
            };
            if !self.validate_path_programme(
                lessons,
                &p,
                &proposal.provision(b"whole-body-validation"),
            )? {
                continue;
            }
            if best.as_ref().is_none_or(|b| p.groups() < b.groups()) {
                best = Some(p);
            }
        }
        self.guard(0)?;
        Ok(best)
    }
    fn path_selected(
        &mut self,
        input: &Grid,
        before: &Address,
        program: &GroupProgram,
        at: &Address,
    ) -> Result<Selection, Error> {
        let colour = field(before, b"value")?.follow()?;
        let active = at.provision(b"selected-actions");
        for w in field(&program.root, b"groups")?.inspect_children() {
            self.path_observe()?;
            let rule = w.follow()?;
            let observed = at
                .provision(b"rule-predicates")
                .provision(&rule.identity_key());
            observed.provision(b"rule").point_to(&rule);
            observed
                .provision(b"action-port-at-bind")
                .point_to(&field(&rule, b"action-port")?.follow()?);
            let mut scoped = true;
            if let Some(s) = rule.child(b"source-colour") {
                let (tag, fact) = self.context_equality_fact(&colour, &s.follow()?)?;
                observed.provision(b"native-source-scope").point_to(&fact);
                scoped = tag.same(&self.yes);
            } else {
                for e in field(&rule, b"excluded-source-colours")?.inspect_children() {
                    let (tag, fact) = self.context_equality_fact(&colour, &e.follow()?)?;
                    observed
                        .provision(b"negative-scope")
                        .provision(&e.identity_key())
                        .point_to(&fact);
                    if tag.same(&self.yes) {
                        scoped = false;
                        break;
                    }
                }
            }
            if !scoped {
                continue;
            }
            let Some(action) = self.copy_action_slot(before, &rule)? else {
                observed.provision(b"action-unavailable").terminate();
                continue;
            };
            let tag = self.path_predicate(
                input,
                before,
                &rule,
                &action,
                &observed.provision(b"selector"),
            )?;
            observed.provision(b"native-selector-result").point_to(&tag);
            if tag.same(&self.yes) {
                records::record(&active, b"action", &action, &observed)?;
            }
        }
        Ok(records::resolve(&active, b"action")?)
    }
    fn validate_path_programme(
        &mut self,
        lessons: &[GroupLesson],
        program: &GroupProgram,
        at: &Address,
    ) -> Result<bool, Error> {
        for d in lessons {
            for row in field(&d.evidence, b"pairs")?.inspect_children() {
                for pair in row.inspect_children() {
                    let before = field(&pair, b"before")?.follow()?;
                    let observed = at.provision(&pair.identity_key());
                    observed.provision(b"pair").point_to(&pair);
                    let source = match self.path_selected(&d.input, &before, program, &observed)? {
                        Selection::Unique { target, .. } => target,
                        Selection::Unknown => field(&before, b"value")?,
                        _ => {
                            at.provision(b"ambiguous-source-slots").point_to(&observed);
                            return Ok(false);
                        }
                    };
                    let after = field(&field(&pair, b"after")?.follow()?, b"value")?.follow()?;
                    let (tag, fact) = self.context_equality_fact(&source.follow()?, &after)?;
                    observed
                        .provision(b"native-output-equality")
                        .point_to(&fact);
                    if !tag.same(&self.yes) {
                        at.provision(b"wrong-output").point_to(&observed);
                        return Ok(false);
                    }
                }
            }
        }
        at.provision(b"all-observations-valid").terminate();
        Ok(true)
    }
    pub fn agree_path_value_actions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        let prior = self.agree_source_pair_actions(lessons)?;
        let induced = self.learn_path_copy(lessons)?;
        let choice = self
            .root
            .provision(b"path-copy-construction-evidence")
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
    pub(super) fn apply_path_copy(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<Grid, Error> {
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        self.path_upper.set(0);
        self.frozen = true;
        let root = self
            .root
            .provision(b"group-queries")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        root.provision(b"input").point_to(&input.anchor);
        root.provision(b"program").point_to(&program.root);
        let mut rows = Vec::new();
        for y in 0..input.height() {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let cell = row.provision(b"cells").provision(&x.to_be_bytes());
                cell.provision(b"before").point_to(&before);
                let source = match self.path_selected(input, &before, program, &cell)? {
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
