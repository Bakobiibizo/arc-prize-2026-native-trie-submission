//! Generic component-reference sets. Native comparisons decide all semantic
//! inclusion; ordinary physical set indexing deduplicates actual committed proofs.
#[cfg(test)]
#[path = "component_context_tests.rs"]
mod tests;
use super::*;
use crate::reference_set_count::SetCountWitness;
use std::collections::BTreeMap;
impl SparseRewrite {
    pub(super) fn context_equal(&mut self, a: &Address, b: &Address) -> Result<bool, Error> {
        Ok(self.context_equality_fact(a, b)?.0.same(&self.yes))
    }
    pub(super) fn context_equality_fact(
        &mut self,
        a: &Address,
        b: &Address,
    ) -> Result<(Address, Address), Error> {
        let fact = self
            .root
            .provision(b"context-equalities")
            .provision(&a.identity_key())
            .provision(&b.identity_key());
        let tag = if let Some(slot) = fact.child(b"result") {
            slot.follow()?
        } else {
            let yes = self.yes.clone();
            let no = self.no.clone();
            let (tag, proof) = self.compare(a, b, &yes, &no)?;
            fact.provision(b"native-comparison").point_to(&proof);
            fact.provision(b"a-at-bind").point_to(a);
            fact.provision(b"b-at-bind").point_to(b);
            fact.provision(b"result").point_to(&tag);
            tag
        };
        // Branch on the actual native boolean, never host equality of pixel/class operands.
        Ok((tag, fact))
    }
    pub(super) fn run_set_count(
        &mut self,
        set: Address,
        guards: &[(Address, Address)],
        input: &Grid,
    ) -> Result<(Address, Address), Error> {
        let parent = self.root.provision(b"native-components");
        self.guard(SetCountWitness::reserve_bytes(
            &set,
            b"reference",
            guards.len(),
            &parent,
        ))?;
        let mut w = SetCountWitness::prepare(
            set,
            b"reference",
            guards,
            &parent,
            &self.set_proofs.to_be_bytes(),
            vec![input.clone()],
        )?;
        self.set_proofs += 1;
        let executed = w.execute(w.budget);
        self.set_copies += w.machine.cycles;
        executed?;
        let p = w.record()?;
        self.guard((128 * 160 + 900 * 24) * 640)?;
        Ok((p.proof.clone(), p.value.clone()))
    }
    pub(super) fn grid_known(&mut self, input: &Grid) -> Result<Address, Error> {
        let cache = self
            .root
            .provision(b"context-grid-known")
            .provision(&input.anchor.identity_key());
        if let Some(slot) = cache.child(b"proof") {
            let p = slot.follow()?;
            if !p.child(b"committed").is_some_and(|n| n.is_terminal()) {
                return Err(Error::Incomplete);
            }
            return Ok(p);
        }
        let mut guards = Vec::new();
        for y in 0..input.height() {
            for x in 0..input.width() {
                guards.push((field(&input.cell(y, x)?, b"value")?, self.unknown.clone()));
            }
        }
        let (p, _) = self.run_set_count(cache.provision(b"empty-set"), &guards, input)?;
        cache.provision(b"proof").point_to(&p);
        Ok(p)
    }
    pub(super) fn context_value(
        &mut self,
        input: &Grid,
        cell: &Address,
        role: &Address,
    ) -> Result<(Address, Address), Error> {
        let colour = field(role, b"context-colour")?.follow()?;
        let parent = self.root.provision(b"native-components");
        let missing = parent
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let (scope, subject, kind) = if let Some(k) = role.child(b"neighbour-kind") {
            let kind = k.follow()?;
            let (proof, _) = self.component_value(input, cell)?;
            (
                self.root
                    .provision(b"local-component-contexts")
                    .provision(&input.anchor.identity_key())
                    .provision(&proof.identity_key())
                    .provision(&colour.identity_key())
                    .provision(&kind.identity_key()),
                Some(proof),
                Some(kind),
            )
        } else {
            (
                self.root
                    .provision(b"global-component-contexts")
                    .provision(&input.anchor.identity_key())
                    .provision(&colour.identity_key()),
                None,
                None,
            )
        };
        if let Some(slot) = scope.child(b"proof") {
            let p = slot.follow()?;
            if !p.child(b"committed").is_some_and(|n| n.is_terminal())
                || !field(&p, b"context-input")?.follow()?.same(&input.anchor)
            {
                return Err(Error::Incomplete);
            }
            return Ok((p.clone(), field(&p, b"value")?.follow()?));
        }
        scope.provision(b"colour-at-bind").point_to(&colour);
        scope.provision(b"input").point_to(&input.anchor);
        let set = scope.provision(b"references");
        let mut guards = BTreeMap::new();
        let mut inspected = 0usize;
        if let Some(subject) = subject {
            let (complete, value) = self.complete_count_value(input, cell)?;
            let unavailable = self
                .root
                .provision(b"complete-count-tags")
                .provision(b"unavailable");
            guards.insert(
                complete.identity_key(),
                (field(&complete, b"value")?, unavailable.clone()),
            );
            scope.provision(b"subject").point_to(&subject);
            scope.provision(b"subject-completeness").point_to(&complete);
            let wanted = kind.ok_or(Error::Incomplete)?;
            scope.provision(b"kind-at-bind").point_to(&wanted);
            if !self.context_equal(&value, &unavailable)? {
                let member = parent.provision(b"class-tags").provision(b"member");
                let unknown_kind = parent.provision(b"relation-tags").provision(b"unresolved");
                for view in field(&subject, b"views")?.inspect_children() {
                    let class = field(&view, b"class")?.follow()?;
                    if !self.context_equal(&class, &member)? {
                        continue;
                    }
                    let source = field(&view, b"source")?.follow()?;
                    for port in PORTS {
                        if inspected.is_multiple_of(128) {
                            self.guard((128 * 160 + 900 * 24) * 640)?;
                        }
                        inspected += 1;
                        let neighbour = field(&source, port)?.follow()?;
                        let Some(slot) = neighbour.child(b"value") else {
                            continue;
                        };
                        let value = slot.follow()?;
                        if !self.context_equal(&value, &colour)? {
                            continue;
                        }
                        let (proof, kind) = self.component_value(input, &neighbour)?;
                        guards.insert(
                            proof.identity_key(),
                            (field(&proof, b"result")?, unknown_kind.clone()),
                        );
                        if self.context_equal(&kind, &wanted)? {
                            records::record(&set, b"reference", &proof, &view)?;
                        }
                    }
                }
            }
        } else {
            let known = self.grid_known(input)?;
            guards.insert(
                known.identity_key(),
                (field(&known, b"value")?, missing.clone()),
            );
            scope.provision(b"grid-completeness").point_to(&known);
            for y in 0..input.height() {
                for x in 0..input.width() {
                    if inspected.is_multiple_of(128) {
                        self.guard((128 * 160 + 900 * 24) * 640)?;
                    }
                    inspected += 1;
                    let cell = input.cell(y, x)?;
                    let value = field(&cell, b"value")?.follow()?;
                    if self.context_equal(&value, &colour)? {
                        let (proof, _) = self.component_value(input, &cell)?;
                        records::record(&set, b"reference", &proof, &cell)?;
                    }
                }
            }
        }
        let guards: Vec<_> = guards.into_values().collect();
        let (p, value) = self.run_set_count(set, &guards, input)?;
        p.provision(b"context-input").point_to(&input.anchor);
        p.provision(b"context-role-at-bind").point_to(role);
        p.provision(b"context-scope").point_to(&scope);
        scope.provision(b"proof").point_to(&p);
        Ok((p, value))
    }
    pub(super) fn predicate_context(
        &mut self,
        input: &Grid,
        cell: &Address,
        group: &Address,
        at: &Address,
    ) -> Result<Address, Error> {
        let all = at.provision(b"predicates");
        if group.child(b"expected-count").is_some() {
            let base_at = at.provision(b"base");
            let base = self.predicate_count(input, cell, group, &base_at)?;
            records::record(&all, b"predicate", &base, &base_at)?;
            if !base.same(&self.yes) {
                return self.tag(&all);
            }
        }
        let role = field(group, b"context-role")?.follow()?;
        let (producer, value) = self.context_value(input, cell, &role)?;
        let missing = self
            .root
            .provision(b"native-components")
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let (known, fact) = self.property_knownness(&producer, &value, &missing)?;
        let expected = field(group, b"expected-context-count")?.follow()?;
        let yes = self.yes.clone();
        let no = self.no.clone();
        let (tag, proof) = self.compare(&value, &expected, &yes, &no)?;
        proof.provision(b"context-producer").point_to(&producer);
        proof.provision(b"context-role-consumed").point_to(&role);
        proof
            .provision(b"expected-context-count-consumed")
            .point_to(&expected);
        proof.provision(b"context-count-consumed").point_to(&value);
        records::record(&all, b"predicate", &known, &fact)?;
        records::record(&all, b"predicate", &tag, &proof)?;
        self.tag(&all)
    }
}
