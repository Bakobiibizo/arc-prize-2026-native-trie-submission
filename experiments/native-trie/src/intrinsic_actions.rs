//! Output-delta constructions attached to input-derived intrinsic addresses.
//!
//! This initial vocabulary records preservation, a unique directional input-value copy, or an
//! evidenced literal native reference. It does not select or invoke a named solver family.
use crate::{
    Address, Trie,
    arc_grids::{Grid, GridError},
    construction_memory::{ConstructionMemory, RouteError},
    intrinsic_grid::{CellIntrinsic, IntrinsicEncoder, IntrinsicGrid},
};

const PORTS: [&[u8]; 4] = [b"north", b"west", b"south", b"east"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionError {
    Shape,
    Grid(GridError),
    Route(RouteError),
}
impl From<GridError> for ActionError {
    fn from(value: GridError) -> Self {
        Self::Grid(value)
    }
}
impl From<RouteError> for ActionError {
    fn from(value: RouteError) -> Self {
        Self::Route(value)
    }
}

#[derive(Default)]
pub struct IntrinsicActions {
    encoder: IntrinsicEncoder,
    memory: ConstructionMemory,
    actions: Trie,
}
impl IntrinsicActions {
    fn value(cell: &Address) -> Result<Address, GridError> {
        Ok(cell.child(b"value").ok_or(GridError::Malformed)?.follow()?)
    }

    fn action(
        &self,
        input_cell: &Address,
        output_cell: &Address,
        outside: &Address,
    ) -> Result<(Address, bool), GridError> {
        let before = Self::value(input_cell)?;
        let after = Self::value(output_cell)?;
        if before.same(&after) {
            return Ok((self.actions.root().provision(b"preserve"), false));
        }
        let mut matching = Vec::new();
        for port in PORTS {
            let neighbour = input_cell
                .child(port)
                .ok_or(GridError::Malformed)?
                .follow()?;
            if neighbour.same(outside) {
                continue;
            }
            if Self::value(&neighbour)?.same(&after) {
                matching.push(port);
            }
        }
        if let [port] = matching.as_slice() {
            return Ok((
                self.actions
                    .root()
                    .provision(b"copy-input-port")
                    .provision(port),
                false,
            ));
        }
        let literal = self
            .actions
            .root()
            .provision(b"emit-reference")
            .provision(&after.identity_key());
        literal.provision(b"value").point_to(&after);
        Ok((literal, true))
    }

    fn route(cell: &CellIntrinsic, refined: bool) -> Vec<Address> {
        if refined {
            vec![
                cell.structural.clone(),
                cell.component.clone(),
                cell.valued.clone(),
            ]
        } else {
            vec![cell.structural.clone()]
        }
    }

    /// Record demonstrated per-cell constructions. The route is input-only; the output supplies
    /// the construction and evidence stored at that route.
    pub fn observe(
        &mut self,
        input: &Grid,
        output: &Grid,
        outside: &Address,
        unknown: &Address,
    ) -> Result<IntrinsicGrid, ActionError> {
        if input.width() != output.width() || input.height() != output.height() {
            return Err(ActionError::Shape);
        }
        let encoded = self.encoder.encode(input, outside, unknown)?;
        for y in 0..input.height() {
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let after = output.cell(y, x)?;
                let (construction, refined) = self.action(&before, &after, outside)?;
                self.memory.record(
                    &Self::route(encoded.cell(y, x).ok_or(GridError::Malformed)?, refined),
                    &construction,
                    &after,
                )?;
            }
        }
        Ok(encoded)
    }

    pub fn encode_query(
        &mut self,
        input: &Grid,
        outside: &Address,
        unknown: &Address,
    ) -> Result<IntrinsicGrid, ActionError> {
        Ok(self.encoder.encode(input, outside, unknown)?)
    }

    /// Resolve by following the cell's structural-to-valued intrinsic route.
    pub fn resolve(&self, cell: &CellIntrinsic) -> Result<Address, ActionError> {
        Ok(self
            .memory
            .resolve_most_specific(&Self::route(cell, true))?)
    }

    pub fn applicable(
        &mut self,
        input: &Grid,
        outside: &Address,
        unknown: &Address,
    ) -> Result<bool, ActionError> {
        let encoded = self.encode_query(input, outside, unknown)?;
        for y in 0..input.height() {
            for x in 0..input.width() {
                if self
                    .resolve(encoded.cell(y, x).ok_or(GridError::Malformed)?)
                    .is_err()
                {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Render only constructions reached on each cell's intrinsic route. Missing or ambiguous
    /// cells abort the whole output rather than invoking another learner.
    pub fn render(
        &mut self,
        grids: &mut crate::arc_grids::Grids,
        input: &Grid,
        program: &Address,
        outside: &Address,
        unknown: &Address,
    ) -> Result<Grid, ActionError> {
        let encoded = self.encode_query(input, outside, unknown)?;
        let result = self
            .actions
            .root()
            .provision(b"renderings")
            .provision(&input.anchor.identity_key());
        let mut rows = Vec::with_capacity(input.height());
        for y in 0..input.height() {
            let row = result.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..input.width() {
                let construction = self.resolve(encoded.cell(y, x).ok_or(GridError::Malformed)?)?;
                let source = input.cell(y, x)?;
                let value = if let Some(slot) = construction.child(b"value") {
                    slot.follow().map_err(GridError::from)?
                } else {
                    match construction.token() {
                        Some(b"preserve") => Self::value(&source)?,
                        Some(port) if PORTS.contains(&port) => {
                            let neighbour = source
                                .child(port)
                                .ok_or(GridError::Malformed)?
                                .follow()
                                .map_err(GridError::from)?;
                            if neighbour.same(outside) {
                                return Err(ActionError::Grid(GridError::Malformed));
                            }
                            Self::value(&neighbour)?
                        }
                        _ => return Err(ActionError::Route(RouteError::Missing)),
                    }
                };
                let cell = row.provision(b"cells").provision(&x.to_be_bytes());
                cell.provision(b"value").point_to(&value);
                cell.provision(b"construction").point_to(&construction);
            }
            rows.push(row);
        }
        Ok(grids.derived(input, program, &rows)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc_grids::{Grids, Side};

    fn admit(grids: &Grids, id: &[u8], side: Side, cells: &[Vec<Option<u8>>]) -> Grid {
        let source = grids.source(id, id).unwrap();
        grids.admit(id, side, cells, &source).unwrap()
    }

    #[test]
    fn output_delta_is_stored_at_input_structure_and_reached_after_palette_transfer() {
        let grids = Grids::new();
        let input = admit(
            &grids,
            b"train-in",
            Side::TrainingInput,
            &[
                vec![Some(0); 4],
                vec![Some(0), Some(1), Some(1), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let output = admit(
            &grids,
            b"train-out",
            Side::TrainingOutput,
            &[
                vec![Some(0); 4],
                vec![Some(0), Some(1), Some(1), Some(1)],
                vec![Some(0); 4],
            ],
        );
        let query = admit(
            &grids,
            b"query",
            Side::QueryInput,
            &[
                vec![Some(0); 4],
                vec![Some(0); 4],
                vec![Some(0), Some(7), Some(7), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let mut model = IntrinsicActions::default();
        model
            .observe(&input, &output, &grids.outside(), &grids.unresolved())
            .unwrap();
        let encoded = model
            .encode_query(&query, &grids.outside(), &grids.unresolved())
            .unwrap();
        let construction = model.resolve(encoded.cell(2, 3).unwrap()).unwrap();
        assert_eq!(construction.token(), Some(b"west".as_slice()));
    }
}
