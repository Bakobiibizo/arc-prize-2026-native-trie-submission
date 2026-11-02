//! Compact transferred scalar-identity knowledge. Runtime has no scalar integers/range table.
//! Numeric range decomposition lives only in the offline teaching compiler/test oracle.
use crate::{
    Address,
    prefixes::{self, PrefixMatch, PrefixSelection},
    records::{self, RecordError},
    representations::{AliasError, Representations},
};
pub type ScalarPath = [[u8; 1]; 21];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterError {
    InvalidBit,
    Unrecorded,
    Ambiguous(usize),
    Record(RecordError),
    Execute(AliasError),
}
pub struct CharacterProjection {
    pub value: Address,
    pub matched_prefix: Address,
    pub construction: Address,
    pub depth: usize,
}
pub struct CharacterIdentity {
    pub model: Representations,
    pub rules: Address,
    values: Address,
}
impl Default for CharacterIdentity {
    fn default() -> Self {
        Self::new()
    }
}
impl CharacterIdentity {
    pub fn new() -> Self {
        let model = Representations::new();
        let rules = model.root().provision(b"scalar");
        let values = model.root().provision(b"values");
        let manifest = model
            .root()
            .provision(b"manifests")
            .provision(include_bytes!("../curriculum/character-prefixes.json"));
        // Ten supplied prefix facts, not an answer entry or specialized program per scalar.
        for line in include_str!("../curriculum/character-prefixes.txt").lines() {
            let mut at = rules.clone();
            for token in line.bytes() {
                let bit = match token {
                    b'0' => 0,
                    b'1' => 1,
                    _ => panic!("invalid teaching prefix"),
                };
                at = at.provision(&[bit]);
            }
            let source = model.source(line.as_bytes());
            source.provision(b"kind").point_to(&rules);
            source.provision(b"operation").point_to(&model.construction);
            source.provision(b"manifest").point_to(&manifest);
            records::record(&at, b"construction", &model.construction, &source)
                .expect("valid teaching data");
        }
        Self {
            model,
            rules,
            values,
        }
    }
    fn validate(path: &ScalarPath) -> Result<(), CharacterError> {
        if path.iter().any(|token| token[0] > 1) {
            Err(CharacterError::InvalidBit)
        } else {
            Ok(())
        }
    }
    pub fn select(&self, path: &ScalarPath) -> Result<PrefixMatch, CharacterError> {
        Self::validate(path)?;
        match prefixes::select(&self.rules, path.iter().map(|token| token.as_slice()))
            .map_err(CharacterError::Record)?
        {
            PrefixSelection::Unknown => Err(CharacterError::Unrecorded),
            PrefixSelection::Ambiguous { alternatives, .. } => {
                Err(CharacterError::Ambiguous(alternatives))
            }
            PrefixSelection::Match(found) => Ok(found),
        }
    }
    /// Physical input-value interning, not scalar decoding or a computed answer table.
    /// One immutable supplied 21-bit token names a canonical datum in actual trie adjacency.
    /// This can allocate on first occurrence; repeated values reuse the same native identity.
    pub fn intern_value(&self, path: &ScalarPath) -> Result<Address, CharacterError> {
        Self::validate(path)?;
        let bits: [u8; 21] = std::array::from_fn(|i| path[i][0]);
        Ok(self.values.provision(&bits))
    }
    /// Actual physical input-value cardinality, not a semantic cache counter.
    pub fn interned_values(&self) -> usize {
        match self.values.sole_child() {
            Ok(None) => 0,
            Ok(Some(_)) => 1,
            Err(n) => n,
        }
    }
    pub fn construct(&mut self, path: &ScalarPath) -> Result<CharacterProjection, CharacterError> {
        let selected = self.select(path)?;
        let value = self.intern_value(path)?;
        self.model
            .emit_with(&selected.construction, &value)
            .map_err(CharacterError::Execute)?;
        Ok(CharacterProjection {
            value: self
                .model
                .result
                .follow()
                .map_err(|fault| CharacterError::Execute(AliasError::Machine(fault)))?,
            matched_prefix: selected.anchor,
            construction: selected.construction,
            depth: selected.depth,
        })
    }
}
