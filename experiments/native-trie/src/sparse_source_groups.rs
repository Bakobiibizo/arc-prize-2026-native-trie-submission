//! docs/sparse-relations/SPARSE-SOURCE-GROUPS-PROTOCOL.md. Native mask facts, per-source constraint
//! aggregation and explicit all-demo applicability; not a colour/opcode decoder.
use super::*;
pub struct GroupLesson {
    pub evidence: Address,
    pub changed: usize,
    pub groups: usize,
    pub(super) input: Grid,
    pub(super) output: Grid,
    pub(super) owner: Address,
}
#[derive(Clone)]
pub struct GroupProgram {
    pub root: Address,
    pub(super) owner: Address,
}
impl GroupProgram {
    pub fn groups(&self) -> usize {
        self.root
            .child(b"groups")
            .map_or(0, |a| a.inspect_children().len())
    }
}
pub(super) fn candidate(root: &Address, head: &[u8], value: &Address) -> Option<Address> {
    root.child(b"holds")?
        .child(head)?
        .child(b"candidates")?
        .child(&value.identity_key())
}
pub(super) fn candidates(root: &Address, head: &[u8]) -> Vec<Address> {
    root.child(b"holds")
        .and_then(|a| a.child(head))
        .and_then(|a| a.child(b"candidates"))
        .map_or_else(Vec::new, |a| a.inspect_children())
}
pub(super) fn frames(c: Option<Address>) -> Result<Vec<Address>, Error> {
    match c {
        None => Ok(Vec::new()),
        Some(c) => field(&c, b"evidence")?
            .inspect_children()
            .iter()
            .map(|w| Ok(w.follow()?))
            .collect(),
    }
}
impl SparseRewrite {
    pub fn derive_groups(
        &mut self,
        id: &[u8],
        input: &Grid,
        output: &Grid,
    ) -> Result<GroupLesson, Error> {
        if self.frozen || !input.owner.same(&self.space) || !output.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        if input.width() != output.width() || input.height() != output.height() {
            return Err(Error::Shape);
        }
        self.guard((128 * 160 + 900 * 24) * 640 + id.len())?;
        let bank = self.root.provision(b"group-lessons");
        if bank.child(id).is_some() {
            return Err(Error::NotReady);
        }
        let evidence = bank.provision(id);
        evidence.provision(b"input").point_to(&input.anchor);
        evidence.provision(b"output").point_to(&output.anchor);
        let changed = evidence.provision(b"changed");
        let unchanged = evidence.provision(b"unchanged");
        for y in 0..input.height() {
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let after = output.cell(y, x)?;
                let pair = evidence
                    .provision(b"pairs")
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes());
                pair.provision(b"before").point_to(&before);
                pair.provision(b"after").point_to(&after);
                let a = field(&before, b"value")?.follow()?;
                let b = field(&after, b"value")?.follow()?;
                let (dest, proof) = self.compare(&a, &b, &unchanged, &changed)?;
                pair.provision(b"change-proof").point_to(&proof);
                records::record(&dest, b"source-colour", &a, &pair)?;
            }
        }
        let mut count = 0;
        let sources = candidates(&changed, b"source-colour");
        for c in &sources {
            let colour = c.follow()?;
            let group = evidence
                .provision(b"groups")
                .provision(&colour.identity_key());
            group.provision(b"source-colour").point_to(&colour);
            let targets = group.provision(b"targets");
            for pair in frames(Some(c.clone()))? {
                let after = field(&pair, b"after")?.follow()?;
                let t = field(&after, b"value")?.follow()?;
                records::record(&targets, b"literal", &t, &pair)?;
                count += 1;
            }
        }
        self.guard(0)?;
        Ok(GroupLesson {
            evidence,
            changed: count,
            groups: sources.len(),
            input: input.clone(),
            output: output.clone(),
            owner: self.root.clone(),
        })
    }
    pub fn agree_groups(&mut self, lessons: &[GroupLesson]) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        let assembly = self
            .root
            .provision(b"group-assemblies")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        let sources = assembly.provision(b"source-groups");
        for (i, d) in lessons.iter().enumerate() {
            assembly
                .provision(b"lessons")
                .provision(&i.to_be_bytes())
                .point_to(&d.evidence);
            for c in candidates(&field(&d.evidence, b"changed")?, b"source-colour") {
                records::record(&sources, b"source", &c.follow()?, &c)?;
            }
        }
        let mut signature = self.root.provision(b"group-encodings");
        let mut body = Vec::new();
        for source in candidates(&sources, b"source") {
            let colour = source.follow()?;
            if colour.same(&self.unknown) {
                return Ok(None);
            }
            let group = assembly
                .provision(b"groups")
                .provision(&colour.identity_key());
            group.provision(b"source-colour").point_to(&colour);
            let targets = group.provision(b"target-alternatives");
            let mut positives = Vec::new();
            let mut negatives = Vec::new();
            for d in lessons {
                let changed = field(&d.evidence, b"changed")?;
                let unchanged = field(&d.evidence, b"unchanged")?;
                let p = frames(candidate(&changed, b"source-colour", &colour))?;
                // Every supplied demo participates. No positive source occurrence is
                // not a wildcard for its unchanged cells: these are actual negatives.
                let n = frames(candidate(&unchanged, b"source-colour", &colour))?;
                let scope = group
                    .provision(b"applicability")
                    .provision(&d.evidence.identity_key());
                scope
                    .provision(if p.is_empty() && n.is_empty() {
                        b"source-absent"
                    } else {
                        b"source-present"
                    })
                    .terminate();
                if let Some(per) = d
                    .evidence
                    .child(b"groups")
                    .and_then(|a| a.child(&colour.identity_key()))
                {
                    for t in candidates(&field(&per, b"targets")?, b"literal") {
                        records::record(&targets, b"literal", &t.follow()?, &per)?;
                    }
                }
                positives.extend(p.into_iter().map(|pair| (d, pair)));
                negatives.extend(n.into_iter().map(|pair| (d, pair)));
            }
            let target = match records::resolve(&targets, b"literal")? {
                Selection::Unique { target, .. } => target,
                _ => {
                    group.provision(b"target-conflict").terminate();
                    return Ok(None);
                }
            };
            if target.same(&self.unknown) {
                return Ok(None);
            }
            let literal = self
                .root
                .provision(b"literal-action-sources")
                .provision(&target.identity_key());
            literal.point_to(&target);
            group.provision(b"literal-action-slot").point_to(&literal);
            signature = signature
                .provision(&colour.identity_key())
                .provision(&target.identity_key());
            if negatives.is_empty() {
                group.provision(b"source-only").terminate();
                signature = signature.provision(b"native-true-identity");
            } else {
                let known = group.provision(b"positive-kinds");
                let unknown = group.provision(b"unavailable-positive-kinds");
                for (d, pair) in positives {
                    let before = field(&pair, b"before")?.follow()?;
                    let (proof, value) = self.component_value(&d.input, &before)?;
                    let unavailable = self
                        .root
                        .provision(b"native-components")
                        .provision(b"relation-tags")
                        .provision(b"unresolved");
                    let (dest, gate) = self.compare(&value, &unavailable, &unknown, &known)?;
                    gate.provision(b"component-proof").point_to(&proof);
                    gate.provision(b"positive-pair").point_to(&pair);
                    records::record(&dest, b"value", &value, &gate)?;
                }
                if !matches!(records::resolve(&unknown, b"value")?, Selection::Unknown) {
                    return Ok(None);
                }
                let expected = match records::resolve(&known, b"value")? {
                    Selection::Unique { target, .. } => target,
                    _ => return Ok(None),
                };
                group.provision(b"expected-kind").point_to(&expected);
                signature = signature.provision(&expected.identity_key());
                let yes = self.yes.clone();
                let rejected = group.provision(b"counterexamples");
                let accepted = group.provision(b"accepted-negatives");
                let total = negatives.len();
                for (i, (d, pair)) in negatives.into_iter().enumerate() {
                    let before = field(&pair, b"before")?.follow()?;
                    let tag = self.predicate_component(
                        &d.input,
                        &before,
                        &group,
                        &pair
                            .provision(b"group-negative")
                            .provision(&group.identity_key()),
                    )?;
                    let (dest, proof) = self.compare(&tag, &yes, &rejected, &accepted)?;
                    records::record(&dest, b"pair", &pair, &proof)?;
                    if !matches!(records::resolve(&rejected, b"pair")?, Selection::Unknown) {
                        group
                            .provision(b"negative-checked")
                            .provision(&(i + 1).to_be_bytes());
                        group
                            .provision(b"negative-unchecked")
                            .provision(&(total - i - 1).to_be_bytes());
                        return Ok(None);
                    }
                }
                group
                    .provision(b"negative-checked")
                    .provision(&total.to_be_bytes());
                group.provision(b"all-negatives-checked").terminate();
            }
            body.push(group);
        }
        let program = signature.provision(b"program");
        let groups = program.provision(b"groups");
        for (i, g) in body.iter().enumerate() {
            groups.provision(&i.to_be_bytes()).point_to(g);
        }
        assembly.provision(b"encoding").point_to(&program);
        records::record(&program, b"support", &program, &assembly)?;
        self.guard(0)?;
        Ok(Some(GroupProgram {
            root: program,
            owner: self.root.clone(),
        }))
    }
    pub fn apply_groups(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<Grid, Error> {
        let output = self.apply_groups_inner(s, input, program)?;
        Ok(output.retain_owners(self.owned_roots().to_vec()))
    }
    fn apply_groups_inner(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<Grid, Error> {
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
            let episode = self.serial.to_be_bytes();
            self.persistent_arc()?;
            let disk = self.persistent_arc.as_ref().ok_or(Error::Incomplete)?;
            let evidence = disk
                .store_root()
                .provision(b"resident-plan-evidence")
                .map_err(|_| Error::Incomplete)?
                .provision(&program.root.identity_key())
                .map_err(|_| Error::Incomplete)?;
            evidence.terminate().map_err(|_| Error::Incomplete)?;
            let output = self
                .episode_transducer
                .transduce_grid_persistent(
                    disk,
                    &episode,
                    &evidence,
                    input.height(),
                    input.width(),
                    &values,
                )
                .map_err(transduction_error)?;
            let disk_root = disk.store_root();
            let input_state = disk_root
                .locate([b"arc".as_slice(), b"states", b"input", episode.as_slice()])
                .map_err(|_| Error::Incomplete)?
                .ok_or(Error::Incomplete)?;
            let output_state = disk_root
                .locate([b"arc".as_slice(), b"states", b"output", episode.as_slice()])
                .map_err(|_| Error::Incomplete)?
                .ok_or(Error::Incomplete)?;
            let receipts =
                crate::disk_trie::DiskReceipts::new(&disk_root).map_err(|_| Error::Incomplete)?;
            receipts
                .issue(
                    &episode,
                    &input_state,
                    &output_state,
                    std::slice::from_ref(&evidence),
                    b"resident-cue-retrieval-persistent-execution",
                )
                .map_err(|_| Error::Incomplete)?;
            receipts
                .associate_outcome(
                    &episode,
                    b"execution-complete",
                    crate::disk_trie::OutcomeAssessment::Unclassified,
                )
                .map_err(|_| Error::Incomplete)?;
            let root = self
                .root
                .provision(b"episode-transduction-results")
                .provision(&episode);
            root.provision(b"persistent-execution-evidence")
                .provision(&program.root.identity_key())
                .terminate();
            root.provision(b"persistent-receipt")
                .provision(&episode)
                .terminate();
            self.serial += 1;
            let mut rows = Vec::with_capacity(input.height());
            for (y, output_values) in output.iter().enumerate() {
                let row = root.provision(b"rows").provision(&y.to_be_bytes());
                for (x, output_value) in output_values.iter().copied().enumerate() {
                    let value = self
                        .root
                        .provision(b"persistent-output-values")
                        .provision(&[output_value]);
                    value.terminate();
                    row.provision(b"cells")
                        .provision(&x.to_be_bytes())
                        .provision(b"value")
                        .point_to(&value);
                }
                rows.push(row);
            }
            self.frozen = true;
            return Ok(s.derived(input, &program.root, &rows)?);
        }
        if program.root.child(b"intrinsic-addressed-program").is_some() {
            if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
                return Err(Error::NotReady);
            }
            self.frozen = true;
            return self
                .intrinsic_actions
                .render(s, input, &program.root, &self.outside, &self.unknown)
                .map_err(|error| match error {
                    crate::intrinsic_actions::ActionError::Shape => Error::Shape,
                    crate::intrinsic_actions::ActionError::Grid(error) => Error::Grid(error),
                    crate::intrinsic_actions::ActionError::Route(_) => Error::Incomplete,
                });
        }
        if program
            .root
            .child(b"canonical-whole-grid-transform")
            .is_some()
            || program.root.child(b"progression-construction").is_some()
            || program
                .root
                .child(b"orthogonal-connector-construction")
                .is_some()
            || program.root.child(b"axis-beam-construction").is_some()
            || program.root.child(b"move-to-touch-construction").is_some()
            || program.root.child(b"sparse-lattice-construction").is_some()
            || program
                .root
                .child(b"alternating-stripe-construction")
                .is_some()
            || program.root.child(b"boundary-ray-construction").is_some()
            || program
                .root
                .child(b"component-stencil-construction")
                .is_some()
            || program.root.child(b"paired-line-construction").is_some()
            || program
                .root
                .child(b"component-pair-fill-construction")
                .is_some()
            || program
                .root
                .child(b"ordinal-column-fill-construction")
                .is_some()
            || program
                .root
                .child(b"nearest-marker-envelope-construction")
                .is_some()
            || program
                .root
                .child(b"lattice-separator-construction")
                .is_some()
            || program.root.child(b"corner-halo-construction").is_some()
            || program.root.child(b"corner-pixel-construction").is_some()
            || program
                .root
                .child(b"periodic-diamond-construction")
                .is_some()
            || program
                .root
                .child(b"glyph-classifier-construction")
                .is_some()
            || program
                .root
                .child(b"reflect-internal-motif-construction")
                .is_some()
            || program.root.child(b"marker-cross-construction").is_some()
            || program.root.child(b"arrow-ray-construction").is_some()
            || program.root.child(b"legend-block-construction").is_some()
            || program.root.child(b"template-ray-construction").is_some()
            || program
                .root
                .child(b"relocate-diagram-construction")
                .is_some()
            || program
                .root
                .child(b"append-marker-period-construction")
                .is_some()
            || program.root.child(b"adaptive-inset-construction").is_some()
            || program
                .root
                .child(b"center-marker-transfer-construction")
                .is_some()
            || program
                .root
                .child(b"tiled-mode-denoise-construction")
                .is_some()
            || program
                .root
                .child(b"square-partition-construction")
                .is_some()
            || program.root.child(b"marker-network-construction").is_some()
            || program
                .root
                .child(b"axis-profile-projection-construction")
                .is_some()
            || program
                .root
                .child(b"separator-template-construction")
                .is_some()
            || program
                .root
                .child(b"missing-corner-path-construction")
                .is_some()
            || program.root.child(b"partial-cross-construction").is_some()
        {
            return self.apply_broad_structural(s, input, program);
        }
        if program.root.child(b"cyclic-lattice-program").is_some() {
            return self.apply_cyclic_actions(s, input, program);
        }
        if program.root.child(b"ordered-object-program").is_some() {
            return self.apply_ordered_objects(s, input, program);
        }
        if program.root.child(b"path-copy-program").is_some() {
            return self.apply_path_copy(s, input, program);
        }
        if program.root.child(b"source-pair-program").is_some() {
            return self.apply_source_pairs(s, input, program);
        }
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
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
        let scope = root.provision(b"scope");
        let mut variable_groups = Vec::new();
        for wrapper in field(&program.root, b"groups")?.inspect_children() {
            let g = wrapper.follow()?;
            let snapshot = root
                .provision(b"consumed-groups")
                .provision(&g.identity_key());
            if let Some(slot) = g.child(b"source-colour") {
                let colour = slot.follow()?;
                records::record(&scope, b"group", &colour, &g)?;
                snapshot.provision(b"source-colour").point_to(&colour);
            } else {
                let exclusions = field(&g, b"excluded-source-colours")?;
                for (i, e) in exclusions.inspect_children().iter().enumerate() {
                    snapshot
                        .provision(b"excluded-source-colours")
                        .provision(&i.to_be_bytes())
                        .point_to(&e.follow()?);
                }
                snapshot.provision(b"variable-source").terminate();
                variable_groups.push(g.clone());
            }
            let literal = field(&g, b"literal-action-slot")?.follow()?;
            snapshot
                .provision(b"literal-action-slot")
                .point_to(&literal);
            snapshot
                .provision(b"literal-value")
                .point_to(&literal.follow()?);
            if let Some(k) = g.child(b"expected-kind") {
                snapshot.provision(b"expected-kind").point_to(&k.follow()?);
            }
            if let Some(c) = g.child(b"expected-count") {
                snapshot.provision(b"expected-count").point_to(&c.follow()?);
            }
            if let Some(r) = g.child(b"relative-role") {
                let role = r.follow()?;
                snapshot.provision(b"relative-role").point_to(&role);
                snapshot
                    .provision(b"relative-colour")
                    .point_to(&field(&role, b"context-colour")?.follow()?);
                snapshot
                    .provision(b"expected-relative-count")
                    .point_to(&field(&g, b"expected-relative-count")?.follow()?);
            }
            if let Some(r) = g.child(b"context-role") {
                let role = r.follow()?;
                snapshot.provision(b"context-role").point_to(&role);
                snapshot
                    .provision(b"context-colour")
                    .point_to(&field(&role, b"context-colour")?.follow()?);
                if let Some(k) = role.child(b"neighbour-kind") {
                    snapshot
                        .provision(b"context-neighbour-kind")
                        .point_to(&k.follow()?);
                }
                snapshot
                    .provision(b"expected-context-count")
                    .point_to(&field(&g, b"expected-context-count")?.follow()?);
            }
        }
        let yes = self.yes.clone();
        let no = self.no.clone();
        let mut rows = Vec::new();
        for y in 0..input.height() {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let value = field(&before, b"value")?.follow()?;
                let source = field(&before, b"value")?;
                let cell = row.provision(b"cells").provision(&x.to_be_bytes());
                cell.provision(b"before").point_to(&before);
                let active = cell.provision(b"selected-actions");
                let inactive = cell.provision(b"inactive-actions");
                let mut applicable = frames(candidate(&scope, b"group", &value))?;
                applicable.extend(variable_groups.iter().cloned());
                for g in applicable {
                    let at = cell
                        .provision(b"group-predicates")
                        .provision(&g.identity_key());
                    let tag = if g.child(b"relative-role").is_some() {
                        self.predicate_relative(input, &before, &g, &at)?
                    } else if g.child(b"context-role").is_some() {
                        self.predicate_context(input, &before, &g, &at)?
                    } else if g.child(b"excluded-source-colours").is_some() {
                        self.predicate_variable_count(input, &before, &g, &at)?
                    } else if g.child(b"expected-count").is_some() {
                        self.predicate_count(input, &before, &g, &at)?
                    } else if g.child(b"expected-kind").is_some() {
                        self.predicate_component(input, &before, &g, &at)?
                    } else {
                        let (tag, proof) = self.compare(&yes, &yes, &yes, &no)?;
                        at.provision(b"native-true-identity").point_to(&proof);
                        tag
                    };
                    let (dest, proof) = self.compare(&tag, &yes, &active, &inactive)?;
                    proof.provision(b"group").point_to(&g);
                    let literal = field(&g, b"literal-action-slot")?.follow()?;
                    records::record(&dest, b"action", &literal, &proof)?;
                }
                let selected = match records::resolve(&active, b"action")? {
                    Selection::Unique { target, .. } => target,
                    Selection::Unknown => source,
                    _ => return Err(Error::Incomplete),
                };
                self.emit(&selected, &cell.provision(b"value"))?;
                cell.provision(b"emitted-source-slot").point_to(&selected);
                cell.provision(b"written-value")
                    .point_to(&field(&cell, b"value")?.follow()?);
            }
            rows.push(row);
        }
        self.guard(0)?;
        Ok(s.derived(input, &program.root, &rows)?)
    }
}
