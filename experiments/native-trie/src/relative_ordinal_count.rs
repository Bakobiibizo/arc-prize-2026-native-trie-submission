//! Native relative reference-set count. Selection is the actual completed native
//! strict-reachability tag, not host ordering/counting of ordinal payloads.
use crate::{
    Address,
    arc_grids::GridError,
    ordinal_relation::{OrdinalBank, OrdinalInput, OrdinalProjection},
    records,
    reference_set_count::{SetCountProjection, SetCountWitness},
    region_witness::RegionError,
};
use std::{collections::BTreeMap, rc::Rc};
fn field(a: &Address, k: &[u8]) -> Result<Address, RegionError> {
    a.child(k).ok_or(GridError::Malformed.into())
}
#[derive(Debug)]
pub enum RelativeOrdinalError {
    Region(RegionError),
    Record(records::RecordError),
}
impl From<RegionError> for RelativeOrdinalError {
    fn from(e: RegionError) -> Self {
        Self::Region(e)
    }
}
impl From<records::RecordError> for RelativeOrdinalError {
    fn from(e: records::RecordError) -> Self {
        Self::Record(e)
    }
}
pub struct OrdinalCandidate {
    pub reference: Address,
    pub count: Rc<OrdinalInput>,
}
pub struct RelativeOrdinalCount {
    pub root: Address,
    parent: Address,
    zero: Address,
    banks: BTreeMap<Vec<u8>, OrdinalBank>,
    serial: u64,
    pub comparison_copies: u64,
    pub count_copies: u64,
}
pub struct RelativeOrdinalProjection {
    count: SetCountProjection,
    pub evidence: Address,
    _subject: Rc<OrdinalInput>,
    _comparisons: Vec<OrdinalProjection>,
    _owner: Address,
}
impl RelativeOrdinalProjection {
    pub fn count(&self) -> &SetCountProjection {
        &self.count
    }
}
impl RelativeOrdinalCount {
    pub fn new(parent: &Address, zero: &Address) -> Self {
        Self {
            root: parent.provision(b"relative-ordinal-relations"),
            parent: parent.clone(),
            zero: zero.clone(),
            banks: BTreeMap::new(),
            serial: 0,
            comparison_copies: 0,
            count_copies: 0,
        }
    }
    pub fn bank_count(&self) -> usize {
        self.banks.len()
    }
    pub fn prepare(
        &mut self,
        subject: Rc<OrdinalInput>,
        candidates: &[OrdinalCandidate],
        universe_guards: &[(Address, Address)],
        id: &[u8],
        mut reserve: impl FnMut(usize) -> Result<(), RegionError>,
    ) -> Result<RelativeOrdinalProjection, RelativeOrdinalError> {
        if !self
            .parent
            .child(b"cardinalities")
            .and_then(|n| n.child(b"zero"))
            .is_some_and(|n| n.same(&self.zero))
        {
            return Err(RegionError::from(GridError::ForeignSpace).into());
        }
        subject.current()?;
        reserve(128 * 640)?;
        let proofs = self.root.provision(b"proofs");
        if proofs.child(id).is_some() {
            return Err(RegionError::Duplicate.into());
        }
        let evidence = proofs.provision(id);
        evidence
            .provision(b"subject-count-producer")
            .point_to(&subject.proof);
        let set = evidence.provision(b"selected-references");
        let mut guards = universe_guards.to_vec();
        guards.push((
            field(&subject.proof, b"value")?,
            subject.unavailable.clone(),
        ));
        let mut comparisons = Vec::new();
        let mut counts = BTreeMap::new();
        for (i, candidate) in candidates.iter().enumerate() {
            if i.is_multiple_of(128) {
                reserve((128 * 160 + 900 * 24) * 640)?;
            }
            let value = candidate.count.current()?;
            let key = value.identity_key().to_vec();
            let occurrence = evidence
                .provision(b"occurrences")
                .provision(&i.to_be_bytes());
            occurrence
                .provision(b"reference")
                .point_to(&candidate.reference);
            occurrence
                .provision(b"count-producer")
                .point_to(&candidate.count.proof);
            occurrence.provision(b"count-at-bind").point_to(&value);
            if let Some(old) = counts.insert(candidate.reference.identity_key(), value.clone())
                && !old.same(&value)
            {
                evidence
                    .provision(b"contradictory-reference-count-binding")
                    .point_to(&occurrence);
                return Err(RegionError::Incomplete.into());
            }
            if !self.banks.contains_key(&key) {
                reserve(OrdinalBank::reserve_bytes())?;
                let b = OrdinalBank::prepare(
                    &self.zero,
                    candidate.count.clone(),
                    &self.parent,
                    &self.serial.to_be_bytes(),
                )?;
                self.serial += 1;
                self.banks.insert(key.clone(), b);
            }
            reserve(OrdinalBank::invocation_reserve_bytes())?;
            let bank = self.banks.get_mut(&key).ok_or(RegionError::Incomplete)?;
            let yes = bank.greater_tag();
            let unknown = bank.unavailable_tag();
            let mut witness = bank.compare(
                subject.clone(),
                candidate.count.clone(),
                &self.serial.to_be_bytes(),
            )?;
            self.serial += 1;
            let executed = witness.execute(witness.budget);
            self.comparison_copies += witness.machine.cycles;
            executed?;
            let projection = witness.record()?;
            occurrence
                .provision(b"native-comparison")
                .point_to(&projection.proof);
            guards.push((field(&projection.proof, b"value")?, unknown));
            if projection.value.same(&yes) {
                records::record(&set, b"reference", &candidate.reference, &projection.proof)?;
            }
            comparisons.push(projection);
        }
        reserve(SetCountWitness::reserve_bytes(
            &set,
            b"reference",
            guards.len(),
            &self.parent,
        ))?;
        let mut count_id = b"relative-ordinal:".to_vec();
        count_id.extend_from_slice(&self.serial.to_be_bytes());
        let mut witness = SetCountWitness::prepare(
            set,
            b"reference",
            &guards,
            &self.parent,
            &count_id,
            Vec::new(),
        )?;
        self.serial += 1;
        let executed = witness.execute(witness.budget);
        self.count_copies += witness.machine.cycles;
        executed?;
        let count = witness.record()?;
        count
            .proof
            .provision(b"relative-count-evidence")
            .point_to(&evidence);
        evidence.provision(b"count-proof").point_to(&count.proof);
        evidence.provision(b"committed").terminate();
        Ok(RelativeOrdinalProjection {
            count,
            evidence,
            _subject: subject,
            _comparisons: comparisons,
            _owner: self.parent.clone(),
        })
    }
}
