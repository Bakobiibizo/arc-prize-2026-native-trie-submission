//! Native observed-one AND no unknown frontier, publishing original source color.
//! Loader binds actual committed proof references; no host class/count/color comparison.
use crate::{
    Address, Fault, Machine,
    arc_grids::GridError,
    install,
    region_witness::{RegionError, RegionProjection},
};
fn field(at: &Address, key: &[u8]) -> Result<Address, RegionError> {
    at.child(key).ok_or(GridError::Malformed.into())
}
fn constant(at: &Address, key: &[u8], value: &Address) -> Address {
    let c = at.provision(key);
    c.point_to(value);
    c
}
fn copy(at: &Address, source: &Address, target: &Address, next: &Address) {
    install(at, source, target, &constant(at, b"continuation", next));
}
fn branch(at: &Address, x: &Address, y: &Address, yes: &Address, no: &Address) -> Address {
    let reset = at.provision(b"reset-y");
    let check = at.provision(b"write-x-follow-y");
    copy(&reset, &constant(at, b"different", no), y, &check);
    install(&check, &constant(at, b"same", yes), x, y);
    reset
}
pub struct SingletonWitness {
    pub machine: Machine,
    pub root: Address,
    pub construction: Address,
    pub budget: usize,
    halt: Address,
    completed: bool,
    failed: bool,
    region: RegionProjection,
    owner: Address,
}
pub struct SingletonProjection {
    pub proof: Address,
    pub status: Address,
    pub candidate: Option<Address>,
    pub copies: u64,
    _region: RegionProjection,
    _owner: Address,
}
impl SingletonWitness {
    pub fn prepare(
        region: RegionProjection,
        parent: &Address,
        id: &[u8],
    ) -> Result<Self, RegionError> {
        if !region
            .proof
            .child(b"committed")
            .is_some_and(|n| n.is_terminal())
        {
            return Err(RegionError::Incomplete);
        }
        let owner = region.owner_root();
        let count_slot = field(&region.proof, b"observed-cardinality")?;
        let count = count_slot.follow()?;
        let one = field(
            &field(&field(&owner, b"cardinalities")?, b"zero")?,
            b"successor",
        )?;
        let unknown = field(&field(&owner, b"class-tags")?, b"unknown")?;
        let seed = field(&region.proof, b"seed")?.follow()?;
        let color = field(&seed, b"value")?;
        let proofs = parent.provision(b"singleton-proofs");
        if proofs.child(id).is_some() {
            return Err(RegionError::Duplicate);
        }
        let root = proofs.provision(id);
        root.provision(b"region-proof").point_to(&region.proof);
        root.provision(b"source-occurrence").point_to(&seed);
        root.provision(b"producer").point_to(
            &parent
                .provision(b"singleton-constructors")
                .provision(b"observed-one-no-unknown-v1"),
        );
        let args = root.provision(b"arguments");
        args.provision(b"count-source").point_to(&count_slot);
        args.provision(b"count-reference").point_to(&count);
        args.provision(b"wanted-one").point_to(&one);
        args.provision(b"unknown-class").point_to(&unknown);
        let machine = Machine::default();
        let execution = root.provision(b"execution");
        execution
            .provision(b"machine")
            .publish_prepared_child(&machine.pointer());
        let construction = execution.provision(b"construction");
        let values = execution.provision(b"private-reference-views");
        let alias = |v: &Address| values.provision(&v.identity_key());
        let halt = execution.provision(b"halt");
        halt.terminate();
        let result = root.provision(b"status");
        let candidate = root.provision(b"candidate");
        let tags = parent.provision(b"singleton-tags");
        let emit = construction.provision(b"emit-source-color");
        let done = construction.provision(b"emit-singleton");
        let no = construction.provision(b"not-observed-one");
        let uncertain = construction.provision(b"unknown-frontier");
        copy(&emit, &color, &candidate, &done);
        copy(
            &done,
            &constant(&done, b"tag", &tags.provision(b"singleton")),
            &result,
            &halt,
        );
        copy(
            &no,
            &constant(&no, b"tag", &tags.provision(b"not-observed-one")),
            &result,
            &halt,
        );
        copy(
            &uncertain,
            &constant(&uncertain, b"tag", &tags.provision(b"unknown-frontier")),
            &result,
            &halt,
        );
        let facts = field(&region.proof, b"views")?.inspect_children();
        let mut next = emit;
        for (i, fact) in facts.iter().enumerate().rev() {
            let class_slot = field(fact, b"class")?;
            let class = class_slot.follow()?;
            let arg = args.provision(b"frontier").provision(&i.to_be_bytes());
            arg.provision(b"source-slot").point_to(&class_slot);
            arg.provision(b"bound-reference").point_to(&class);
            next = branch(
                &construction
                    .provision(b"check-frontier")
                    .provision(&i.to_be_bytes()),
                &alias(&class),
                &alias(&unknown),
                &uncertain,
                &next,
            );
        }
        let entry = branch(
            &construction.provision(b"check-count"),
            &alias(&count),
            &alias(&one),
            &next,
            &no,
        );
        machine.pointer().point_to(&entry);
        Ok(Self {
            machine,
            root,
            construction,
            budget: 2 * facts.len() + 4,
            halt,
            completed: false,
            failed: false,
            region,
            owner: parent.clone(),
        })
    }
    pub fn execute(&mut self, limit: usize) -> Result<u64, RegionError> {
        if self.completed || self.failed {
            return Err(RegionError::Incomplete);
        }
        for _ in 0..limit {
            if let Err(e) = self.machine.cycle() {
                self.failed = true;
                return Err(e.into());
            }
            if self.machine.current()?.same(&self.halt) {
                self.completed = true;
                return Ok(self.machine.cycles);
            }
        }
        Err(RegionError::Limit)
    }
    pub fn record(self) -> Result<SingletonProjection, RegionError> {
        self.root
            .provision(b"attempt-copies")
            .provision(&self.machine.cycles.to_be_bytes());
        if !self.completed {
            return Err(RegionError::Incomplete);
        }
        let status = field(&self.root, b"status")?.follow()?;
        let candidate = match field(&self.root, b"candidate")?.follow() {
            Ok(a) => Some(a),
            Err(Fault::Unset) => None,
            Err(e) => return Err(e.into()),
        };
        self.root.provision(b"committed").terminate();
        self.root
            .provision(b"copies")
            .provision(&self.machine.cycles.to_be_bytes());
        Ok(SingletonProjection {
            proof: self.root,
            status,
            candidate,
            copies: self.machine.cycles,
            _region: self.region,
            _owner: self.owner,
        })
    }
}
