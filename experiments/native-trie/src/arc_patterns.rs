//! Complete directed three-cell contexts, learned only from actual before/after occurrences.
//! No absolute coordinate in rule keys, palette decoder, pattern catalogue or host transform.
use crate::{
    Address,
    arc_colors::{Cell, Choice, ColorError, ColorModel, ColorRun},
    arc_grids::{Grid, GridError, Grids},
    records::{self, Selection},
};
#[derive(Clone, Copy)]
pub enum Axis {
    Horizontal,
    Vertical,
}
impl Axis {
    fn token(self) -> &'static [u8] {
        match self {
            Self::Horizontal => b"horizontal",
            Self::Vertical => b"vertical",
        }
    }
    fn ports(self) -> (&'static [u8], &'static [u8]) {
        match self {
            Self::Horizontal => (b"west", b"east"),
            Self::Vertical => (b"north", b"south"),
        }
    }
}
const AXES: [Axis; 2] = [Axis::Horizontal, Axis::Vertical];
pub struct PatternModel {
    pub base: ColorModel,
    outside: Address,
    statuses: [Address; 4],
}
fn field(at: &Address, key: &[u8]) -> Result<Address, ColorError> {
    Ok(at.child(key).ok_or(GridError::Malformed)?)
}
fn value(at: &Address) -> Result<Address, ColorError> {
    Ok(field(at, b"value")?.follow()?)
}
fn observe(
    cell: &Address,
    axis: Axis,
    outside: &Address,
) -> Result<Option<[Address; 3]>, ColorError> {
    let (before, after) = axis.ports();
    let before = field(cell, before)?.follow()?;
    let after = field(cell, after)?.follow()?;
    if before.same(outside) || after.same(outside) {
        return Ok(None);
    }
    Ok(Some([value(&before)?, value(cell)?, value(&after)?]))
}
fn anchor(root: &Address, axis: Axis, values: &[Address; 3]) -> Option<Address> {
    values
        .iter()
        .try_fold(root.child(b"contexts")?.child(axis.token())?, |at, v| {
            at.child(&v.identity_key())
        })
}
impl PatternModel {
    pub fn new(grids: &Grids, scope: &[u8]) -> Result<Self, ColorError> {
        let base = ColorModel::new(grids, scope)?;
        let status = base.root.provision(b"context-status");
        let statuses = [
            status.provision(b"incomplete"),
            status.provision(b"unseen"),
            status.provision(b"unique"),
            status.provision(b"ambiguous"),
        ];
        Ok(Self {
            base,
            outside: grids.outside(),
            statuses,
        })
    }
    pub fn observe(&self, cell: &Address, axis: Axis) -> Result<Option<[Address; 3]>, ColorError> {
        observe(cell, axis, &self.outside)
    }
    pub fn anchor(&self, axis: Axis, values: &[Address; 3]) -> Option<Address> {
        anchor(&self.base.root, axis, values)
    }
    pub fn teach(&self, id: &[u8], input: &Grid, output: &Grid) -> Result<(), ColorError> {
        self.base.teach(id, input, output)?;
        let episode = field(&field(&self.base.root, b"examples")?, id)?;
        for y in 0..input.height() {
            for x in 0..input.width() {
                let cell = input.cell(y, x)?;
                let target = output.cell(y, x)?;
                let out = value(&target)?;
                for axis in AXES {
                    if let Some(values) = self.observe(&cell, axis)? {
                        let mut at = self
                            .base
                            .root
                            .provision(b"contexts")
                            .provision(axis.token());
                        for v in &values {
                            at = at.provision(&v.identity_key());
                        }
                        at.provision(b"construction")
                            .point_to(&self.base.construction);
                        let evidence = episode
                            .provision(&y.to_be_bytes())
                            .provision(&x.to_be_bytes())
                            .provision(b"contexts")
                            .provision(axis.token());
                        evidence.provision(b"input-cell").point_to(&cell);
                        evidence.provision(b"output-cell").point_to(&target);
                        for (name, v) in [b"before".as_slice(), b"center", b"after"]
                            .into_iter()
                            .zip(values)
                        {
                            evidence.provision(name).point_to(&v)
                        }
                        evidence
                            .provision(b"construction")
                            .point_to(&self.base.construction);
                        records::record(&at, b"output", &out, &evidence)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn begin(&self, id: &[u8], input: &Grid) -> Result<ColorRun, ColorError> {
        let run = self.base.begin(id, input)?;
        for cell in &run.cells {
            for axis in AXES {
                let view = cell.node.provision(b"contexts").provision(axis.token());
                for key in [b"lookup".as_slice(), b"candidate", b"status"] {
                    view.provision(key);
                }
            }
        }
        Ok(run)
    }
    pub fn execute(&mut self, run: &mut ColorRun, limit: usize) -> Result<(), ColorError> {
        let root = self.base.root.clone();
        let outside = self.outside.clone();
        let statuses = self.statuses.clone();
        self.base
            .execute_with(run, limit, |cell| resolve(&root, &outside, &statuses, cell))
    }
    pub fn record(&self, grids: &mut Grids, run: ColorRun) -> Result<Grid, ColorError> {
        self.base.record(grids, run)
    }
}
fn resolve(
    root: &Address,
    outside: &Address,
    statuses: &[Address; 4],
    cell: &Cell,
) -> Result<Choice, ColorError> {
    let mut complete = 0;
    let mut choice = Choice::Unknown;
    for axis in AXES {
        let view = field(&field(&cell.node, b"contexts")?, axis.token())?;
        let (selected, status) = match observe(&cell.input, axis, outside)? {
            None => (Choice::Unknown, &statuses[0]),
            Some(values) => {
                complete += 1;
                match anchor(root, axis, &values) {
                    None => (Choice::Unknown, &statuses[1]),
                    Some(at) => {
                        field(&view, b"lookup")?.point_to(&at);
                        match records::resolve(&at, b"output")? {
                            Selection::Unknown => (Choice::Unknown, &statuses[1]),
                            Selection::Ambiguous { .. } => (Choice::Ambiguous, &statuses[3]),
                            Selection::Unique { candidate, target } => {
                                field(&view, b"candidate")?.point_to(&candidate);
                                (Choice::Unique { candidate, target }, &statuses[2])
                            }
                        }
                    }
                }
            }
        };
        field(&view, b"status")?.point_to(status);
        choice = choice.agree(selected);
    }
    if complete == 0 {
        let v = value(&cell.input)?;
        choice = match root
            .child(b"traits")
            .and_then(|a| a.child(b"color"))
            .and_then(|a| a.child(&v.identity_key()))
        {
            Some(at) => {
                cell.lookup.point_to(&at);
                records::resolve(&at, b"output")?.into()
            }
            None => Choice::Unknown,
        };
    } else if let Choice::Unique { candidate, .. } = &choice {
        cell.lookup
            .point_to(&field(candidate, b"anchor")?.follow()?);
    }
    Ok(choice)
}
