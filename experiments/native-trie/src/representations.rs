//! Representation-selected paths in one physical trie; no encoding decoder or semantic evaluator.
use crate::{
    Address, Fault, Machine, install,
    records::{self, RecordError, Recorded, Selection},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasError {
    EmptyPath,
    Records(RecordError),
    Machine(Fault),
    Incomplete,
}
pub enum Resolution {
    Missing,
    Prefix {
        anchor: Address,
    },
    Complete {
        anchor: Address,
        candidate: Address,
        character: Address,
    },
    Ambiguous {
        anchor: Address,
        alternatives: usize,
    },
}
pub struct Representations {
    pub machine: Machine,
    pub result: Address,
    pub construction: Address,
    selected: Address,
    halt: Address,
    serial: usize,
}
impl Default for Representations {
    fn default() -> Self {
        Self::new()
    }
}
impl Representations {
    pub fn new() -> Self {
        let machine = Machine::default();
        let root = machine.states.root();
        let selected = root.provision(b"selected-address");
        let result = machine.output.root().provision(b"character-address");
        let halt = root.provision(b"alias-return");
        halt.terminate();
        let next = root.provision(b"alias-return-cell");
        next.point_to(&halt);
        let construction = root
            .provision(b"constructions")
            .provision(b"value-to-value");
        install(&construction, &selected, &result, &next);
        Self {
            machine,
            result,
            construction,
            selected,
            halt,
            serial: 0,
        }
    }
    /// Canonical identity is supplied data; this does not infer a character from a host string.
    pub fn character(&self, id: &[u8]) -> Address {
        self.machine
            .states
            .root()
            .provision(b"characters")
            .provision(id)
    }
    pub fn root(&self) -> Address {
        self.machine.states.root()
    }
    /// Caller-owned raw source data belongs in a source subtree, not a summary of inferred values.
    pub fn source(&self, id: &[u8]) -> Address {
        self.root().provision(b"sources").provision(id)
    }
    /// A shown association. Path boundaries/meaning are supplied, never decoded here.
    pub fn teach(
        &mut self,
        path: &[&[u8]],
        character: &Address,
        source: &Address,
    ) -> Result<Recorded, AliasError> {
        let first = path.first().ok_or(AliasError::EmptyPath)?;
        let selector = self.root().provision(first);
        let anchor = path[1..]
            .iter()
            .fold(selector.clone(), |at, key| at.provision(key));
        let observation = self
            .root()
            .provision(b"observations")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        observation.provision(b"representation").point_to(&selector);
        observation.provision(b"path").point_to(&anchor);
        observation.provision(b"source").point_to(source);
        observation
            .provision(b"construction")
            .point_to(&self.construction);
        records::record(&anchor, b"value-to-value", character, &observation)
            .map_err(AliasError::Records)
    }
    pub fn lookup<'a>(
        &self,
        path: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Resolution, AliasError> {
        match self.root().locate(path) {
            Some(at) => self.at(&at),
            None => Ok(Resolution::Missing),
        }
    }
    /// Reuse a retained prefix/native address instead of replaying its path.
    pub fn at(&self, anchor: &Address) -> Result<Resolution, AliasError> {
        match records::resolve(anchor, b"value-to-value").map_err(AliasError::Records)? {
            Selection::Unknown => Ok(Resolution::Prefix {
                anchor: anchor.clone(),
            }),
            Selection::Unique { candidate, target } => Ok(Resolution::Complete {
                anchor: anchor.clone(),
                candidate,
                character: target,
            }),
            Selection::Ambiguous { alternatives } => Ok(Resolution::Ambiguous {
                anchor: anchor.clone(),
                alternatives,
            }),
        }
    }
    /// Optional single shared construction, after lookup. Binding copies an already-resolved
    /// native address at the entry boundary; it does not decode or derive a character value.
    pub fn emit(&mut self, character: &Address) -> Result<u64, AliasError> {
        self.emit_with(&self.construction.clone(), character)
    }
    /// Execute the construction actually selected from trie knowledge, with the given value binding.
    pub fn emit_with(
        &mut self,
        construction: &Address,
        character: &Address,
    ) -> Result<u64, AliasError> {
        self.selected.point_to(character);
        self.machine.enter(construction);
        self.machine.cycle().map_err(AliasError::Machine)?;
        if !self
            .machine
            .current()
            .map_err(AliasError::Machine)?
            .same(&self.halt)
        {
            return Err(AliasError::Incomplete);
        }
        Ok(self.machine.cycles)
    }
}
