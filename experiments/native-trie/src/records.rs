//! Generic physical reference storage at supplied addresses. Not semantic inference or an opcode.
//! All indices below are actual trie child adjacency; identity tokens are process-local pointers.
use crate::{Address, Fault};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
    Reference(Fault),
    Malformed,
    IdentityMismatch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    NewTarget,
    AddedEvidence,
    AlreadyRecorded,
}
pub struct Recorded {
    pub candidate: Address,
    pub evidence: Address,
    pub change: Change,
}
pub enum Selection {
    Unknown,
    Unique {
        candidate: Address,
        target: Address,
    },
    /// Different reference identities, not a claim of semantic inequivalence.
    Ambiguous {
        alternatives: usize,
    },
}

/// Only native handles into prepared trie structure, not a host copy of the evidence/model.
/// Dropping this leaves the public candidate/evidence sets unchanged.
pub struct Prepared {
    index: Address,
    candidate: Address,
    evidence: Address,
    target: Address,
    source: Address,
    anchor: Address,
    slot: Address,
}
fn link(at: &Address) -> Result<Address, RecordError> {
    at.follow().map_err(RecordError::Reference)
}
fn field(at: &Address, key: &[u8]) -> Result<Address, RecordError> {
    at.child(key).ok_or(RecordError::Malformed)
}
fn validate_candidate(
    candidate: &Address,
    target: &Address,
    anchor: &Address,
    slot: &Address,
) -> Result<(), RecordError> {
    if !link(candidate)?.same(target) {
        return Err(RecordError::IdentityMismatch);
    }
    if !link(&field(candidate, b"anchor")?)?.same(anchor)
        || !link(&field(candidate, b"slot")?)?.same(slot)
    {
        return Err(RecordError::Malformed);
    }
    Ok(())
}
fn validate_evidence(
    evidence: &Address,
    source: &Address,
    target: &Address,
    anchor: &Address,
) -> Result<(), RecordError> {
    if !link(evidence)?.same(source) {
        return Err(RecordError::IdentityMismatch);
    }
    if !link(&field(evidence, b"anchor")?)?.same(anchor)
        || !link(&field(evidence, b"target")?)?.same(target)
    {
        return Err(RecordError::Malformed);
    }
    Ok(())
}

/// Staging can allocate. Neither candidates nor evidence become query-visible before publication.
/// Evidence/target owners must be retained by the caller (reference edges are weak).
pub fn prepare(
    anchor: &Address,
    kind: &[u8],
    target: &Address,
    source: &Address,
) -> Result<Prepared, RecordError> {
    let slot = anchor.provision(b"holds").provision(kind);
    let index = slot.provision(b"candidates");
    let target_key = target.identity_key();
    let source_key = source.identity_key();
    let candidate = if let Some(existing) = index.child(&target_key) {
        validate_candidate(&existing, target, anchor, &slot)?;
        existing
    } else {
        let at = Address::prepared_node(&target_key);
        at.point_to(target);
        at.provision(b"anchor").point_to(anchor);
        at.provision(b"slot").point_to(&slot);
        at.provision(b"evidence");
        at
    };
    let evidence_root = field(&candidate, b"evidence")?;
    let evidence = if let Some(existing) = evidence_root.child(&source_key) {
        validate_evidence(&existing, source, target, anchor)?;
        existing
    } else {
        let at = Address::prepared_node(&source_key);
        at.point_to(source);
        at.provision(b"anchor").point_to(anchor);
        at.provision(b"target").point_to(target);
        at
    };
    // A new candidate is still unpublished. Give it complete initial evidence before adoption.
    if index.child(&target_key).is_none() {
        evidence_root.publish_prepared_child(&evidence);
    }
    Ok(Prepared {
        index,
        candidate,
        evidence,
        target: target.clone(),
        source: source.clone(),
        anchor: anchor.clone(),
        slot,
    })
}
impl Prepared {
    /// One child-edge insertion publishes a complete new candidate or a complete new evidence record.
    /// Interleaved preparations are reconciled against the current actual index, not stale host state.
    pub fn publish(self) -> Result<Recorded, RecordError> {
        validate_candidate(&self.candidate, &self.target, &self.anchor, &self.slot)?;
        validate_evidence(&self.evidence, &self.source, &self.target, &self.anchor)?;
        let existing = self.index.child(&self.target.identity_key());
        let new_target = existing.is_none();
        let candidate = if let Some(existing) = existing {
            validate_candidate(&existing, &self.target, &self.anchor, &self.slot)?;
            existing
        } else {
            self.index.publish_prepared_child(&self.candidate)
        };
        let evidence_root = field(&candidate, b"evidence")?;
        let old = evidence_root.child(&self.source.identity_key());
        let (evidence, change) = if let Some(old) = old {
            validate_evidence(&old, &self.source, &self.target, &self.anchor)?;
            (
                old,
                if new_target {
                    Change::NewTarget
                } else {
                    Change::AlreadyRecorded
                },
            )
        } else {
            (
                evidence_root.publish_prepared_child(&self.evidence),
                Change::AddedEvidence,
            )
        };
        Ok(Recorded {
            candidate,
            evidence,
            change,
        })
    }
}
pub fn record(
    anchor: &Address,
    kind: &[u8],
    target: &Address,
    source: &Address,
) -> Result<Recorded, RecordError> {
    prepare(anchor, kind, target, source)?.publish()
}

/// Read only the actual candidate container's cardinality/single entry, never scan alternatives.
pub fn resolve(anchor: &Address, kind: &[u8]) -> Result<Selection, RecordError> {
    let Some(slot) = anchor.child(b"holds").and_then(|at| at.child(kind)) else {
        return Ok(Selection::Unknown);
    };
    let index = field(&slot, b"candidates")?;
    match index.sole_child() {
        Ok(None) => Ok(Selection::Unknown),
        Ok(Some(candidate)) => {
            let target = link(&candidate)?;
            validate_candidate(&candidate, &target, anchor, &slot)?;
            Ok(Selection::Unique { candidate, target })
        }
        Err(alternatives) => Ok(Selection::Ambiguous { alternatives }),
    }
}
/// Reference-set agreement for an explicit constraint grammar, not semantic equivalence.
/// Empty/unrecorded containers impose no constraint. Known disjoint sets return
/// Ambiguous{alternatives:0}, NOT Unknown. Does not change strict `resolve`.
/// Actual child-key lookup only; no allocation or dereference of target payloads.
pub fn intersect(anchors: &[Option<&Address>], kind: &[u8]) -> Result<Selection, RecordError> {
    fn index(anchor: &Address, kind: &[u8]) -> Result<Option<(Address, Address)>, RecordError> {
        let Some(slot) = anchor.child(b"holds").and_then(|n| n.child(kind)) else {
            return Ok(None);
        };
        let at = field(&slot, b"candidates")?;
        Ok(Some((slot, at)))
    }
    let mut first = None;
    for anchor in anchors.iter().flatten() {
        if let Some((slot, at)) = index(anchor, kind)? {
            let nonempty = !at.0.children.borrow().is_empty();
            if nonempty {
                first = Some((*anchor, slot, at));
                break;
            }
        }
    }
    let Some((anchor, slot, at)) = first else {
        return Ok(Selection::Unknown);
    };
    let entries = at.0.children.borrow();
    let mut count = 0;
    let mut selected = None;
    for candidate in entries.values() {
        let target = link(candidate)?;
        validate_candidate(candidate, &target, anchor, &slot)?;
        let mut compatible = true;
        for other in anchors.iter().flatten() {
            if let Some((other_slot, other_index)) = index(other, kind)? {
                let map = other_index.0.children.borrow();
                if map.is_empty() {
                    continue;
                }
                match map.get(&target.identity_key()[..]) {
                    Some(c) => validate_candidate(c, &target, other, &other_slot)?,
                    None => {
                        compatible = false;
                        break;
                    }
                }
            }
        }
        if compatible {
            count += 1;
            if selected.is_none() {
                selected = Some((candidate.clone(), target));
            }
        }
    }
    if count == 1 {
        let (candidate, target) = selected.unwrap();
        Ok(Selection::Unique { candidate, target })
    } else {
        Ok(Selection::Ambiguous {
            alternatives: count,
        })
    }
}
/// Retrieve every physically recorded candidate. No applicability filter, semantic comparison,
/// or winner selection occurs here.
pub fn retrieve_all(anchor: &Address, kind: &[u8]) -> Result<Vec<(Address, Address)>, RecordError> {
    let Some(slot) = anchor.child(b"holds").and_then(|at| at.child(kind)) else {
        return Ok(Vec::new());
    };
    let index = field(&slot, b"candidates")?;
    index
        .inspect_children()
        .into_iter()
        .map(|candidate| {
            let target = link(&candidate)?;
            validate_candidate(&candidate, &target, anchor, &slot)?;
            Ok((candidate, target))
        })
        .collect()
}

/// Allocating diagnostic export only; never used to select or consolidate an answer.
pub fn inspect_candidates(anchor: &Address, kind: &[u8]) -> Vec<Address> {
    anchor
        .child(b"holds")
        .and_then(|at| at.child(kind))
        .and_then(|at| at.child(b"candidates"))
        .map_or_else(Vec::new, |at| at.inspect_children())
}
/// Allocating diagnostic export of evidence-entry addresses, not reconstructed payloads.
pub fn inspect_evidence(candidate: &Address) -> Result<Vec<Address>, RecordError> {
    Ok(field(candidate, b"evidence")?.inspect_children())
}

#[cfg(test)]
mod ownership {
    use super::*;
    use crate::Trie;
    use std::rc::Rc;
    #[test]
    fn published_backlinks_do_not_keep_the_owner_alive() {
        let weak = {
            let data = Trie::default();
            let anchor = data.root();
            let source = anchor.provision(b"source");
            record(&anchor, b"self", &anchor, &source).unwrap();
            Rc::downgrade(&anchor.0)
        };
        assert!(weak.upgrade().is_none());
    }
}
