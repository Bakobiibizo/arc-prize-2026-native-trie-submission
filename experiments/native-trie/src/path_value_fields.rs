//! Bound actual input-value reference sets along stored topology; membership
//! truth is emitted by SetMembershipWitness, never host equality/set lookup.
use super::*;
use crate::reference_set_membership::SetMembershipWitness;
use std::collections::BTreeSet;
impl SparseRewrite {
    // Escrow ALL new-family allocations, not repeated O(model) scans per property.
    // Each observation covers <=2 native160-node comparison frames plus192
    // metadata/emission nodes; field constructors have separate precharges.
    // Nested field observations also cover first-producer knownness frames.
    pub(super) fn path_guard(&self, reserve: usize) -> Result<(), Error> {
        let mut upper = self.path_upper.get();
        if upper == 0 || upper.saturating_add(reserve) > 286_720_000 {
            let actual = self.workspace_bytes();
            if upper != 0 && actual > upper {
                return Err(Error::WorkspaceLimit);
            }
            upper = actual;
        }
        if upper.saturating_add(reserve) > 286_720_000 {
            return Err(Error::WorkspaceLimit);
        }
        self.path_upper.set(upper + reserve);
        Ok(())
    }
    pub(super) fn path_observe(&mut self) -> Result<(), Error> {
        if self.path_observations.is_multiple_of(128) {
            let actual = self.workspace_bytes();
            if self.path_upper.get() != 0 && actual > self.path_upper.get() {
                return Err(Error::WorkspaceLimit);
            }
            if actual.saturating_add((128 * 160 + 900 * 24) * 640) > 286_720_000 {
                return Err(Error::WorkspaceLimit);
            }
            self.path_upper.set(actual);
        }
        self.path_guard(512 * 640)?;
        self.path_observations += 1;
        Ok(())
    }
    pub(super) fn copy_action_slot(
        &self,
        cell: &Address,
        rule: &Address,
    ) -> Result<Option<Address>, Error> {
        let port = field(rule, b"action-port")?.follow()?;
        let token = port.token().ok_or(Error::Incomplete)?;
        if !PORTS.contains(&token) {
            return Err(Error::Incomplete);
        }
        Ok(field(cell, token)?.follow()?.child(b"value"))
    }
    pub(super) fn path_membership(
        &mut self,
        input: &Grid,
        cell: &Address,
        rule: &Address,
        action: &Address,
    ) -> Result<(Address, Address), Error> {
        self.path_observe()?;
        let expected = if rule.child(b"expected-action-source").is_some() {
            action.clone()
        } else {
            field(cell, b"value")?
        };
        let expected_value = expected.follow()?;
        let new_known = self
            .root
            .child(b"context-grid-known")
            .and_then(|r| r.child(&input.anchor.identity_key()))
            .and_then(|r| r.child(b"proof"))
            .is_none();
        let known = self.grid_known(input)?;
        // This existing constructor performs its own exact preallocation guards.
        if new_known {
            self.path_upper.set(0);
        }
        let known_value = field(&known, b"value")?.follow()?;
        let mut path = Vec::new();
        for step in field(rule, b"step-word")?.inspect_children() {
            let p = step.follow()?;
            let token = p.token().ok_or(Error::Incomplete)?;
            if !PORTS.contains(&token) {
                return Err(Error::Incomplete);
            }
            path.push(p);
        }
        if path.is_empty() || path.len() > 2 {
            return Err(Error::Incomplete);
        }
        let mut scope = self
            .root
            .provision(b"path-value-index")
            .provision(&input.anchor.identity_key())
            .provision(&cell.identity_key())
            .provision(&expected.identity_key())
            .provision(&expected_value.identity_key())
            .provision(&known.identity_key())
            .provision(&known_value.identity_key());
        for p in &path {
            scope = scope.provision(p.token().ok_or(Error::Incomplete)?);
        }
        let key = scope.identity_key().to_vec();
        if let Some(p) = self.path_values.get(&key) {
            if !p.proof.child(b"committed").is_some_and(|n| n.is_terminal()) {
                return Err(Error::Incomplete);
            }
            return Ok((p.proof.clone(), field(&p.proof, b"value")?.follow()?));
        }
        // At most30 strict steps for the supplied one-/two-port monotone words;
        // the extra width+height margin also protects invalid-word detection metadata.
        self.path_guard((64 * (input.width() + input.height() + 1) + 128) * 640)?;
        scope.provision(b"input").point_to(&input.anchor);
        scope.provision(b"origin").point_to(cell);
        scope.provision(b"expected-slot").point_to(&expected);
        let set = scope.provision(b"reference-set");
        let mut at = cell.clone();
        let mut seen = BTreeSet::new();
        seen.insert(at.identity_key());
        let mut index = 0usize;
        'ray: loop {
            let step = scope.provision(b"steps").provision(&index.to_be_bytes());
            for (i, port) in path.iter().enumerate() {
                let source = field(&at, port.token().ok_or(Error::Incomplete)?)?;
                let next = source.follow()?;
                let edge = step.provision(b"ports").provision(&i.to_be_bytes());
                edge.provision(b"source-slot").point_to(&source);
                edge.provision(b"reached").point_to(&next);
                if next.child(b"value").is_none() {
                    step.provision(b"outside").point_to(&next);
                    break 'ray;
                }
                at = next;
            }
            if !seen.insert(at.identity_key()) {
                scope.provision(b"cyclic-path").point_to(&at);
                return Err(Error::Incomplete);
            }
            if index >= 900 {
                scope.provision(b"path-domain-exceeded").terminate();
                return Err(Error::Incomplete);
            }
            let value = field(&at, b"value")?.follow()?;
            step.provision(b"source").point_to(&at);
            step.provision(b"value-at-bind").point_to(&value);
            records::record(&set, b"reference", &value, &step)?;
            index += 1;
        }
        let parent = self.root.provision(b"native-path-memberships");
        let missing = self
            .root
            .provision(b"native-components")
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let guards = vec![
            (field(&known, b"value")?, missing),
            (expected.clone(), self.unknown.clone()),
        ];
        self.path_guard(SetMembershipWitness::reserve_bytes(
            &set,
            b"reference",
            guards.len(),
        ))?;
        let mut w = SetMembershipWitness::prepare(
            set,
            b"reference",
            &expected,
            &guards,
            &parent,
            &self.membership_proofs.to_be_bytes(),
            vec![input.clone()],
        )?;
        self.membership_proofs += 1;
        let result = w.execute(w.budget);
        self.membership_copies += w.machine.cycles;
        result?;
        let p = w.record()?.retain_owners(self.owned_roots().to_vec());
        p.proof.provision(b"path-scope").point_to(&scope);
        p.proof.provision(b"known-input-proof").point_to(&known);
        scope.provision(b"membership-proof").point_to(&p.proof);
        let output = (p.proof.clone(), p.value.clone());
        self.path_values.insert(key, p);
        Ok(output)
    }
    pub(super) fn path_predicate(
        &mut self,
        input: &Grid,
        cell: &Address,
        rule: &Address,
        action: &Address,
        at: &Address,
    ) -> Result<Address, Error> {
        if rule.child(b"always").is_some() {
            let yes = self.yes.clone();
            let no = self.no.clone();
            let (tag, p) = self.compare(&yes, &yes, &yes, &no)?;
            at.provision(b"native-true").point_to(&p);
            return Ok(tag);
        }
        let (producer, value) = self.path_membership(input, cell, rule, action)?;
        let missing = self
            .root
            .provision(b"native-path-memberships")
            .provision(b"set-membership-tags")
            .provision(b"unavailable");
        let (known, fact) = self.property_knownness(&producer, &value, &missing)?;
        let expected = field(rule, b"expected-membership")?.follow()?;
        let yes = self.yes.clone();
        let no = self.no.clone();
        let (tag, p) = self.compare(&value, &expected, &yes, &no)?;
        p.provision(b"membership-producer").point_to(&producer);
        p.provision(b"membership-consumed").point_to(&value);
        p.provision(b"expected-membership-consumed")
            .point_to(&expected);
        let all = at.provision(b"predicates");
        records::record(&all, b"predicate", &known, &fact)?;
        records::record(&all, b"predicate", &tag, &p)?;
        self.tag(&all)
    }
}
