//! Separate word trie keyed by canonical character addresses, never host strings.
//! Boundaries and character occurrences are supplied; this is not a tokenizer/activation policy.
use crate::{
    Address, Fault, Trie,
    records::{self, RecordError, Recorded, Selection},
};

#[derive(Clone)]
pub struct CharacterEvent {
    pub character: Address,
    pub origin: Address,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordError {
    Empty,
    InspectionLimit,
    Reference(Fault),
    OriginMismatch,
    CorruptPath,
    Records(RecordError),
}
pub enum WordResolution {
    Prefix,
    Complete { word: Address, candidate: Address },
    Ambiguous { alternatives: usize },
}
pub struct Words {
    pub trie: Trie,
}
impl Default for Words {
    fn default() -> Self {
        Self::new()
    }
}
impl Words {
    pub fn new() -> Self {
        Self {
            trie: Trie::default(),
        }
    }
    pub fn root(&self) -> Address {
        self.trie.root()
    }
    // Disjoint from textual metadata keys; only a physical process-local reference identity.
    fn key(character: &Address) -> [u8; 1 + std::mem::size_of::<usize>()] {
        let mut key = [0; 1 + std::mem::size_of::<usize>()];
        key[1..].copy_from_slice(&character.identity_key());
        key
    }
    fn validate(at: &Address, parent: &Address, character: &Address) -> Result<(), WordError> {
        if !at.follow().map_err(WordError::Reference)?.same(character)
            || !at
                .child(b"previous")
                .ok_or(WordError::CorruptPath)?
                .follow()
                .map_err(WordError::Reference)?
                .same(parent)
        {
            return Err(WordError::CorruptPath);
        }
        Ok(())
    }
    fn append(parent: &Address, character: &Address) -> Result<Address, WordError> {
        let key = Self::key(character);
        if let Some(at) = parent.child(&key) {
            Self::validate(&at, parent, character)?;
            return Ok(at);
        }
        let at = Address::prepared_node(&key);
        at.point_to(character);
        at.provision(b"previous").point_to(parent);
        Ok(parent.publish_prepared_child(&at))
    }
    /// Physical read-only child traversal at an existing native prefix. No label extraction.
    pub fn step(
        &self,
        parent: &Address,
        character: &Address,
    ) -> Result<Option<Address>, WordError> {
        let Some(at) = parent.child(&Self::key(character)) else {
            return Ok(None);
        };
        Self::validate(&at, parent, character)?;
        Ok(Some(at))
    }
    pub fn locate(&self, characters: &[Address]) -> Result<Option<Address>, WordError> {
        let mut at = self.root();
        for character in characters {
            match self.step(&at, character)? {
                Some(next) => at = next,
                None => return Ok(None),
            }
        }
        Ok(Some(at))
    }
    /// Supplied nonempty bounded sequence. The endpoint itself is the canonical word identity.
    /// Origin must reference its character identity (e.g. an actual representation candidate).
    pub fn teach(
        &mut self,
        events: &[CharacterEvent],
        boundary: &Address,
    ) -> Result<Recorded, WordError> {
        if events.is_empty() {
            return Err(WordError::Empty);
        }
        // Validate all supplied occurrence links before modifying the word trie.
        for event in events {
            if !event
                .origin
                .follow()
                .map_err(WordError::Reference)?
                .same(&event.character)
            {
                return Err(WordError::OriginMismatch);
            }
        }
        let mut at = self.root();
        for event in events {
            at = Self::append(&at, &event.character)?;
        }
        let mut episode = self
            .root()
            .provision(b"episodes")
            .provision(&Self::key(boundary));
        match episode.follow() {
            Ok(existing) if existing.same(boundary) => {}
            Err(Fault::Unset) => episode.point_to(boundary),
            Err(fault) => return Err(WordError::Reference(fault)),
            _ => return Err(WordError::CorruptPath),
        }
        // The episode path itself retains ordered original origin identities. Do not
        // duplicate it as a second constituent list; the word path retains character values.
        for event in events {
            episode = Self::append(&episode, &event.origin)?;
        }
        if let Some(word) = episode.child(b"word") {
            if !word.follow().map_err(WordError::Reference)?.same(&at) {
                return Err(WordError::CorruptPath);
            }
        } else {
            episode.provision(b"word").point_to(&at);
            episode.provision(b"boundary").point_to(boundary);
        }
        records::record(&at, b"word", &at, &episode).map_err(WordError::Records)
    }
    /// Allocating inspection only: walk the two already-stored paths in parallel, without
    /// consulting labels or recomputing word recognition. A supplied cap bounds corrupt cycles.
    pub fn inspect_episode(
        &self,
        episode: &Address,
        limit: usize,
    ) -> Result<Vec<CharacterEvent>, WordError> {
        let get = |at: &Address, key: &[u8]| {
            at.child(key)
                .ok_or(WordError::CorruptPath)?
                .follow()
                .map_err(WordError::Reference)
        };
        let mut word = get(episode, b"word")?;
        let boundary = get(episode, b"boundary")?;
        let mut origin_path = episode.clone();
        let mut result = Vec::new();
        while !word.same(&self.root()) {
            if result.len() == limit {
                return Err(WordError::InspectionLimit);
            }
            let character = word.follow().map_err(WordError::Reference)?;
            let origin = origin_path.follow().map_err(WordError::Reference)?;
            if !origin
                .follow()
                .map_err(WordError::Reference)?
                .same(&character)
            {
                return Err(WordError::OriginMismatch);
            }
            result.push(CharacterEvent { character, origin });
            word = get(&word, b"previous")?;
            origin_path = get(&origin_path, b"previous")?;
        }
        if !origin_path
            .follow()
            .map_err(WordError::Reference)?
            .same(&boundary)
        {
            return Err(WordError::CorruptPath);
        }
        result.reverse();
        Ok(result)
    }
    /// Completing a word does not terminate its prefix node: longer words remain possible.
    pub fn resolve(&self, at: &Address) -> Result<WordResolution, WordError> {
        match records::resolve(at, b"word").map_err(WordError::Records)? {
            Selection::Unknown => Ok(WordResolution::Prefix),
            Selection::Unique { candidate, target } => {
                if !target.same(at) {
                    return Err(WordError::CorruptPath);
                }
                Ok(WordResolution::Complete {
                    word: target,
                    candidate,
                })
            }
            Selection::Ambiguous { alternatives } => Ok(WordResolution::Ambiguous { alternatives }),
        }
    }
}
