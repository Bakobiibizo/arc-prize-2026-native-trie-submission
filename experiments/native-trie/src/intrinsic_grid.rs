//! Output-independent intrinsic addresses for admitted ARC inputs.
//!
//! Encoding walks the supplied input topology once. Each cell receives canonical native
//! addresses for its structural neighbourhood and for the same structure refined by actual value
//! references. Outputs, task IDs, coordinates, and construction-family names are not inputs.
use crate::{
    Address, Trie,
    arc_grids::{Grid, GridError},
};

const PORTS: [&[u8]; 4] = [b"north", b"west", b"south", b"east"];

#[derive(Clone)]
pub struct CellIntrinsic {
    /// Palette-independent equality/boundary structure.
    pub structural: Address,
    /// Palette-independent shape of the four-connected equal-value component.
    pub component: Address,
    /// The structural address refined by actual native value identities.
    pub valued: Address,
}

pub struct IntrinsicGrid {
    pub anchor: Address,
    width: usize,
    cells: Vec<CellIntrinsic>,
}

impl IntrinsicGrid {
    pub fn cell(&self, y: usize, x: usize) -> Option<&CellIntrinsic> {
        (x < self.width).then(|| self.cells.get(y.checked_mul(self.width)?.checked_add(x)?))?
    }
}

#[derive(Default)]
pub struct IntrinsicEncoder {
    storage: Trie,
    retained: Vec<Address>,
}

impl IntrinsicEncoder {
    pub fn root(&self) -> Address {
        self.storage.root()
    }

    fn token_fact(&self, kind: &[u8], token: &[u8]) -> Address {
        self.storage
            .root()
            .provision(b"intrinsic-facts")
            .provision(kind)
            .provision(token)
    }

    fn components(&self, input: &Grid) -> Result<Vec<Address>, GridError> {
        let width = input.width();
        let height = input.height();
        let values = (0..height)
            .flat_map(|y| (0..width).map(move |x| (y, x)))
            .map(|(y, x)| {
                input
                    .cell(y, x)?
                    .child(b"value")
                    .ok_or(GridError::Malformed)?
                    .follow()
                    .map_err(GridError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut assigned = vec![None; values.len()];
        let mut visited = vec![false; values.len()];
        for seed in 0..values.len() {
            if visited[seed] {
                continue;
            }
            let mut pending = vec![seed];
            let mut members = Vec::new();
            visited[seed] = true;
            while let Some(index) = pending.pop() {
                members.push(index);
                let y = index / width;
                let x = index % width;
                let neighbours = [
                    y.checked_sub(1).map(|ny| ny * width + x),
                    x.checked_sub(1).map(|nx| y * width + nx),
                    (y + 1 < height).then_some((y + 1) * width + x),
                    (x + 1 < width).then_some(y * width + x + 1),
                ];
                for next in neighbours.into_iter().flatten() {
                    if !visited[next] && values[next].same(&values[seed]) {
                        visited[next] = true;
                        pending.push(next);
                    }
                }
            }
            let min_y = members.iter().map(|index| index / width).min().unwrap_or(0);
            let min_x = members.iter().map(|index| index % width).min().unwrap_or(0);
            let max_y = members
                .iter()
                .map(|index| index / width)
                .max()
                .unwrap_or(min_y);
            let max_x = members
                .iter()
                .map(|index| index % width)
                .max()
                .unwrap_or(min_x);
            let mut shape = self
                .storage
                .root()
                .provision(b"component-shape")
                .provision(&members.len().to_be_bytes())
                .provision(&(max_y - min_y + 1).to_be_bytes())
                .provision(&(max_x - min_x + 1).to_be_bytes());
            let mut relative = members
                .iter()
                .map(|index| (index / width - min_y, index % width - min_x))
                .collect::<Vec<_>>();
            relative.sort_unstable();
            for (y, x) in relative {
                shape = shape
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes());
            }
            for index in members {
                assigned[index] = Some(shape.clone());
            }
        }
        assigned
            .into_iter()
            .map(|address| address.ok_or(GridError::Malformed))
            .collect()
    }

    /// Encode only facts available from the input before any output or construction is examined.
    pub fn encode(
        &mut self,
        input: &Grid,
        outside: &Address,
        unknown: &Address,
    ) -> Result<IntrinsicGrid, GridError> {
        let occurrence = self
            .storage
            .root()
            .provision(b"encoded-inputs")
            .provision(&input.anchor.identity_key());
        occurrence.provision(b"input").point_to(&input.anchor);
        let components = self.components(input)?;
        let mut cells = Vec::with_capacity(input.width() * input.height());
        for y in 0..input.height() {
            for x in 0..input.width() {
                let source = input.cell(y, x)?;
                let center = source
                    .child(b"value")
                    .ok_or(GridError::Malformed)?
                    .follow()?;
                let mut structural = self.storage.root().provision(b"cell-structure");
                let mut valued = self
                    .storage
                    .root()
                    .provision(b"cell-values")
                    .provision(&center.identity_key());
                valued.provision(b"center-value").point_to(&center);
                for port in PORTS {
                    let neighbour = source.child(port).ok_or(GridError::Malformed)?.follow()?;
                    let value = neighbour
                        .child(b"value")
                        .map(|slot| slot.follow())
                        .transpose()?;
                    let relation = match value.as_ref() {
                        None if neighbour.same(outside) => self.token_fact(b"boundary", port),
                        None => return Err(GridError::Malformed),
                        Some(value) if value.same(unknown) => self.token_fact(b"unknown", port),
                        Some(value) if value.same(&center) => self.token_fact(b"equal", port),
                        Some(_) => self.token_fact(b"different", port),
                    };
                    structural = structural.provision(&relation.identity_key());
                    structural.provision(b"fact").point_to(&relation);
                    structural = structural.provision(b"next");
                    valued = valued.provision(&relation.identity_key());
                    valued.provision(b"relation").point_to(&relation);
                    if let Some(value) = value {
                        valued = valued.provision(&value.identity_key());
                        valued.provision(b"neighbour-value").point_to(&value);
                    }
                    valued = valued.provision(b"next");
                    self.retained.push(relation);
                }
                occurrence
                    .provision(b"cells")
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes())
                    .provision(b"structural")
                    .point_to(&structural);
                cells.push(CellIntrinsic {
                    structural,
                    component: components[y * input.width() + x].clone(),
                    valued,
                });
                self.retained.push(center);
            }
        }
        Ok(IntrinsicGrid {
            anchor: occurrence,
            width: input.width(),
            cells,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        arc_grids::{Grids, Side},
        construction_memory::ConstructionMemory,
    };

    fn admit(grids: &Grids, id: &[u8], cells: &[Vec<Option<u8>>]) -> Grid {
        let source = grids.source(id, id).unwrap();
        grids.admit(id, Side::QueryInput, cells, &source).unwrap()
    }

    #[test]
    fn equal_local_structure_has_one_address_across_position_and_palette() {
        let grids = Grids::new();
        let a = admit(
            &grids,
            b"a",
            &[
                vec![Some(0); 4],
                vec![Some(0), Some(1), Some(1), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let b = admit(
            &grids,
            b"b",
            &[
                vec![Some(0); 4],
                vec![Some(0); 4],
                vec![Some(0), Some(7), Some(7), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let mut encoder = IntrinsicEncoder::default();
        let ea = encoder
            .encode(&a, &grids.outside(), &grids.unresolved())
            .unwrap();
        let eb = encoder
            .encode(&b, &grids.outside(), &grids.unresolved())
            .unwrap();
        assert!(
            ea.cell(1, 1)
                .unwrap()
                .structural
                .same(&eb.cell(2, 1).unwrap().structural)
        );
        assert!(
            !ea.cell(1, 1)
                .unwrap()
                .valued
                .same(&eb.cell(2, 1).unwrap().valued)
        );
    }

    #[test]
    fn component_shape_is_shared_across_position_and_palette_but_not_topology() {
        let grids = Grids::new();
        let horizontal = admit(
            &grids,
            b"horizontal",
            &[
                vec![Some(0); 4],
                vec![Some(0), Some(1), Some(1), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let translated = admit(
            &grids,
            b"translated",
            &[
                vec![Some(0); 4],
                vec![Some(0); 4],
                vec![Some(0), Some(7), Some(7), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let vertical = admit(
            &grids,
            b"vertical",
            &[
                vec![Some(0); 4],
                vec![Some(0), Some(9), Some(0), Some(0)],
                vec![Some(0), Some(9), Some(0), Some(0)],
            ],
        );
        let mut encoder = IntrinsicEncoder::default();
        let a = encoder
            .encode(&horizontal, &grids.outside(), &grids.unresolved())
            .unwrap();
        let b = encoder
            .encode(&translated, &grids.outside(), &grids.unresolved())
            .unwrap();
        let c = encoder
            .encode(&vertical, &grids.outside(), &grids.unresolved())
            .unwrap();
        assert!(
            a.cell(1, 1)
                .unwrap()
                .component
                .same(&b.cell(2, 1).unwrap().component)
        );
        assert!(
            !a.cell(1, 1)
                .unwrap()
                .component
                .same(&c.cell(1, 1).unwrap().component)
        );
    }

    #[test]
    fn structural_input_address_reaches_construction_across_palette_and_position() {
        let grids = Grids::new();
        let a = admit(
            &grids,
            b"learn",
            &[
                vec![Some(0); 4],
                vec![Some(0), Some(1), Some(1), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let b = admit(
            &grids,
            b"query",
            &[
                vec![Some(0); 4],
                vec![Some(0); 4],
                vec![Some(0), Some(7), Some(7), Some(0)],
                vec![Some(0); 4],
            ],
        );
        let mut encoder = IntrinsicEncoder::default();
        let learned = encoder
            .encode(&a, &grids.outside(), &grids.unresolved())
            .unwrap();
        let query = encoder
            .encode(&b, &grids.outside(), &grids.unresolved())
            .unwrap();
        let programs = Trie::default();
        let construction = programs.provision(&[b"copy-east-value"]);
        let evidence = programs.provision(&[b"demonstration"]);
        let mut memory = ConstructionMemory::default();
        memory
            .record(
                std::slice::from_ref(&learned.cell(1, 1).unwrap().structural),
                &construction,
                &evidence,
            )
            .unwrap();
        assert!(
            memory
                .resolve(std::slice::from_ref(&query.cell(2, 1).unwrap().structural,))
                .unwrap()
                .same(&construction)
        );
    }

    #[test]
    fn boundary_and_unknown_are_intrinsic_and_distinct() {
        let grids = Grids::new();
        let grid = admit(
            &grids,
            b"grid",
            &[vec![Some(1), None], vec![Some(1), Some(2)]],
        );
        let mut encoder = IntrinsicEncoder::default();
        let encoded = encoder
            .encode(&grid, &grids.outside(), &grids.unresolved())
            .unwrap();
        assert!(
            !encoded
                .cell(0, 0)
                .unwrap()
                .structural
                .same(&encoded.cell(1, 0).unwrap().structural)
        );
        assert!(
            !encoded
                .cell(0, 0)
                .unwrap()
                .structural
                .same(&encoded.cell(0, 1).unwrap().structural)
        );
    }
}
