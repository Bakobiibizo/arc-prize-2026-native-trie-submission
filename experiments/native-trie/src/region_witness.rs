//! Finite native DFS through bound source views. All semantic control is stored copy/follow.
//! Loader loops wire supplied topology only; execute never iterates cells or reads a branch tag.
use crate::{
    Address, Fault, Machine,
    arc_grids::{Grid, GridError, Grids},
    install,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionError {
    Grid(GridError),
    Machine(Fault),
    Duplicate,
    Limit,
    Incomplete,
}
impl From<Fault> for RegionError {
    fn from(e: Fault) -> Self {
        Self::Machine(e)
    }
}
impl From<GridError> for RegionError {
    fn from(e: GridError) -> Self {
        Self::Grid(e)
    }
}
const PORTS: [&[u8]; 4] = [b"north", b"west", b"south", b"east"];
fn field(a: &Address, k: &[u8]) -> Result<Address, RegionError> {
    Ok(a.child(k).ok_or(GridError::Malformed)?)
}
fn copy(at: &Address, source: &Address, target: &Address, next: &Address) {
    let link = at.provision(b"continuation");
    link.point_to(next);
    install(at, source, target, &link)
}
fn constant(at: &Address, k: &[u8], value: &Address) -> Address {
    let c = at.provision(k);
    c.point_to(value);
    c
}
fn jump(at: &Address, next_cell: &Address, discard: &Address) {
    install(at, next_cell, discard, next_cell)
}
/// The same post-write alias control validated by reference_branch, with fixed bound operands.
fn branch(at: &Address, x: &Address, y: &Address, yes: &Address, no: &Address) -> Address {
    let init = at.provision(b"reset-y");
    let conditional = at.provision(b"write-x-follow-y");
    copy(&init, &constant(at, b"different", no), y, &conditional);
    install(&conditional, &constant(at, b"same", yes), x, y);
    init
}
struct View {
    root: Address,
    facts: Address,
    source: Address,
    entry: Address,
    body: Address,
    mark: Address,
    seen: Address,
    saved: Address,
    class: Address,
    expanded: Address,
    value: Address,
    ports: [Address; 4],
    cache: Address,
}
pub struct RegionWitness {
    pub machine: Machine,
    pub root: Address,
    pub result: Address,
    pub construction: Address,
    draft: Address,
    activation: Address,
    halt: Address,
    owner: Address,
    _input: Grid,
    views: Vec<View>,
    pub budget: usize,
    completed: bool,
    failed: bool,
}
/// Allocating inspection facts; never consumed by traversal or learning decisions.
pub struct RegionFact {
    pub source: Address,
    pub class: Option<Address>,
    pub expanded: bool,
    pub edges: Vec<(Address, bool)>,
}
pub struct RegionProjection {
    pub value: Address,
    pub proof: Address,
    pub copies: u64,
    pub recorded_views: usize,
    _owner: Address,
    _input: Grid,
}
impl RegionProjection {
    pub(crate) fn owner_root(&self) -> Address {
        self._owner.clone()
    }
}
impl RegionWitness {
    /// Cache/parent are private model storage, not fields of the observed grid.
    pub fn prepare(
        grids: &Grids,
        input: &Grid,
        seed: &Address,
        parent: &Address,
        id: &[u8],
        cache: &Address,
    ) -> Result<Self, RegionError> {
        if !input.owner.same(&grids.storage.root()) {
            return Err(GridError::ForeignSpace.into());
        }
        Self::prepare_bound(input, seed, parent, id, cache)
    }
    /// Supplied finite typed unary topology; physical provisioning, not a runtime count.
    pub fn cardinality_zero(parent: &Address) -> Address {
        let bank = parent.provision(b"cardinalities");
        let zero = bank.provision(b"zero");
        if !bank.child(b"ready-900").is_some_and(|n| n.is_terminal()) {
            let mut at = zero.clone();
            for _ in 0..900 {
                let next = at.provision(b"successor");
                at.point_to(&next);
                at = next;
            }
            bank.provision(b"ready-900").terminate();
        }
        zero
    }
    pub fn prepare_counted(
        grids: &Grids,
        input: &Grid,
        seed: &Address,
        parent: &Address,
        id: &[u8],
        cache: &Address,
    ) -> Result<Self, RegionError> {
        if !input.owner.same(&grids.storage.root()) {
            return Err(GridError::ForeignSpace.into());
        }
        Self::prepare_inner(input, seed, parent, id, cache, true)
    }
    pub(crate) fn prepare_counted_bound(
        input: &Grid,
        seed: &Address,
        parent: &Address,
        id: &[u8],
        cache: &Address,
    ) -> Result<Self, RegionError> {
        Self::prepare_inner(input, seed, parent, id, cache, true)
    }
    pub(crate) fn prepare_bound(
        input: &Grid,
        seed: &Address,
        parent: &Address,
        id: &[u8],
        cache: &Address,
    ) -> Result<Self, RegionError> {
        Self::prepare_inner(input, seed, parent, id, cache, false)
    }
    fn prepare_inner(
        input: &Grid,
        seed: &Address,
        parent: &Address,
        id: &[u8],
        cache: &Address,
        counted: bool,
    ) -> Result<Self, RegionError> {
        let proofs = parent.provision(b"proofs");
        if proofs.child(id).is_some() {
            return Err(RegionError::Duplicate);
        }
        let root = proofs.provision(id);
        root.provision(b"input").point_to(&input.anchor);
        root.provision(b"seed").point_to(seed);
        let machine = Machine::default();
        let work = Address::prepared_node(b"execution");
        let draft = Address::prepared_node(b"native-facts");
        root.provision(b"producer").point_to(
            &parent
                .provision(b"region-constructors")
                .provision(b"copy-dfs-v2"),
        );
        root.provision(b"proof-format")
            .provision(b"native-written-views-v2");
        let values = work.provision(b"alias-values");
        let alias = |v: &Address| values.provision(&v.identity_key());
        let outside = field(&input.owner, b"outside")?;
        let unknown = field(&input.owner, b"unresolved")?;
        let outside_alias = alias(&outside);
        let unknown_alias = alias(&unknown);
        let seed_value = field(seed, b"value")?.follow()?;
        let seed_alias = alias(&seed_value);
        let discard = work.provision(b"discard");
        let ret = work.provision(b"return-register");
        let halt = work.provision(b"halt");
        halt.terminate();
        let result = root.provision(b"result");
        let exhausted = root.provision(b"frontier-exhausted");
        let tags = parent.provision(b"relation-tags");
        let escape = tags.provision(b"escape");
        let bounded = tags.provision(b"bounded");
        let unresolved = tags.provision(b"unresolved");
        let yes = tags.provision(b"true");
        let cardinality = if counted {
            // Supplied typed unary topology, not a scalar decoder/answer table.
            let zero = Self::cardinality_zero(parent);
            root.provision(b"producer").point_to(
                &parent
                    .provision(b"region-constructors")
                    .provision(b"copy-dfs-counted-v3"),
            );
            let cursor = root.provision(b"observed-cardinality");
            cursor.point_to(&zero);
            Some(cursor)
        } else {
            None
        };
        let class_root = parent.provision(b"class-tags");
        let member = class_root.provision(b"member");
        let barrier = class_root.provision(b"barrier");
        let outside_class = class_root.provision(b"outside");
        let unknown_class = class_root.provision(b"unknown");
        let code = work.provision(b"construction");
        let finish = code.provision(b"finish");
        let dispatch = code.provision(b"escaped-dispatch");
        let check_unknown = code.provision(b"unknown-dispatch");
        let skip = code.provision(b"already-seen-return");
        jump(&skip, &ret, &discard);
        let emit_escape = code.provision(b"emit-escape");
        let emit_bounded = code.provision(b"emit-bounded");
        let emit_unknown = code.provision(b"emit-unresolved");
        copy(
            &emit_escape,
            &constant(&work, b"escape-tag", &escape),
            &result,
            &halt,
        );
        copy(
            &emit_bounded,
            &constant(&work, b"bounded-tag", &bounded),
            &result,
            &halt,
        );
        copy(
            &emit_unknown,
            &constant(&work, b"unresolved-tag", &unresolved),
            &result,
            &halt,
        );
        let escaped_flag = work.provision(b"escaped-continuation");
        escaped_flag.point_to(&check_unknown);
        let unknown_flag = work.provision(b"unknown-continuation");
        unknown_flag.point_to(&emit_bounded);
        let true_cell = constant(&work, b"true-tag", &yes);
        let escaped_cell = constant(&work, b"escape-action", &emit_escape);
        let unknown_cell = constant(&work, b"unknown-action", &emit_unknown);
        let skip_cell = constant(&work, b"skip-return", &skip);
        let proof_cell = constant(&work, b"proof-reference", &root);
        copy(&finish, &true_cell, &exhausted, &dispatch);
        jump(&dispatch, &escaped_flag, &discard);
        jump(&check_unknown, &unknown_flag, &discard);
        ret.point_to(&finish);
        let view_root = work.provision(b"views");
        let mut views = Vec::new();
        for y in 0..input.height() {
            for x in 0..input.width() {
                let cell = input.cell(y, x)?;
                let value = field(&cell, b"value")?.follow()?;
                views.push(Self::new_view(
                    &view_root,
                    &draft,
                    &cell,
                    alias(&value),
                    cache,
                ));
            }
        }
        views.push(Self::new_view(
            &view_root,
            &draft,
            &outside,
            outside_alias.clone(),
            cache,
        ));
        let seed_entry = view_root
            .child(&seed.identity_key())
            .ok_or(GridError::Malformed)?
            .child(b"entry")
            .ok_or(GridError::Malformed)?;
        for (view_index, view) in views.iter().enumerate() {
            let v = &view.root;
            let outside_state = v.provision(b"classify-outside");
            let return_state = v.provision(b"return");
            install(&return_state, &view.saved, &ret, &ret);
            let outflag = v.provision(b"escape-flag");
            copy(
                &outside_state,
                &constant(v, b"outside-tag", &outside_class),
                &view.class,
                &outflag,
            );
            copy(&outflag, &escaped_cell, &escaped_flag, &return_state);
            if counted && view_index + 1 == views.len() {
                // Last view is the supplied outside sentinel, NOT a classified source pixel.
                // Preserve its actual native comparison/path; don't allocate unreachable
                // member/unknown/barrier/port-call code for this typed sentinel.
                let compare = branch(
                    &v.provision(b"compare-outside"),
                    &view.value,
                    &outside_alias,
                    &outside_state,
                    &halt,
                );
                jump(&view.entry, &view.seen, &discard);
                copy(&view.body, &ret, &view.saved, &view.mark);
                copy(&view.mark, &skip_cell, &view.seen, &compare);
                for port in PORTS {
                    let edge = view.facts.provision(b"edges").provision(port);
                    edge.provision(b"source-neighbor").point_to(&outside);
                    edge.provision(b"checked");
                }
                continue;
            }
            let unknown_state = v.provision(b"classify-unknown");
            let barrier_state = v.provision(b"classify-barrier");
            let member_state = v.provision(b"classify-member");
            let write_cache = v.provision(b"write-member-proof");
            let complete = v.provision(b"complete-member");
            let unkflag = v.provision(b"unknown-flag");
            copy(
                &unknown_state,
                &constant(v, b"unknown-tag", &unknown_class),
                &view.class,
                &unkflag,
            );
            copy(&unkflag, &unknown_cell, &unknown_flag, &return_state);
            copy(
                &barrier_state,
                &constant(v, b"barrier-tag", &barrier),
                &view.class,
                &return_state,
            );
            copy(
                &member_state,
                &constant(v, b"member-tag", &member),
                &view.class,
                &write_cache,
            );
            let member_next = if let Some(cursor) = &cardinality {
                let patch = v.provision(b"count-patch");
                let step = v.provision(b"count-step");
                copy(&step, &discard, cursor, &view.ports[0]);
                copy(
                    &patch,
                    cursor,
                    &crate::source_field(&step).ok_or(GridError::Malformed)?,
                    &step,
                );
                patch
            } else {
                view.ports[0].clone()
            };
            copy(&write_cache, &proof_cell, &view.cache, &member_next);
            copy(&complete, &true_cell, &view.expanded, &return_state);
            let compare_value = branch(
                &v.provision(b"compare-seed"),
                &view.value,
                &seed_alias,
                &member_state,
                &barrier_state,
            );
            let compare_unknown = branch(
                &v.provision(b"compare-unknown"),
                &view.value,
                &unknown_alias,
                &unknown_state,
                &compare_value,
            );
            let compare_outside = branch(
                &v.provision(b"compare-outside"),
                &view.value,
                &outside_alias,
                &outside_state,
                &compare_unknown,
            );
            jump(&view.entry, &view.seen, &discard);
            copy(&view.body, &ret, &view.saved, &view.mark);
            copy(&view.mark, &skip_cell, &view.seen, &compare_outside);
            for (i, port) in PORTS.iter().enumerate() {
                let after = view.ports.get(i + 1).unwrap_or(&complete);
                let set_return = v.provision(b"port-return").provision(port);
                let call = v.provision(b"port-call").provision(port);
                let edge = view.facts.provision(b"edges").provision(port);
                let neighbor = if view.source.same(&outside) {
                    outside.clone()
                } else {
                    field(&view.source, port)?.follow()?
                };
                let neighbor_view = view_root
                    .child(&neighbor.identity_key())
                    .ok_or(GridError::Malformed)?;
                edge.provision(b"source-neighbor").point_to(&neighbor);
                copy(
                    &view.ports[i],
                    &true_cell,
                    &edge.provision(b"checked"),
                    &set_return,
                );
                copy(
                    &set_return,
                    &constant(&set_return, b"caller-next", after),
                    &ret,
                    &call,
                );
                jump(
                    &call,
                    &constant(&call, b"neighbor-entry", &field(&neighbor_view, b"entry")?),
                    &discard,
                );
            }
        }
        let call_seed = code.provision(b"call-seed");
        jump(
            &call_seed,
            &constant(&work, b"seed-entry", &seed_entry),
            &discard,
        );
        let entry = branch(
            &code.provision(b"check-seed-unknown"),
            &seed_alias,
            &unknown_alias,
            &emit_unknown,
            &call_seed,
        );
        let mut witness = Self {
            machine,
            root,
            result,
            construction: code,
            draft,
            activation: work,
            halt,
            owner: parent.clone(),
            _input: input.clone(),
            views,
            budget: 40 * input.width() * input.height() + 32,
            completed: false,
            failed: false,
        };
        witness.machine.enter(&entry);
        Ok(witness)
    }
    fn new_view(
        root: &Address,
        draft: &Address,
        source: &Address,
        value: Address,
        cache: &Address,
    ) -> View {
        let node = root.provision(&source.identity_key());
        let facts = draft.provision(&source.identity_key());
        facts.provision(b"source").point_to(source);
        let entry = node.provision(b"entry");
        let body = node.provision(b"save-return");
        let seen = node.provision(b"seen-dispatch");
        seen.point_to(&body);
        View {
            root: node.clone(),
            facts: facts.clone(),
            source: source.clone(),
            entry,
            body,
            mark: node.provision(b"mark-seen"),
            seen,
            saved: node.provision(b"saved-return"),
            class: facts.provision(b"class"),
            expanded: facts.provision(b"expanded"),
            value,
            ports: std::array::from_fn(|i| node.provision(b"port-touch").provision(PORTS[i])),
            cache: cache.provision(&source.identity_key()),
        }
    }
    pub fn execute(&mut self, limit: usize) -> Result<u64, RegionError> {
        if self.completed || self.failed {
            return Err(RegionError::Incomplete);
        }
        for _ in 0..limit {
            if let Err(f) = self.machine.cycle() {
                self.failed = true;
                return Err(f.into());
            }
            if self.machine.current()?.same(&self.halt) {
                self.completed = true;
                return Ok(self.machine.cycles);
            }
        }
        Err(RegionError::Limit)
    }
    pub fn record(self) -> Result<RegionProjection, RegionError> {
        if !self.completed {
            return Err(RegionError::Incomplete);
        }
        let value = self.result.follow()?;
        let recorded_views = self.publish_written();
        self.root.provision(b"committed").terminate();
        self.root
            .provision(b"copies")
            .provision(&self.machine.cycles.to_be_bytes());
        Ok(RegionProjection {
            value,
            proof: self.root.clone(),
            copies: self.machine.cycles,
            recorded_views,
            _owner: self.owner.clone(),
            _input: self._input.clone(),
        })
    }
    fn publish_written(&self) -> usize {
        let views = self.root.provision(b"views");
        let mut count = 0;
        for fact in self.draft.inspect_children() {
            // Presence of an actual native write, not classification of a source pixel.
            if !matches!(fact.child(b"class").unwrap().follow(), Err(Fault::Unset)) {
                views.publish_prepared_child(&fact);
                count += 1;
            }
        }
        count
    }
    /// Physical owned-node census, resource/inspection only, never a traversal decision.
    pub fn owned_nodes(root: &Address) -> usize {
        let mut stack = vec![root.clone()];
        let mut n = 0;
        while let Some(at) = stack.pop() {
            n += 1;
            stack.extend(at.0.children.borrow().values().cloned());
        }
        n
    }
    /// Conservative physical workspace accounting (not RSS): fixed node/map proxy plus
    /// actual token bytes, counting shared owned paths conservatively more than once.
    pub fn storage_charge(root: &Address) -> usize {
        let mut stack = vec![root.clone()];
        let mut bytes = 0;
        while let Some(at) = stack.pop() {
            bytes += 512 + at.token().map_or(0, |b| b.len());
            // Same owned paths/multiplicity/charge; avoid one temporary Vec allocation
            // per visited node. Pure resource inspection, never kernel execution.
            stack.extend(at.0.children.borrow().values().cloned());
        }
        bytes
    }
    /// Diagnostic link-lifetime audit only; not used by inference or to repair a graph.
    pub fn dangling_links(root: &Address) -> usize {
        let mut stack = vec![root.clone()];
        let mut n = 0;
        while let Some(at) = stack.pop() {
            n += usize::from(matches!(at.follow(), Err(Fault::Dangling)));
            stack.extend(at.0.children.borrow().values().cloned());
        }
        n
    }
    pub fn recorded_view(proof: &Address, source: &Address) -> Option<Address> {
        proof.child(b"views")?.child(&source.identity_key())
    }
    pub fn inspect_facts(proof: &Address) -> Result<Vec<RegionFact>, RegionError> {
        let mut facts = Vec::new();
        for view in field(proof, b"views")?.inspect_children() {
            let source = field(&view, b"source")?.follow()?;
            let class = field(&view, b"class")?.follow().ok();
            let expanded = field(&view, b"expanded")?.follow().is_ok();
            let mut edges = Vec::new();
            for p in PORTS {
                let edge = field(&field(&view, b"edges")?, p)?;
                edges.push((
                    field(&edge, b"source-neighbor")?.follow()?,
                    field(&edge, b"checked")?.follow().is_ok(),
                ));
            }
            facts.push(RegionFact {
                source,
                class,
                expanded,
                edges,
            });
        }
        Ok(facts)
    }
    /// Diagnostic lifetime/causality inspection only.
    pub fn activation_root(&self) -> Address {
        self.activation.clone()
    }
    /// Diagnostic/native field addressing; this does not compute a membership classification.
    pub fn view(&self, source: &Address) -> Option<Address> {
        self.draft.child(&source.identity_key())
    }
    /// Pure physical lookup of a native-written member proof; incomplete proof is not a result.
    pub fn cached(
        cache: &Address,
        cell: &Address,
    ) -> Result<Option<(Address, Address)>, RegionError> {
        let Some(slot) = cache.child(&cell.identity_key()) else {
            return Ok(None);
        };
        let proof = match slot.follow() {
            Ok(p) => p,
            Err(Fault::Unset) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        if !proof.child(b"committed").is_some_and(|a| a.is_terminal()) {
            return Ok(None);
        }
        let result = field(&proof, b"result")?;
        match result.follow() {
            Ok(value) => Ok(Some((proof, value))),
            Err(Fault::Unset) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    /// Retain an actual committed cached projection, not a recomputed classification.
    /// Source/input/owner are checked before exposing its native result to another witness.
    pub(crate) fn projection_from_committed(
        owner: &Address,
        input: &Grid,
        proof: &Address,
    ) -> Result<RegionProjection, RegionError> {
        if !proof.child(b"committed").is_some_and(|n| n.is_terminal())
            || !field(proof, b"input")?.follow()?.same(&input.anchor)
            || !field(owner, b"proofs")?
                .inspect_children()
                .iter()
                .any(|p| p.same(proof))
        {
            return Err(RegionError::Incomplete);
        }
        let count = field(proof, b"copies")?.inspect_children();
        let bytes = count
            .first()
            .and_then(|c| c.token())
            .ok_or(RegionError::Incomplete)?;
        let copies = u64::from_be_bytes(bytes.try_into().map_err(|_| RegionError::Incomplete)?);
        if count.len() != 1 {
            return Err(RegionError::Incomplete);
        }
        Ok(RegionProjection {
            value: field(proof, b"result")?.follow()?,
            proof: proof.clone(),
            copies,
            recorded_views: field(proof, b"views")?.inspect_children().len(),
            _owner: owner.clone(),
            _input: input.clone(),
        })
    }
    /// Diagnostic member/edge census only; never called to select a traversal branch/result.
    pub fn inspect(&self) -> Result<(usize, usize, usize), RegionError> {
        let member = self
            .owner
            .child(b"class-tags")
            .and_then(|a| a.child(b"member"))
            .ok_or(GridError::Malformed)?;
        let mut members = 0;
        let mut checked = 0;
        let mut expanded = 0;
        for v in &self.views {
            if v.class.follow().is_ok_and(|c| c.same(&member)) {
                members += 1;
                expanded += usize::from(v.expanded.follow().is_ok());
                for p in PORTS {
                    checked += usize::from(
                        v.facts
                            .child(b"edges")
                            .and_then(|a| a.child(p))
                            .and_then(|a| a.child(b"checked"))
                            .is_some_and(|a| a.follow().is_ok()),
                    );
                }
            }
        }
        Ok((members, checked, expanded))
    }
}
impl Drop for RegionWitness {
    fn drop(&mut self) {
        self.publish_written();
        if self
            .root
            .child(b"committed")
            .is_some_and(|a| a.is_terminal())
        {
            self.root.provision(b"activation-retired").terminate();
        } else {
            // Incomplete/faulted control state is primary failure evidence, not disposable.
            self.activation.publish_prepared_child(&self.draft);
            self.activation
                .provision(b"machine")
                .publish_prepared_child(&self.machine.pointer());
            self.activation
                .provision(b"cycles")
                .provision(&self.machine.cycles.to_be_bytes());
            self.root.publish_prepared_child(&self.activation);
        }
    }
}
#[cfg(test)]
mod retirement_lifetime {
    use super::*;
    #[test]
    fn committed_proof_does_not_keep_native_execution_graph_alive() {
        let s = Grids::new();
        let source = s.source(b"source", b"1x1 opaque data").unwrap();
        let g = s
            .admit(
                b"input",
                crate::arc_grids::Side::QueryInput,
                &[vec![Some(0)]],
                &source,
            )
            .unwrap();
        let owner = crate::Trie::default();
        let root = owner.root();
        let mut w = RegionWitness::prepare(
            &s,
            &g,
            &g.cell(0, 0).unwrap(),
            &root,
            b"proof",
            &root.provision(b"cache"),
        )
        .unwrap();
        let weak = std::rc::Rc::downgrade(&w.activation_root().0);
        w.execute(w.budget).unwrap();
        let proof = w.record().unwrap();
        assert!(weak.upgrade().is_none());
        assert_eq!(proof.value.token(), Some(b"escape".as_slice()));
    }
}
