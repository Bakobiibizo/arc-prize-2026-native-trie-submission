//! ARC supplied-grid storage and serialization only. No solver, background/color policy,
//! structural matcher or transformation. Physical coordinates/adjacency are input wiring.
use crate::{Address, Fault, Trie};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    TrainingInput,
    TrainingOutput,
    QueryInput,
}
impl Side {
    fn token(self) -> &'static [u8] {
        match self {
            Self::TrainingInput => b"training-input",
            Self::TrainingOutput => b"training-output",
            Self::QueryInput => b"query-input",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridError {
    InvalidShape,
    InvalidColor,
    DuplicateOccurrence,
    ForeignSpace,
    Incomplete,
    Reference(Fault),
    Malformed,
}
impl From<Fault> for GridError {
    fn from(value: Fault) -> Self {
        Self::Reference(value)
    }
}
#[derive(Clone)]
pub struct Grid {
    pub anchor: Address,
    pub(crate) owner: Address,
    width: usize,
    height: usize,
    retained_owners: Vec<Address>,
}
impl Grid {
    pub(crate) fn retain_owners(mut self, owners: Vec<Address>) -> Self {
        self.retained_owners.extend(owners);
        self
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn row_cell(&self, index: usize) -> Option<Address> {
        self.anchor.child(b"rows")?.child(&index.to_be_bytes())
    }
    pub fn row(&self, index: usize) -> Result<Address, GridError> {
        Ok(self.row_cell(index).ok_or(GridError::Malformed)?.follow()?)
    }
    pub fn cell(&self, y: usize, x: usize) -> Result<Address, GridError> {
        self.row(y)?
            .child(b"cells")
            .and_then(|a| a.child(&x.to_be_bytes()))
            .ok_or(GridError::Malformed)
    }
    pub fn observed_source(&self) -> Result<Option<Address>, GridError> {
        self.anchor
            .child(b"observed-source")
            .map(|a| a.follow().map_err(GridError::from))
            .transpose()
    }
    /// Inspection only: render references already present in the result. Not called by execution.
    pub fn inspect(&self) -> Result<Vec<Vec<Option<u8>>>, GridError> {
        let unknown = self
            .owner
            .child(b"unresolved")
            .ok_or(GridError::Malformed)?;
        let mut output = Vec::with_capacity(self.height);
        for y in 0..self.height {
            let mut row = Vec::with_capacity(self.width);
            for x in 0..self.width {
                let value = self
                    .cell(y, x)?
                    .child(b"value")
                    .ok_or(GridError::Malformed)?
                    .follow()?;
                row.push(if value.same(&unknown) {
                    None
                } else {
                    let token = value.token().ok_or(GridError::Malformed)?;
                    if token.len() != 1 || token[0] > 9 {
                        return Err(GridError::Malformed);
                    }
                    Some(token[0])
                });
            }
            output.push(row);
        }
        Ok(output)
    }
}
pub struct Grids {
    pub(crate) storage: Trie,
    serial: usize,
}
impl Default for Grids {
    fn default() -> Self {
        Self::new()
    }
}
impl Grids {
    pub fn new() -> Self {
        let storage = Trie::default();
        let root = storage.root();
        root.provision(b"outside");
        root.provision(b"unresolved");
        Self { storage, serial: 0 }
    }
    pub fn outside(&self) -> Address {
        self.storage.root().child(b"outside").unwrap()
    }
    pub fn unresolved(&self) -> Address {
        self.storage.root().child(b"unresolved").unwrap()
    }
    /// Supplied raw source bytes, not decoded or reconstructed from the admitted value.
    pub fn source(&self, id: &[u8], raw: &[u8]) -> Result<Address, GridError> {
        let sources = self.storage.root().provision(b"sources");
        if sources.child(id).is_some() {
            return Err(GridError::DuplicateOccurrence);
        }
        let source = sources.provision(id);
        source.provision(b"raw-bytes").provision(raw);
        Ok(source)
    }
    pub(crate) fn owns_source(&self, source: &Address) -> bool {
        source
            .token()
            .and_then(|id| self.storage.root().child(b"sources")?.child(id))
            .is_some_and(|at| at.same(source))
    }
    pub fn admit(
        &self,
        id: &[u8],
        side: Side,
        cells: &[Vec<Option<u8>>],
        source: &Address,
    ) -> Result<Grid, GridError> {
        if !self.owns_source(source) {
            return Err(GridError::ForeignSpace);
        }
        let height = cells.len();
        let width = cells.first().map_or(0, Vec::len);
        if height == 0
            || height > 30
            || width == 0
            || width > 30
            || cells.iter().any(|r| r.len() != width)
        {
            return Err(GridError::InvalidShape);
        }
        if cells.iter().flatten().flatten().any(|v| *v > 9) {
            return Err(GridError::InvalidColor);
        }
        let namespace = self
            .storage
            .root()
            .provision(b"arc-grid")
            .provision(b"observed")
            .provision(side.token());
        if namespace.child(id).is_some() {
            return Err(GridError::DuplicateOccurrence);
        }
        let anchor = namespace.provision(id);
        anchor.provision(b"observed-source").point_to(source);
        let row_links = anchor.provision(b"rows");
        let mut positions = Vec::with_capacity(height);
        for (y, supplied_row) in cells.iter().enumerate() {
            let row = anchor.provision(b"row-data").provision(&y.to_be_bytes());
            row.provision(b"source-grid").point_to(&anchor);
            row_links.provision(&y.to_be_bytes()).point_to(&row);
            let mut positions_row = Vec::with_capacity(width);
            for (x, supplied) in supplied_row.iter().enumerate() {
                let cell = row.provision(b"cells").provision(&x.to_be_bytes());
                cell.provision(b"source-row").point_to(&row);
                let value = match supplied {
                    Some(color) => self
                        .storage
                        .root()
                        .provision(b"arc-color")
                        .provision(&[*color]),
                    None => self.unresolved(),
                };
                cell.provision(b"value").point_to(&value);
                positions_row.push(cell);
            }
            positions.push(positions_row);
        }
        // Only supplied rectangular topology; not segmentation, neighbors-as-roles, or a transform.
        for y in 0..height {
            for x in 0..width {
                for (port, at) in [
                    (b"north".as_slice(), y.checked_sub(1).map(|ny| (ny, x))),
                    (b"west".as_slice(), x.checked_sub(1).map(|nx| (y, nx))),
                    (b"south".as_slice(), (y + 1 < height).then_some((y + 1, x))),
                    (b"east".as_slice(), (x + 1 < width).then_some((y, x + 1))),
                ] {
                    positions[y][x].provision(port).point_to(
                        &at.map_or_else(|| self.outside(), |(ny, nx)| positions[ny][nx].clone()),
                    );
                }
            }
        }
        Ok(Grid {
            anchor,
            owner: self.storage.root(),
            width,
            height,
            retained_owners: Vec::new(),
        })
    }
    /// Recording an already computed result, preserving linked original row/cell occurrences.
    /// `rows` is VM output order, never a template evaluated by this recorder.
    pub(crate) fn derived(
        &mut self,
        input: &Grid,
        construction: &Address,
        rows: &[Address],
    ) -> Result<Grid, GridError> {
        self.derived_extent(input, construction, rows, input.width)
    }

    pub(crate) fn derived_extent(
        &mut self,
        input: &Grid,
        construction: &Address,
        rows: &[Address],
        width: usize,
    ) -> Result<Grid, GridError> {
        if !input.owner.same(&self.storage.root()) {
            return Err(GridError::ForeignSpace);
        }
        if rows.is_empty() || rows.len() > 30 || width == 0 || width > 30 {
            return Err(GridError::InvalidShape);
        }
        for row in rows {
            if row.child(b"cells").is_none() {
                return Err(GridError::Incomplete);
            }
        }
        // Validate every source reference before publishing any derived occurrence.
        let mut values = Vec::new();
        for row in rows {
            let mut supplied = Vec::new();
            for x in 0..width {
                let cell = row
                    .child(b"cells")
                    .and_then(|a| a.child(&x.to_be_bytes()))
                    .ok_or(GridError::Malformed)?;
                let value = cell.child(b"value").ok_or(GridError::Malformed)?.follow()?;
                supplied.push((cell, value));
            }
            values.push(supplied);
        }
        let anchor = self
            .storage
            .root()
            .provision(b"arc-grid")
            .provision(b"derived")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        anchor.provision(b"input").point_to(&input.anchor);
        anchor.provision(b"construction").point_to(construction);
        let mut positions = Vec::new();
        for (y, (original, supplied)) in rows.iter().zip(values).enumerate() {
            let row = anchor.provision(b"row-data").provision(&y.to_be_bytes());
            row.provision(b"derived-from").point_to(original);
            anchor
                .provision(b"rows")
                .provision(&y.to_be_bytes())
                .point_to(&row);
            let mut line = Vec::new();
            for (x, (original, value)) in supplied.into_iter().enumerate() {
                let cell = row.provision(b"cells").provision(&x.to_be_bytes());
                cell.provision(b"derived-from").point_to(&original);
                cell.provision(b"value").point_to(&value);
                line.push(cell);
            }
            positions.push(line);
        }
        for y in 0..rows.len() {
            for x in 0..width {
                for (port, at) in [
                    (b"north".as_slice(), y.checked_sub(1).map(|ny| (ny, x))),
                    (b"west".as_slice(), x.checked_sub(1).map(|nx| (y, nx))),
                    (
                        b"south".as_slice(),
                        (y + 1 < rows.len()).then_some((y + 1, x)),
                    ),
                    (b"east".as_slice(), (x + 1 < width).then_some((y, x + 1))),
                ] {
                    positions[y][x].provision(port).point_to(
                        &at.map_or_else(|| self.outside(), |(ny, nx)| positions[ny][nx].clone()),
                    );
                }
            }
        }
        Ok(Grid {
            anchor,
            owner: self.storage.root(),
            width,
            height: rows.len(),
            retained_owners: input.retained_owners.clone(),
        })
    }
}
