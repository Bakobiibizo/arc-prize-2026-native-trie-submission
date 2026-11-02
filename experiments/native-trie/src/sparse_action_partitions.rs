//! docs/native/NATIVE-ACTION-PARTITIONS-PROTOCOL.md: refine only contradicted input prefixes.
use super::source_groups::{candidate, candidates, frames};
#[cfg(test)]
#[path = "action_partition_cache_tests.rs"]
mod cache_tests;
use super::*;
use crate::{complete_count_witness::CompleteCountWitness, region_witness::RegionWitness};
use std::collections::BTreeMap;
#[derive(Clone)]
pub(super) struct Row {
    pub(super) input: Grid,
    pub(super) pair: Address,
    pub(super) changed: bool,
}
#[derive(Clone)]
pub(super) struct Prefix {
    pub(super) colour: Address,
    pub(super) kind: Option<Address>,
    pub(super) count: Option<Address>,
}
impl SparseRewrite {
    pub(super) fn property_knownness(
        &mut self,
        producer: &Address,
        value: &Address,
        unavailable: &Address,
    ) -> Result<(Address, Address), Error> {
        if !producer
            .child(b"committed")
            .is_some_and(|n| n.is_terminal())
        {
            return Err(Error::Incomplete);
        }
        let fact = self
            .root
            .provision(b"property-knownness-facts")
            .provision(&producer.identity_key())
            .provision(&value.identity_key())
            .provision(&unavailable.identity_key());
        if fact.child(b"committed").is_some_and(|n| n.is_terminal()) {
            return Ok((field(&fact, b"result")?.follow()?, fact));
        }
        let yes = self.yes.clone();
        let no = self.no.clone();
        let (tag, proof) = self.compare(value, unavailable, &no, &yes)?;
        fact.provision(b"producer-proof").point_to(producer);
        fact.provision(b"value-at-bind").point_to(value);
        fact.provision(b"unavailable-at-bind").point_to(unavailable);
        fact.provision(b"native-comparison").point_to(&proof);
        fact.provision(b"result").point_to(&tag);
        fact.provision(b"committed").terminate();
        Ok((tag, fact))
    }
    pub(super) fn complete_count_value(
        &mut self,
        input: &Grid,
        cell: &Address,
    ) -> Result<(Address, Address), Error> {
        let (region_proof, _) = self.component_value(input, cell)?;
        let cache = self.root.provision(b"complete-count-cache");
        if let Some(slot) = cache.child(&region_proof.identity_key()) {
            let proof = slot.follow()?;
            if !proof.child(b"committed").is_some_and(|n| n.is_terminal())
                || !field(&proof, b"region-proof")?
                    .follow()?
                    .same(&region_proof)
            {
                return Err(Error::Incomplete);
            }
            return Ok((proof.clone(), field(&proof, b"value")?.follow()?));
        }
        let owner = self.root.provision(b"native-components");
        let region = RegionWitness::projection_from_committed(&owner, input, &region_proof)?;
        self.guard(CompleteCountWitness::reserve_bytes(&region)?)?;
        let mut witness =
            CompleteCountWitness::prepare(region, &self.root, &self.count_proofs.to_be_bytes())?;
        self.count_proofs += 1;
        let execution = witness.execute(witness.budget);
        self.count_copies += witness.machine.cycles;
        execution?;
        let p = witness.record()?;
        cache
            .provision(&region_proof.identity_key())
            .point_to(&p.proof);
        self.guard((128 * 160 + 900 * 24) * 640)?;
        Ok((p.proof.clone(), p.value.clone()))
    }
    pub(super) fn predicate_count(
        &mut self,
        input: &Grid,
        cell: &Address,
        group: &Address,
        at: &Address,
    ) -> Result<Address, Error> {
        let kind = self.predicate_component(input, cell, group, &at.provision(b"kind"))?;
        let gate = at.provision(b"kind-applicability");
        records::record(&gate, b"tag", &kind, at)?;
        let yes = self.yes.clone();
        let no = self.no.clone();
        // Actual native false-kind evidence scopes count provisioning away entirely.
        if candidate(&gate, b"tag", &yes).is_none() {
            return Ok(kind);
        }
        let (proof, value) = self.complete_count_value(input, cell)?;
        let expected = field(group, b"expected-count")?.follow()?;
        at.provision(b"complete-count-proof").point_to(&proof);
        at.provision(b"count-consumed").point_to(&value);
        at.provision(b"expected-count-consumed").point_to(&expected);
        let unavailable = self
            .root
            .provision(b"complete-count-tags")
            .provision(b"unavailable");
        let (known, kproof) = self.property_knownness(&proof, &value, &unavailable)?;
        let (matched, mproof) = self.compare(&value, &expected, &yes, &no)?;
        let conjunction = at.provision(b"conjunction");
        records::record(&conjunction, b"predicate", &kind, &gate)?;
        records::record(&conjunction, b"predicate", &known, &kproof)?;
        records::record(&conjunction, b"predicate", &matched, &mproof)?;
        self.tag(&conjunction)
    }
    pub(super) fn refine_prefix(
        &mut self,
        rows: Vec<Row>,
        prefix: Prefix,
        at: &Address,
        body: &mut Vec<Address>,
    ) -> Result<bool, Error> {
        if !rows.iter().any(|r| r.changed) {
            at.provision(b"unchanged-only").terminate();
            return Ok(true);
        }
        let labels = at.provision(b"after-alternatives");
        for (i, row) in rows.iter().enumerate() {
            if i.is_multiple_of(128) {
                self.guard((128 * 160 + 900 * 24) * 640)?;
            }
            let after = field(&row.pair, b"after")?.follow()?;
            let value = field(&after, b"value")?.follow()?;
            records::record(&labels, b"value", &value, &row.pair)?;
        }
        if let Selection::Unique { target, .. } = records::resolve(&labels, b"value")? {
            if target.same(&self.unknown) {
                at.provision(b"unavailable-action").terminate();
                return Ok(false);
            }
            let leaf = at.provision(b"action");
            leaf.provision(b"source-colour").point_to(&prefix.colour);
            if let Some(k) = prefix.kind {
                leaf.provision(b"expected-kind").point_to(&k);
            }
            if let Some(c) = prefix.count {
                leaf.provision(b"expected-count").point_to(&c);
            }
            let literal = self
                .root
                .provision(b"literal-action-sources")
                .provision(&target.identity_key());
            literal.point_to(&target);
            leaf.provision(b"literal-action-slot").point_to(&literal);
            leaf.provision(b"constraint-prefix").point_to(at);
            at.provision(b"all-observed-actions-agree").terminate();
            body.push(leaf);
            return Ok(true);
        }
        at.provision(b"action-conflict").terminate();
        if prefix.count.is_some() {
            at.provision(b"unseparated-contradiction").terminate();
            return Ok(false);
        }
        let count_stage = prefix.kind.is_some();
        at.provision(b"refined-by")
            .provision(if count_stage {
                b"complete-count"
            } else {
                b"kind"
            })
            .terminate();
        let mut buckets: BTreeMap<_, (Address, Vec<Row>)> = BTreeMap::new();
        for (i, row) in rows.into_iter().enumerate() {
            if i.is_multiple_of(128) {
                self.guard((128 * 160 + 900 * 24) * 640)?;
            }
            let before = field(&row.pair, b"before")?.follow()?;
            let (proof, value) = if count_stage {
                self.complete_count_value(&row.input, &before)?
            } else {
                self.component_value(&row.input, &before)?
            };
            let unavailable = if count_stage {
                self.root
                    .provision(b"complete-count-tags")
                    .provision(b"unavailable")
            } else {
                self.root
                    .provision(b"native-components")
                    .provision(b"relation-tags")
                    .provision(b"unresolved")
            };
            let observed = at
                .provision(b"input-properties")
                .provision(&row.pair.identity_key());
            observed.provision(b"producer-proof").point_to(&proof);
            observed.provision(b"value").point_to(&value);
            observed.provision(b"pair").point_to(&row.pair);
            let (known, fact) = self.property_knownness(&proof, &value, &unavailable)?;
            let knowledge = observed.provision(b"knownness");
            records::record(&knowledge, b"tag", &known, &fact)?;
            observed.provision(b"native-knownness-fact").point_to(&fact);
            if candidate(&knowledge, b"tag", &self.yes).is_none() {
                if row.changed {
                    at.provision(b"unavailable-positive").point_to(&observed);
                    return Ok(false);
                }
                at.provision(b"unavailable-negative")
                    .provision(&row.pair.identity_key())
                    .point_to(&observed);
                continue;
            }
            buckets
                .entry(value.identity_key())
                .or_insert_with(|| (value.clone(), Vec::new()))
                .1
                .push(row);
        }
        for (_, (value, subset)) in buckets {
            let mut next = prefix.clone();
            if count_stage {
                next.count = Some(value.clone())
            } else {
                next.kind = Some(value.clone())
            }
            let part = at.provision(b"partitions").provision(&value.identity_key());
            part.provision(b"value").point_to(&value);
            if !self.refine_prefix(subset, next, &part, body)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub fn agree_partitions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        let assembly = self
            .root
            .provision(b"action-partition-assemblies")
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
        let mut body = Vec::new();
        for c in candidates(&sources, b"source") {
            let colour = c.follow()?;
            if colour.same(&self.unknown) {
                return Ok(None);
            }
            let mut rows = Vec::new();
            let node = assembly
                .provision(b"source-prefixes")
                .provision(&colour.identity_key());
            node.provision(b"source-colour").point_to(&colour);
            for d in lessons {
                let p = frames(candidate(
                    &field(&d.evidence, b"changed")?,
                    b"source-colour",
                    &colour,
                ))?;
                let n = frames(candidate(
                    &field(&d.evidence, b"unchanged")?,
                    b"source-colour",
                    &colour,
                ))?;
                node.provision(b"applicability")
                    .provision(&d.evidence.identity_key())
                    .provision(if p.is_empty() && n.is_empty() {
                        b"source-absent"
                    } else {
                        b"source-present"
                    })
                    .terminate();
                rows.extend(p.into_iter().map(|pair| Row {
                    input: d.input.clone(),
                    pair,
                    changed: true,
                }));
                rows.extend(n.into_iter().map(|pair| Row {
                    input: d.input.clone(),
                    pair,
                    changed: false,
                }));
            }
            if !self.refine_prefix(
                rows,
                Prefix {
                    colour,
                    kind: None,
                    count: None,
                },
                &node,
                &mut body,
            )? {
                return Ok(None);
            }
        }
        let mut encoding = self.root.provision(b"action-partition-encodings");
        for leaf in &body {
            encoding = encoding.provision(&field(leaf, b"source-colour")?.follow()?.identity_key());
            encoding = if let Some(k) = leaf.child(b"expected-kind") {
                encoding.provision(&k.follow()?.identity_key())
            } else {
                encoding.provision(b"source-only")
            };
            encoding = if let Some(c) = leaf.child(b"expected-count") {
                encoding.provision(&c.follow()?.identity_key())
            } else {
                encoding.provision(b"no-count")
            };
            encoding = encoding.provision(
                &field(leaf, b"literal-action-slot")?
                    .follow()?
                    .follow()?
                    .identity_key(),
            );
        }
        let program = encoding.provision(b"program");
        let groups = program.provision(b"groups");
        for (i, leaf) in body.iter().enumerate() {
            groups.provision(&i.to_be_bytes()).point_to(leaf);
        }
        assembly.provision(b"encoding").point_to(&program);
        records::record(&program, b"support", &program, &assembly)?;
        self.guard(0)?;
        Ok(Some(GroupProgram {
            root: program,
            owner: self.root.clone(),
        }))
    }
}
