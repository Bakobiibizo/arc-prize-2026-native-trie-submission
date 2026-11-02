//! Canonical four-axis event episodes and structurally addressed construction DAGs.
//!
//! This is the first substrate representation probe for ARC construction learning. Admission is
//! setup work; lookup follows exact native edges and never scans construction alternatives.
use crate::{
    Address, Fault, Trie,
    records::{self, RecordError, Selection},
};

#[derive(Clone)]
pub struct AxisEvent {
    pub position: Address,
    pub neighbour_position: Address,
    pub color: Address,
    pub neighbour_color: Address,
    /// Concrete occurrence bindings retained as episode evidence, not address tokens.
    pub focal: Address,
    pub neighbour: Address,
}

#[derive(Clone)]
pub struct AxisConstructions {
    pub position: Address,
    pub neighbour_position: Address,
    pub color: Address,
    pub neighbour_color: Address,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpisodeError {
    Empty,
    Corrupt,
    Reference(Fault),
    Records(RecordError),
    Missing,
    Ambiguous,
}
impl From<Fault> for EpisodeError {
    fn from(value: Fault) -> Self {
        Self::Reference(value)
    }
}
impl From<RecordError> for EpisodeError {
    fn from(value: RecordError) -> Self {
        Self::Records(value)
    }
}

pub struct ResolvedEpisode {
    pub endpoint: Address,
    pub constructions: AxisConstructions,
    pub implementation: Option<Address>,
}

#[derive(Default)]
pub struct AxisEpisodes {
    storage: Trie,
}

impl AxisEpisodes {
    pub fn root(&self) -> Address {
        self.storage.root()
    }

    fn keyed(parent: &Address, axis: &[u8], value: &Address) -> Result<Address, EpisodeError> {
        let axis = parent.provision(axis);
        let key = value.identity_key();
        if let Some(node) = axis.child(&key) {
            let stored = node
                .child(b"trait")
                .ok_or(EpisodeError::Corrupt)?
                .follow()?;
            if !stored.same(value) {
                return Err(EpisodeError::Corrupt);
            }
            return Ok(node);
        }
        let node = axis.provision(&key);
        node.provision(b"trait").point_to(value);
        Ok(node)
    }

    fn ordered(events: &[AxisEvent]) -> Vec<&AxisEvent> {
        let mut ordered = events.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|event| {
            (
                event.position.identity_key(),
                event.neighbour_position.identity_key(),
                event.color.identity_key(),
                event.neighbour_color.identity_key(),
            )
        });
        ordered
    }

    fn path(&self, events: &[AxisEvent], create: bool) -> Result<Address, EpisodeError> {
        if events.is_empty() {
            return Err(EpisodeError::Empty);
        }
        let mut at = if create {
            self.root().provision(b"axis-context-episodes")
        } else {
            self.root()
                .child(b"axis-context-episodes")
                .ok_or(EpisodeError::Missing)?
        };
        for event in Self::ordered(events) {
            for (axis, value) in [
                (b"position".as_slice(), &event.position),
                (b"neighbour-position".as_slice(), &event.neighbour_position),
                (b"color".as_slice(), &event.color),
                (b"neighbour-color".as_slice(), &event.neighbour_color),
            ] {
                if create {
                    at = Self::keyed(&at, axis, value)?;
                } else {
                    let axis = at.child(axis).ok_or(EpisodeError::Missing)?;
                    at = axis
                        .child(&value.identity_key())
                        .ok_or(EpisodeError::Missing)?;
                    let stored = at.child(b"trait").ok_or(EpisodeError::Corrupt)?.follow()?;
                    if !stored.same(value) {
                        return Err(EpisodeError::Corrupt);
                    }
                }
            }
        }
        Ok(at)
    }

    pub fn teach(
        &mut self,
        events: &[AxisEvent],
        constructions: &AxisConstructions,
        evidence: &Address,
    ) -> Result<Address, EpisodeError> {
        let endpoint = self.path(events, true)?;
        for (view, construction) in [
            (b"position-construction".as_slice(), &constructions.position),
            (
                b"neighbour-position-construction".as_slice(),
                &constructions.neighbour_position,
            ),
            (b"color-construction".as_slice(), &constructions.color),
            (
                b"neighbour-color-construction".as_slice(),
                &constructions.neighbour_color,
            ),
        ] {
            records::record(&endpoint, view, construction, evidence)?;
        }
        let episode = endpoint
            .provision(b"occurrences")
            .provision(&evidence.identity_key());
        episode.provision(b"evidence").point_to(evidence);
        for (index, event) in Self::ordered(events).into_iter().enumerate() {
            let occurrence = episode.provision(&index.to_be_bytes());
            occurrence.provision(b"focal").point_to(&event.focal);
            occurrence
                .provision(b"neighbour")
                .point_to(&event.neighbour);
        }
        Ok(endpoint)
    }

    fn unique(endpoint: &Address, view: &[u8]) -> Result<Address, EpisodeError> {
        match records::resolve(endpoint, view)? {
            Selection::Unique { target, .. } => Ok(target),
            Selection::Unknown => Err(EpisodeError::Missing),
            Selection::Ambiguous { .. } => Err(EpisodeError::Ambiguous),
        }
    }

    pub fn record_implementation(
        &mut self,
        endpoint: &Address,
        implementation: &Address,
        evidence: &Address,
    ) -> Result<(), EpisodeError> {
        records::record(
            endpoint,
            b"episode-implementation",
            implementation,
            evidence,
        )?;
        Ok(())
    }

    pub fn implementations(
        &self,
        events: &[AxisEvent],
    ) -> Result<(Address, Vec<Address>), EpisodeError> {
        let endpoint = self.path(events, false)?;
        let implementations = records::retrieve_all(&endpoint, b"episode-implementation")?
            .into_iter()
            .map(|(_, target)| target)
            .collect();
        Ok((endpoint, implementations))
    }

    pub fn resolve(&self, events: &[AxisEvent]) -> Result<ResolvedEpisode, EpisodeError> {
        let endpoint = self.path(events, false)?;
        let implementation = match records::resolve(&endpoint, b"episode-implementation")? {
            Selection::Unique { target, .. } => Some(target),
            Selection::Unknown => None,
            Selection::Ambiguous { .. } => return Err(EpisodeError::Ambiguous),
        };
        Ok(ResolvedEpisode {
            constructions: AxisConstructions {
                position: Self::unique(&endpoint, b"position-construction")?,
                neighbour_position: Self::unique(&endpoint, b"neighbour-position-construction")?,
                color: Self::unique(&endpoint, b"color-construction")?,
                neighbour_color: Self::unique(&endpoint, b"neighbour-color-construction")?,
            },
            endpoint,
            implementation,
        })
    }
}

#[derive(Clone)]
pub struct DagInput {
    pub role: Address,
    pub construction: Address,
}

#[derive(Default)]
pub struct ConstructionDags {
    storage: Trie,
}

impl ConstructionDags {
    pub fn root(&self) -> Address {
        self.storage.root()
    }

    /// Complete one DAG node bottom-up. Inputs must already be canonical construction addresses.
    /// Role order is canonicalized; conflicting duplicate roles remain distinct recorded bodies.
    pub fn intern(
        &mut self,
        operation: &Address,
        inputs: &[DagInput],
        implementation: &Address,
        evidence: &Address,
    ) -> Result<Address, EpisodeError> {
        let mut ordered = inputs.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|input| input.role.identity_key());
        let mut at = self
            .root()
            .provision(b"construction-dags")
            .provision(&operation.identity_key());
        at.provision(b"operation").point_to(operation);
        for input in ordered {
            at = at
                .provision(&input.role.identity_key())
                .provision(&input.construction.identity_key());
            at.provision(b"role").point_to(&input.role);
            at.provision(b"dependency").point_to(&input.construction);
        }
        let endpoint = at.provision(b"complete");
        records::record(&endpoint, b"implementation", implementation, evidence)?;
        Ok(endpoint)
    }

    pub fn implementation(&self, dag: &Address) -> Result<Address, EpisodeError> {
        match records::resolve(dag, b"implementation")? {
            Selection::Unique { target, .. } => Ok(target),
            Selection::Unknown => Err(EpisodeError::Missing),
            Selection::Ambiguous { .. } => Err(EpisodeError::Ambiguous),
        }
    }
}
