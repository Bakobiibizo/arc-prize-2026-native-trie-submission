//! Fresh implementation: owned trie edges and direct, non-owning reference edges.
//! No dependency, address decoder, native-ID registry, instruction table, or rich step receipt.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
};
pub mod arc_colors;
pub mod arc_grids;
pub mod arc_markers;
pub mod arc_patterns;
pub mod arc_regions;
pub mod arc_rows;
pub mod axis_context;
pub mod byte_views;
pub mod change_masks;
pub mod character_prefixes;
pub mod complete_count_witness;
pub mod construction_memory;
pub mod constructions;
pub mod counter;
pub mod disk_arc;
pub mod disk_trie;
pub mod episode_transduction;
pub mod graph_export;
pub mod intrinsic_actions;
pub mod intrinsic_grid;
pub mod ordinal_relation;
pub mod paired_states;
#[cfg(test)]
mod path_value_copy_tests;
#[cfg(test)]
mod physical_accounting_tests;
pub mod positional;
pub mod prefixes;
pub mod records;
#[cfg(test)]
mod rectangle_tests;
pub mod reference_branch;
pub mod reference_set_count;
pub mod reference_set_membership;
pub mod region_witness;
pub mod relative_ordinal_count;
pub mod representations;
pub mod singleton_witness;
pub mod sparse_rewrite;
pub mod words;

const SOURCE: &[u8] = b"source";
const TARGET: &[u8] = b"target";
const NEXT: &[u8] = b"next";

#[derive(Default)]
struct Node {
    children: RefCell<BTreeMap<Rc<[u8]>, Address>>,
    content: RefCell<Stored>,
    incoming_token: Option<Rc<[u8]>>,
}

#[derive(Default)]
enum Stored {
    #[default]
    Unset,
    End,
    Link(Weak<Node>),
}

/// A handle to an actual physical node, not a serialized address or index into a host registry.
#[derive(Clone)]
pub struct Address(Rc<Node>);

impl Address {
    pub fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
    /// Setup: create/find one actual trie child. Allocation is permitted here, not in cycle().
    pub fn provision(&self, token: &[u8]) -> Self {
        if let Some(found) = self.child(token) {
            return found;
        }
        let node = Self::prepared_node(token);
        let key = node.0.incoming_token.as_ref().unwrap().clone();
        self.0.children.borrow_mut().insert(key, node.clone());
        node
    }
    // A privately owned one-node trie, labeled for later child publication. No throwaway parent.
    pub(crate) fn prepared_node(token: &[u8]) -> Self {
        Self(Rc::new(Node {
            incoming_token: Some(Rc::from(token)),
            ..Node::default()
        }))
    }
    /// Runtime: exact existing-edge traversal; key is borrowed and no result is decoded.
    pub fn child(&self, token: &[u8]) -> Option<Self> {
        self.0.children.borrow().get(token).cloned()
    }
    /// Ordinary addressing: follow existing child edges, without executing a construction,
    /// allocating a path, decoding tokens, or creating missing nodes.
    pub fn locate<'a>(&self, tokens: impl IntoIterator<Item = &'a [u8]>) -> Option<Self> {
        tokens
            .into_iter()
            .try_fold(self.clone(), |at, token| at.child(token))
    }
    /// Read-only edge-label inspection. Shared with the physical child-map key; no token registry.
    /// The physical cycle never calls this; output rendering reads already-computed token links.
    pub fn token(&self) -> Option<&[u8]> {
        self.0.incoming_token.as_deref()
    }
    pub fn point_to(&self, target: &Self) {
        *self.0.content.borrow_mut() = Stored::Link(Rc::downgrade(&target.0));
    }
    pub fn terminate(&self) {
        *self.0.content.borrow_mut() = Stored::End;
    }
    pub fn is_terminal(&self) -> bool {
        matches!(*self.0.content.borrow(), Stored::End)
    }
    pub fn follow(&self) -> Result<Self, Fault> {
        match &*self.0.content.borrow() {
            Stored::Unset => Err(Fault::Unset),
            Stored::End => Err(Fault::TerminalUsedAsReference),
            Stored::Link(edge) => edge.upgrade().map(Self).ok_or(Fault::Dangling),
        }
    }
    // Physical identity only, never decoded into an address. Not portable/persistent IDs.
    pub(crate) fn identity_key(&self) -> [u8; std::mem::size_of::<usize>()] {
        (Rc::as_ptr(&self.0) as usize).to_be_bytes()
    }
    // Storage publication only. Callers prepare fresh, acyclic owned subtrees; backlinks are weak.
    // The incoming key is unchanged, so sharing does not invent conflicting token metadata.
    pub(crate) fn publish_prepared_child(&self, prepared: &Self) -> Self {
        let key = prepared
            .0
            .incoming_token
            .as_ref()
            .expect("prepared child has an incoming key")
            .clone();
        self.0
            .children
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| prepared.clone())
            .clone()
    }
    pub(crate) fn sole_child(&self) -> Result<Option<Self>, usize> {
        let children = self.0.children.borrow();
        match children.len() {
            0 => Ok(None),
            1 => Ok(children.first_key_value().map(|(_, at)| at.clone())),
            n => Err(n),
        }
    }
    // Diagnostic export of actual children. Never used by resolution or recording decisions.
    pub(crate) fn inspect_children(&self) -> Vec<Self> {
        self.0.children.borrow().values().cloned().collect()
    }
    fn snapshot(&self) -> Result<Content, Fault> {
        if self.is_terminal() {
            Ok(Content::End)
        } else {
            self.follow().map(Content::Link)
        }
    }
    fn place(&self, value: &Content) {
        match value {
            Content::End => self.terminate(),
            Content::Link(target) => self.point_to(target),
        }
    }
}

enum Content {
    End,
    Link(Address),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    MissingField,
    Unset,
    Dangling,
    TerminalUsedAsReference,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Advanced,
    Halted,
}

/// Storage roots own the physical nodes through child edges. Reference edges are weak, so
/// cyclic computations do not form reference-count ownership cycles. No owner/index table exists.
pub struct Trie {
    root: Address,
}
impl Default for Trie {
    fn default() -> Self {
        Self {
            root: Address(Rc::new(Node::default())),
        }
    }
}
impl Trie {
    pub fn root(&self) -> Address {
        self.root.clone()
    }
    pub fn provision(&self, tokens: &[&[u8]]) -> Address {
        tokens
            .iter()
            .fold(self.root(), |node, token| node.provision(token))
    }
}

/// Seed loader for the three-field physical convention. Fields themselves are writable cells.
pub fn install(at: &Address, source: &Address, target: &Address, next_cell: &Address) {
    at.provision(SOURCE).point_to(source);
    at.provision(TARGET).point_to(target);
    at.provision(NEXT).point_to(next_cell);
}

/// Inspection/setup names for first-class instruction fields, not runtime opcodes.
pub fn source_field(at: &Address) -> Option<Address> {
    at.child(SOURCE)
}
pub fn target_field(at: &Address) -> Option<Address> {
    at.child(TARGET)
}
pub fn next_field(at: &Address) -> Option<Address> {
    at.child(NEXT)
}

pub struct Machine {
    pub states: Trie,
    pub output: Trie,
    pointer: Address,
    pub cycles: u64,
}
impl Default for Machine {
    fn default() -> Self {
        let states = Trie::default();
        let pointer = states.provision(&[b"instruction-pointer"]);
        Self {
            states,
            output: Trie::default(),
            pointer,
            cycles: 0,
        }
    }
}
impl Machine {
    /// External pulse/entry boundary. No machine-specific decision is performed.
    pub fn enter(&mut self, state: &Address) {
        self.pointer.point_to(state);
        self.cycles = 0;
    }
    pub fn current(&self) -> Result<Address, Fault> {
        self.pointer.follow()
    }
    pub fn pointer(&self) -> Address {
        self.pointer.clone()
    }
    pub fn cycle(&mut self) -> Result<Outcome, Fault> {
        let state = self.pointer.follow()?;
        if state.is_terminal() {
            return Ok(Outcome::Halted);
        }
        // These are native node positions reached through existing trie edges.
        let source = state.child(SOURCE).ok_or(Fault::MissingField)?.follow()?;
        let target = state.child(TARGET).ok_or(Fault::MissingField)?.follow()?;
        let next_cell = state.child(NEXT).ok_or(Fault::MissingField)?.follow()?;
        let content = source.snapshot()?;
        // Validate post-write continuation before either write. Pointer identity handles aliasing
        // without a cloned memory image or a rollback table. Descriptors above are latched.
        let next = if target.same(&next_cell) {
            match &content {
                Content::Link(address) => address.clone(),
                Content::End => return Err(Fault::TerminalUsedAsReference),
            }
        } else {
            next_cell.follow()?
        };
        target.place(&content);
        self.pointer.point_to(&next);
        self.cycles += 1;
        Ok(Outcome::Advanced)
    }
}

#[cfg(test)]
mod ownership {
    use super::*;
    #[test]
    fn reference_cycles_do_not_keep_a_trie_alive() {
        let weak = {
            let trie = Trie::default();
            let a = trie.provision(&[b"a"]);
            let b = trie.provision(&[b"b"]);
            a.point_to(&b);
            b.point_to(&a);
            Rc::downgrade(&a.0)
        };
        assert!(weak.upgrade().is_none());
    }
}
