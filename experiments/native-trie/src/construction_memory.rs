//! Direct construction routing by intrinsic native addresses.
//!
//! A route is produced from input facts only. Learning records a construction at the reached
//! address; query resolution follows that same address exactly. There is no construction-family
//! enumeration, candidate scoring, or fallback to a less-specific host-selected solver.
use crate::{
    Address, Trie,
    records::{self, RecordError, Selection},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteError {
    Empty,
    Missing,
    Ambiguous,
    Record(RecordError),
}

impl From<RecordError> for RouteError {
    fn from(value: RecordError) -> Self {
        Self::Record(value)
    }
}

/// Trie-resident index from an ordered intrinsic-fact address to evidenced constructions.
#[derive(Default)]
pub struct ConstructionMemory {
    storage: Trie,
    // Reference edges are deliberately weak. These handles retain facts, constructions, and
    // evidence that have been admitted into this memory without introducing an owner registry
    // consulted during routing.
    retained: Vec<Address>,
}

impl ConstructionMemory {
    pub fn root(&self) -> Address {
        self.storage.root()
    }

    fn route(&self, facts: &[Address], create: bool) -> Result<Address, RouteError> {
        if facts.is_empty() {
            return Err(RouteError::Empty);
        }
        let root = self.storage.root();
        let mut at = if create {
            root.provision(b"intrinsic-routes")
        } else {
            root.child(b"intrinsic-routes").ok_or(RouteError::Missing)?
        };
        for fact in facts {
            let key = fact.identity_key();
            at = if create {
                at.provision(&key)
            } else {
                at.child(&key).ok_or(RouteError::Missing)?
            };
            if create {
                at.provision(b"intrinsic-fact").point_to(fact);
            } else {
                let stored = at
                    .child(b"intrinsic-fact")
                    .ok_or(RouteError::Missing)?
                    .follow()
                    .map_err(|_| RouteError::Missing)?;
                if !stored.same(fact) {
                    return Err(RouteError::Missing);
                }
            }
            at = if create {
                at.provision(b"next")
            } else {
                at.child(b"next").ok_or(RouteError::Missing)?
            };
        }
        Ok(at)
    }

    /// Record one demonstrated construction at an address derived solely from input facts.
    pub fn record(
        &mut self,
        facts: &[Address],
        construction: &Address,
        evidence: &Address,
    ) -> Result<(), RouteError> {
        let at = self.route(facts, true)?;
        records::record(&at, b"construction", construction, evidence)?;
        self.retained.extend(facts.iter().cloned());
        self.retained.push(construction.clone());
        self.retained.push(evidence.clone());
        Ok(())
    }

    /// Follow an exact intrinsic address and return its sole evidenced construction.
    pub fn resolve(&self, facts: &[Address]) -> Result<Address, RouteError> {
        let at = self.route(facts, false)?;
        Self::resolved_at(&at)
    }

    fn resolved_at(at: &Address) -> Result<Address, RouteError> {
        match records::resolve(at, b"construction")? {
            Selection::Unique { target, .. } => Ok(target),
            Selection::Unknown => Err(RouteError::Missing),
            Selection::Ambiguous { .. } => Err(RouteError::Ambiguous),
        }
    }

    /// Follow one input-derived route and retain the most-specific construction encountered on
    /// that path. This is prefix-address traversal, not enumeration of stored constructions.
    pub fn resolve_most_specific(&self, facts: &[Address]) -> Result<Address, RouteError> {
        if facts.is_empty() {
            return Err(RouteError::Empty);
        }
        let mut at = self
            .storage
            .root()
            .child(b"intrinsic-routes")
            .ok_or(RouteError::Missing)?;
        let mut resolved = None;
        for fact in facts {
            let Some(next) = at.child(&fact.identity_key()) else {
                return resolved.ok_or(RouteError::Missing);
            };
            at = next;
            let stored = at
                .child(b"intrinsic-fact")
                .ok_or(RouteError::Missing)?
                .follow()
                .map_err(|_| RouteError::Missing)?;
            if !stored.same(fact) {
                return Err(RouteError::Missing);
            }
            at = at.child(b"next").ok_or(RouteError::Missing)?;
            match Self::resolved_at(&at) {
                Ok(construction) => resolved = Some(construction),
                Err(RouteError::Missing) => {}
                Err(error) => return Err(error),
            }
        }
        resolved.ok_or(RouteError::Missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_facts_route_directly_without_family_order() {
        let facts = Trie::default();
        let kind = facts.provision(&[b"component", b"closed"]);
        let relation = facts.provision(&[b"relation", b"inside"]);
        let programs = Trie::default();
        let construction = programs.provision(&[b"copy-interior"]);
        let evidence = programs.provision(&[b"lesson-0"]);
        let mut memory = ConstructionMemory::default();
        memory
            .record(&[kind.clone(), relation.clone()], &construction, &evidence)
            .unwrap();
        assert!(
            memory
                .resolve(&[kind, relation])
                .unwrap()
                .same(&construction)
        );
    }

    #[test]
    fn unrelated_and_reordered_intrinsics_do_not_fallback_or_scan() {
        let facts = Trie::default();
        let a = facts.provision(&[b"a"]);
        let b = facts.provision(&[b"b"]);
        let c = facts.provision(&[b"c"]);
        let programs = Trie::default();
        let construction = programs.provision(&[b"program"]);
        let evidence = programs.provision(&[b"evidence"]);
        let mut memory = ConstructionMemory::default();
        memory
            .record(&[a.clone(), b.clone()], &construction, &evidence)
            .unwrap();
        assert!(matches!(
            memory.resolve(&[b.clone(), a]),
            Err(RouteError::Missing)
        ));
        assert!(matches!(memory.resolve(&[b, c]), Err(RouteError::Missing)));
    }

    #[test]
    fn one_route_uses_the_most_specific_attached_construction() {
        let facts = Trie::default();
        let structural = facts.provision(&[b"structural"]);
        let valued = facts.provision(&[b"valued"]);
        let programs = Trie::default();
        let general = programs.provision(&[b"general"]);
        let refined = programs.provision(&[b"refined"]);
        let e1 = programs.provision(&[b"general-evidence"]);
        let e2 = programs.provision(&[b"refined-evidence"]);
        let mut memory = ConstructionMemory::default();
        memory
            .record(std::slice::from_ref(&structural), &general, &e1)
            .unwrap();
        memory
            .record(&[structural.clone(), valued.clone()], &refined, &e2)
            .unwrap();
        assert!(
            memory
                .resolve_most_specific(&[structural, valued])
                .unwrap()
                .same(&refined)
        );
    }

    #[test]
    fn recording_order_does_not_select_between_conflicting_constructions() {
        let facts = Trie::default();
        let fact = facts.provision(&[b"same-input"]);
        let programs = Trie::default();
        let a = programs.provision(&[b"a"]);
        let b = programs.provision(&[b"b"]);
        let ea = programs.provision(&[b"ea"]);
        let eb = programs.provision(&[b"eb"]);
        for (first, first_evidence, second, second_evidence) in
            [(&a, &ea, &b, &eb), (&b, &eb, &a, &ea)]
        {
            let mut memory = ConstructionMemory::default();
            memory
                .record(std::slice::from_ref(&fact), first, first_evidence)
                .unwrap();
            memory
                .record(std::slice::from_ref(&fact), second, second_evidence)
                .unwrap();
            assert!(matches!(
                memory.resolve(std::slice::from_ref(&fact)),
                Err(RouteError::Ambiguous)
            ));
        }
    }

    #[test]
    fn conflicting_constructions_remain_ambiguous_at_the_same_address() {
        let facts = Trie::default();
        let fact = facts.provision(&[b"intrinsic"]);
        let programs = Trie::default();
        let first = programs.provision(&[b"first"]);
        let second = programs.provision(&[b"second"]);
        let e1 = programs.provision(&[b"e1"]);
        let e2 = programs.provision(&[b"e2"]);
        let mut memory = ConstructionMemory::default();
        memory
            .record(std::slice::from_ref(&fact), &first, &e1)
            .unwrap();
        memory
            .record(std::slice::from_ref(&fact), &second, &e2)
            .unwrap();
        assert!(matches!(
            memory.resolve(&[fact]),
            Err(RouteError::Ambiguous)
        ));
    }
}
