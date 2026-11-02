//! docs/native/NATIVE-BINDING-GENERALIZATION-PROTOCOL.md. Merge repeated literal bindings
//! only after ALL-colour native counterexample checks, retaining learned exclusions.
use super::source_groups::{candidate, candidates};
use super::*;
use std::collections::BTreeMap;
struct Proposal {
    count: Address,
    slot: Address,
    root: Address,
    leaves: Vec<Address>,
}
impl SparseRewrite {
    pub fn generalize_source_bindings(
        &mut self,
        lessons: &[GroupLesson],
        program: GroupProgram,
    ) -> Result<GroupProgram, Error> {
        if self.frozen
            || !program.owner.same(&self.root)
            || lessons.is_empty()
            || lessons.iter().any(|d| !d.owner.same(&self.root))
        {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        let assembly = self
            .root
            .provision(b"binding-generalizations")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        assembly
            .provision(b"original-program")
            .point_to(&program.root);
        let old: Vec<_> = field(&program.root, b"groups")?
            .inspect_children()
            .iter()
            .map(|w| w.follow())
            .collect::<Result<_, _>>()?;
        let mut proposals = BTreeMap::new();
        for leaf in &old {
            if leaf.child(b"context-role").is_some() || leaf.child(b"relative-role").is_some() {
                continue;
            }
            let Some(count_slot) = leaf.child(b"expected-count") else {
                continue;
            };
            let count = count_slot.follow()?;
            let slot = field(leaf, b"literal-action-slot")?.follow()?;
            let source = field(leaf, b"source-colour")?.follow()?;
            let root = assembly
                .provision(b"candidates")
                .provision(&count.identity_key())
                .provision(&slot.identity_key());
            records::record(&root, b"source-binding", &source, leaf)?;
            let p = proposals
                .entry((count.identity_key(), slot.identity_key()))
                .or_insert_with(|| Proposal {
                    count,
                    slot,
                    root,
                    leaves: Vec::new(),
                });
            p.leaves.push(leaf.clone());
        }
        let mut eligible = BTreeMap::new();
        for (key, p) in proposals {
            if matches!(records::resolve(&p.root,b"source-binding")?,Selection::Ambiguous{alternatives}if alternatives>=2)
            {
                eligible.insert(key, p);
            }
        }
        let proposals = eligible;
        if proposals.is_empty() {
            assembly
                .provision(b"no-distinct-bindings-to-merge")
                .terminate();
            return Ok(program);
        }
        let counts = assembly.provision(b"candidate-counts");
        for p in proposals.values() {
            records::record(&counts, b"value", &p.count, &p.root)?;
        }
        let mut observed = 0usize;
        for lesson in lessons {
            for row in field(&lesson.evidence, b"pairs")?.inspect_children() {
                for pair in row.inspect_children() {
                    if observed.is_multiple_of(128) {
                        self.guard((128 * 160 + 900 * 24) * 640)?;
                    }
                    observed += 1;
                    let before = field(&pair, b"before")?.follow()?;
                    let colour = field(&before, b"value")?.follow()?;
                    let (region, _) = self.component_value(&lesson.input, &before)?;
                    let observed_count = field(&region, b"observed-cardinality")?.follow()?;
                    // Necessary scope filter only: a different observed cursor either
                    // remains different when complete, or becomes unavailable. Never
                    // bind/validate a count here without the native completeness witness.
                    if candidate(&counts, b"value", &observed_count).is_none() {
                        continue;
                    }
                    let (proof, count) = self.complete_count_value(&lesson.input, &before)?;
                    let unavailable = self
                        .root
                        .provision(b"complete-count-tags")
                        .provision(b"unavailable");
                    let (known, fact) = self.property_knownness(&proof, &count, &unavailable)?;
                    let knowledge = pair
                        .provision(b"generalization-knownness")
                        .provision(&assembly.identity_key());
                    records::record(&knowledge, b"tag", &known, &fact)?;
                    if candidate(&knowledge, b"tag", &self.yes).is_none() {
                        continue;
                    }
                    let after = field(&pair, b"after")?.follow()?;
                    let label = field(&after, b"value")?.follow()?;
                    for p in proposals.values().filter(|p| p.count.same(&count)) {
                        let accepted = p.root.provision(b"accepted-sources");
                        let rejected = p.root.provision(b"negative-sources");
                        let target = p.slot.follow()?;
                        let (dest, native) = self.compare(&label, &target, &accepted, &rejected)?;
                        native.provision(b"pair").point_to(&pair);
                        native.provision(b"complete-count-proof").point_to(&proof);
                        records::record(&dest, b"source", &colour, &native)?;
                    }
                }
            }
        }
        let mut accepted = Vec::new();
        for p in proposals.values() {
            let good = p.root.provision(b"accepted-sources");
            let bad = p.root.provision(b"negative-sources");
            // Preserve disjointness with coarser DIFFERENT actions even when the
            // particular count was absent in their training scope.
            for leaf in &old {
                if leaf.child(b"expected-count").is_none()
                    && !field(leaf, b"literal-action-slot")?.follow()?.same(&p.slot)
                {
                    records::record(
                        &bad,
                        b"source",
                        &field(leaf, b"source-colour")?.follow()?,
                        leaf,
                    )?;
                }
            }
            let conflicting = candidates(&bad, b"source").iter().any(|c| {
                c.follow()
                    .is_ok_and(|s| candidate(&good, b"source", &s).is_some())
            });
            if conflicting {
                p.root
                    .provision(b"positive-negative-binding-conflict")
                    .terminate();
                continue;
            }
            if candidates(&good, b"source").is_empty() {
                p.root.provision(b"no-known-support").terminate();
                continue;
            }
            accepted.push(p);
        }
        // Two open variable-source actions at the same count could overlap for
        // an unseen colour: do not choose an arbitrary winner or payload match.
        let mut count_actions = BTreeMap::new();
        for p in &accepted {
            *count_actions
                .entry(p.count.identity_key())
                .or_insert(0usize) += 1;
        }
        accepted.retain(|p| {
            let valid = count_actions[&p.count.identity_key()] == 1;
            if !valid {
                p.root.provision(b"variable-action-overlap").terminate();
            }
            valid
        });
        if accepted.is_empty() {
            assembly
                .provision(b"proposals-rejected-original-kept")
                .terminate();
            return Ok(program);
        }
        let mut body = Vec::new();
        for leaf in old {
            if !accepted
                .iter()
                .any(|p| p.leaves.iter().any(|old| old.same(&leaf)))
            {
                body.push(leaf)
            }
        }
        for p in accepted {
            let leaf = p.root.provision(b"generalized-action");
            leaf.provision(b"expected-count").point_to(&p.count);
            leaf.provision(b"literal-action-slot").point_to(&p.slot);
            let exclusions = leaf.provision(b"excluded-source-colours");
            for (i, c) in candidates(&p.root.provision(b"negative-sources"), b"source")
                .iter()
                .enumerate()
            {
                exclusions
                    .provision(&i.to_be_bytes())
                    .point_to(&c.follow()?);
            }
            leaf.provision(b"validation").point_to(&p.root);
            p.root
                .provision(b"all-known-count-observations-validated")
                .terminate();
            body.push(leaf);
        }
        let result = assembly.provision(b"program");
        let groups = result.provision(b"groups");
        for (i, leaf) in body.iter().enumerate() {
            groups.provision(&i.to_be_bytes()).point_to(leaf);
        }
        assembly.provision(b"encoding").point_to(&result);
        self.guard(0)?;
        Ok(GroupProgram {
            root: result,
            owner: self.root.clone(),
        })
    }
    pub(super) fn predicate_variable_count(
        &mut self,
        input: &Grid,
        cell: &Address,
        group: &Address,
        at: &Address,
    ) -> Result<Address, Error> {
        let yes = self.yes.clone();
        let no = self.no.clone();
        let source = field(cell, b"value")?.follow()?;
        let unknown = self.unknown.clone();
        let checks = at.provision(b"predicates");
        let (known, proof) = self.compare(&source, &unknown, &no, &yes)?;
        records::record(&checks, b"predicate", &known, &proof)?;
        for exclusion in field(group, b"excluded-source-colours")?.inspect_children() {
            let excluded = exclusion.follow()?;
            let (tag, proof) = self.compare(&source, &excluded, &no, &yes)?;
            proof
                .provision(b"excluded-source-consumed")
                .point_to(&excluded);
            records::record(&checks, b"predicate", &tag, &proof)?;
        }
        let scope = self.tag(&checks)?;
        let applicability = at.provision(b"source-applicability");
        records::record(&applicability, b"tag", &scope, &checks)?;
        if candidate(&applicability, b"tag", &yes).is_none() {
            return Ok(scope);
        }
        let (producer, count) = self.complete_count_value(input, cell)?;
        let unavailable = self
            .root
            .provision(b"complete-count-tags")
            .provision(b"unavailable");
        let (known, fact) = self.property_knownness(&producer, &count, &unavailable)?;
        let expected = field(group, b"expected-count")?.follow()?;
        let (tag, proof) = self.compare(&count, &expected, &yes, &no)?;
        proof
            .provision(b"expected-count-consumed")
            .point_to(&expected);
        proof.provision(b"count-consumed").point_to(&count);
        proof.provision(b"count-producer").point_to(&producer);
        records::record(&checks, b"predicate", &known, &fact)?;
        records::record(&checks, b"predicate", &tag, &proof)?;
        self.tag(&checks)
    }
}
