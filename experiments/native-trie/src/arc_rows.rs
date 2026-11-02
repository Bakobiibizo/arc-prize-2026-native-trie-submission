//! Explicit two-row reference-output curricula, not ARC task learning or operator search.
//! Mapping is consumed ONLY by teaching. Execution uses actual stored source/target/next fields.
use crate::{
    Address, Fault, Machine,
    arc_grids::{Grid, GridError, Grids},
    install,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowError {
    InvalidLesson,
    DuplicateScope,
    Grid(GridError),
    Machine(Fault),
    Limit,
    NotReady,
}
impl From<GridError> for RowError {
    fn from(value: GridError) -> Self {
        Self::Grid(value)
    }
}
pub struct RowProjection {
    pub grid: Grid,
    pub cycles: u64,
    pub construction: Address,
}
pub struct RowProgram {
    pub machine: Machine,
    pub construction: Address,
    owner: Address,
    bindings: [Address; 2],
    outputs: Vec<Address>,
    entry: Address,
    halt: Address,
    unproduced: Address,
    bound: Option<Grid>,
    completed: bool,
}
impl RowProgram {
    /// A literal symbolic lesson (swap=[1,0], repeat=[0,1,0,1]) from ARC curriculum.
    /// This finite generic loader teaches reference-copy instructions, not a host row operator.
    pub fn teach(
        grids: &Grids,
        scope: &[u8],
        output_slots: &[usize],
        source: &Address,
    ) -> Result<Self, RowError> {
        if output_slots.is_empty() || output_slots.len() > 30 || output_slots.iter().any(|i| *i > 1)
        {
            return Err(RowError::InvalidLesson);
        }
        if !grids.owns_source(source) {
            return Err(GridError::ForeignSpace.into());
        }
        let owner = grids.storage.root();
        let curricula = owner.provision(b"arc-row-curriculum");
        if curricula.child(scope).is_some() {
            return Err(RowError::DuplicateScope);
        }
        let construction = curricula.provision(scope);
        construction.provision(b"teaching-source").point_to(source);
        let bindings = [
            construction.provision(b"bindings").provision(b"0"),
            construction.provision(b"bindings").provision(b"1"),
        ];
        let halt = construction.provision(b"halt");
        halt.terminate();
        let unproduced = construction.provision(b"not-produced");
        let instructions: Vec<_> = (0..output_slots.len())
            .map(|i| {
                construction
                    .provision(b"instructions")
                    .provision(&i.to_be_bytes())
            })
            .collect();
        let outputs: Vec<_> = (0..output_slots.len())
            .map(|i| {
                construction
                    .provision(b"outputs")
                    .provision(&i.to_be_bytes())
            })
            .collect();
        for (i, slot) in output_slots.iter().enumerate() {
            let next = instructions[i].provision(b"continuation");
            next.point_to(instructions.get(i + 1).unwrap_or(&halt));
            install(&instructions[i], &bindings[*slot], &outputs[i], &next);
            construction
                .provision(b"literal-lesson")
                .provision(&i.to_be_bytes())
                .point_to(&bindings[*slot]);
        }
        let entry = instructions[0].clone();
        Ok(Self {
            machine: Machine::default(),
            construction,
            owner,
            bindings,
            outputs,
            entry,
            halt,
            unproduced,
            bound: None,
            completed: false,
        })
    }
    /// Sequential controller only; supplied-address plumbing, no transformation. Two rows
    /// is the declared lesson arity. A new bind invalidates any unrecorded previous result.
    pub fn bind(&mut self, input: &Grid) -> Result<(), RowError> {
        self.bound = None;
        self.completed = false;
        if !input.owner.same(&self.owner) {
            return Err(GridError::ForeignSpace.into());
        }
        if input.height() != 2 {
            return Err(GridError::InvalidShape.into());
        }
        let rows = [input.row(0)?, input.row(1)?];
        for (binding, row) in self.bindings.iter().zip(rows) {
            binding.point_to(&row)
        }
        for output in &self.outputs {
            output.point_to(&self.unproduced)
        }
        self.machine.enter(&self.entry);
        self.bound = Some(input.clone());
        Ok(())
    }
    /// Only the unchanged copy/follow cycle; no allocation, template traversal or row lookup.
    pub fn execute(&mut self, limit: usize) -> Result<u64, RowError> {
        if self.bound.is_none() || self.completed {
            return Err(RowError::NotReady);
        }
        for _ in 0..limit {
            self.machine.cycle().map_err(RowError::Machine)?;
            if self
                .machine
                .current()
                .map_err(RowError::Machine)?
                .same(&self.halt)
            {
                self.completed = true;
                return Ok(self.machine.cycles);
            }
        }
        Err(RowError::Limit)
    }
    /// Mechanical result materialization is separate from execution. No partial output is valid.
    pub fn record(&mut self, grids: &mut Grids) -> Result<RowProjection, RowError> {
        if !self.completed {
            return Err(RowError::NotReady);
        }
        if !self.owner.same(&grids.storage.root()) {
            return Err(GridError::ForeignSpace.into());
        }
        let input = self.bound.as_ref().ok_or(RowError::NotReady)?;
        let rows: Result<Vec<_>, _> = self.outputs.iter().map(Address::follow).collect();
        let grid = grids.derived(input, &self.construction, &rows.map_err(RowError::Machine)?)?;
        self.completed = false;
        self.bound = None;
        Ok(RowProjection {
            grid,
            cycles: self.machine.cycles,
            construction: self.construction.clone(),
        })
    }
    pub fn apply(
        &mut self,
        grids: &mut Grids,
        input: &Grid,
        limit: usize,
    ) -> Result<RowProjection, RowError> {
        self.bind(input)?;
        self.execute(limit)?;
        self.record(grids)
    }
}
