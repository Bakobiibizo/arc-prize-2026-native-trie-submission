use super::*;

type StructuralEntry = (isize, isize, usize);
type ComponentSignature = (Vec<StructuralEntry>, Vec<Address>, (usize, usize));

#[derive(Clone, Copy)]
enum CanonicalWholeGridTransform {
    Dihedral(u8),
    Scale(usize, usize),
    Tile(usize, usize),
    CropMajority,
}

#[derive(Clone)]
enum CanonicalOverwrite {
    Replace,
    IfCurrentMatches(Address),
}

#[derive(Clone)]
struct CanonicalMaskLayer {
    priority: usize,
    mask: Vec<usize>,
    value: Address,
    overwrite: CanonicalOverwrite,
}

#[derive(Clone, Copy)]
enum CanonicalAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
struct CanonicalCardinalSegment {
    priority: usize,
    start: (usize, usize),
    end: (usize, usize),
    value: Address,
    overwrite: CanonicalOverwrite,
}

#[derive(Clone)]
struct CanonicalAxisProjection {
    priority: usize,
    source: (usize, usize),
    axes: Vec<CanonicalAxis>,
    value: Address,
    overwrite: CanonicalOverwrite,
}

#[derive(Clone, Copy)]
enum CanonicalStencilBounds {
    Reject,
    Clip,
}

#[derive(Clone, Copy)]
enum CanonicalStencilOverlap {
    Ordered,
    RejectDifferentValues,
}

#[derive(Clone)]
struct CanonicalStencilPlacement {
    priority: usize,
    anchor: (isize, isize),
    cells: Vec<(isize, isize, Address)>,
    overwrite: CanonicalOverwrite,
}

#[derive(Clone)]
struct CanonicalRoleTransform {
    priority: usize,
    dihedral: usize,
    translation: (isize, isize),
    cells: Vec<(isize, isize, Address)>,
    overwrite: CanonicalOverwrite,
}

#[derive(Clone)]
struct CanonicalTransformedRoleCell {
    source: usize,
    target: usize,
    value: Address,
}

#[derive(Clone)]
struct CanonicalRoleTransformProjection {
    cells: Vec<CanonicalTransformedRoleCell>,
    overwrite: CanonicalOverwrite,
}

#[derive(Clone, Copy)]
struct CanonicalPeriodicAxis {
    lower: isize,
    upper: isize,
    phase: isize,
    step: usize,
}

#[derive(Clone, Copy)]
struct CanonicalPeriodicLattice {
    rows: CanonicalPeriodicAxis,
    columns: CanonicalPeriodicAxis,
}

struct CanonicalPeriodicLatticeSemantics<'a> {
    provenance: &'a [u8],
    relation: &'a [u8],
    origin: &'a [u8],
    period: &'a [u8],
    bounds: &'a [u8],
    masks: &'a [u8],
    composition: &'a [u8],
    overwrite: &'a [u8],
}

#[derive(Clone, Copy)]
struct CanonicalBoundingEnvelope {
    top: usize,
    bottom: usize,
    left: usize,
    right: usize,
}

impl CanonicalBoundingEnvelope {
    fn from_points(points: impl IntoIterator<Item = (usize, usize)>) -> Option<Self> {
        let mut points = points.into_iter();
        let (y, x) = points.next()?;
        let mut envelope = Self {
            top: y,
            bottom: y,
            left: x,
            right: x,
        };
        for (y, x) in points {
            envelope.top = envelope.top.min(y);
            envelope.bottom = envelope.bottom.max(y);
            envelope.left = envelope.left.min(x);
            envelope.right = envelope.right.max(x);
        }
        Some(envelope)
    }

    fn corners(self) -> [(usize, usize); 4] {
        [
            (self.top, self.left),
            (self.top, self.right),
            (self.bottom, self.left),
            (self.bottom, self.right),
        ]
    }

    fn integral_center(self) -> Option<(usize, usize)> {
        let y = self.top.checked_add(self.bottom)?;
        let x = self.left.checked_add(self.right)?;
        ((y % 2 == 0) && (x % 2 == 0)).then_some((y / 2, x / 2))
    }

    fn maximum_span(self) -> usize {
        self.bottom
            .abs_diff(self.top)
            .max(self.right.abs_diff(self.left))
    }
}

struct CanonicalBoundingEnvelopeSemantics<'a> {
    provenance: &'a [u8],
    relation: &'a [u8],
    source_points: &'a [u8],
    derived_geometry: &'a [u8],
    masks: &'a [u8],
    composition: &'a [u8],
    overwrite: &'a [u8],
}

#[derive(Clone)]
struct Progression {
    background: Address,
    source: Address,
    target: Address,
    reverse: bool,
}

#[derive(Clone)]
struct Connector {
    background: Address,
    anchor: Address,
    first: Address,
    second: Address,
    target: Address,
    first_horizontal: bool,
    second_horizontal: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BeamMode {
    None,
    Horizontal,
    Vertical,
}

#[derive(Clone)]
struct Beams {
    background: Address,
    roles: Vec<(Address, BeamMode)>,
}

#[derive(Clone)]
struct MoveToTouch {
    background: Address,
    moving: Address,
    anchor: Address,
}

#[derive(Clone)]
struct SparseLattice {
    background: Address,
    step: usize,
}

#[derive(Clone)]
struct AlternatingStripes {
    background: Address,
}

#[derive(Clone)]
struct BoundaryRays {
    background: Address,
}

#[derive(Clone)]
struct ComponentStencil {
    background: Address,
    template: Vec<StructuralEntry>,
    effects: Vec<StructuralEntry>,
}

#[derive(Clone)]
struct PairedLines {
    background: Address,
}

#[derive(Clone)]
struct ComponentPairFill {
    background: Address,
}

#[derive(Clone)]
struct OrdinalColumnFill {
    background: Address,
}

#[derive(Clone)]
struct NearestMarkerEnvelope {
    background: Address,
}

#[derive(Clone)]
struct LatticeSeparators {
    background: Address,
    inner: Address,
    outer: Address,
}

#[derive(Clone)]
struct CornerHalo {
    background: Address,
    target: Address,
}

#[derive(Clone)]
struct CornerPixels {
    background: Address,
    convex: Address,
    concave: Address,
}

#[derive(Clone)]
struct PeriodicDiamonds {
    background: Address,
    period: usize,
}

#[derive(Clone)]
struct GlyphClassifier {
    background: Address,
    mapping: Vec<(Vec<u8>, Address)>,
}

#[derive(Clone)]
struct ReflectInternalMotif {
    background: Address,
    frame: Address,
    motif: Address,
}

#[derive(Clone)]
struct MarkerCrosses {
    background: Address,
    marker: Address,
    center: Address,
    diagonal: Address,
    complete_diagonal_neighborhoods: bool,
}

#[derive(Clone)]
struct ArrowRays {
    background: Address,
    empty: Address,
}

#[derive(Clone)]
struct LegendBlocks {
    background: Address,
    carrier: Address,
}

#[derive(Clone)]
struct TemplateRays {
    background: Address,
}

#[derive(Clone)]
struct RelocateDiagrams {
    background: Address,
}

#[derive(Clone)]
struct AppendMarkerPeriod {
    background: Address,
    marker: Address,
}

#[derive(Clone)]
struct AdaptiveInset {
    background: Address,
    target: Address,
}

#[derive(Clone)]
struct CenterMarkerTransfer {
    empty: Address,
    frame: Address,
    marker: Address,
    target: Address,
}

#[derive(Clone)]
struct TiledModeDenoise {
    background: Address,
}

#[derive(Clone)]
struct AxisProfileProjection {
    empty: Address,
    object: Address,
    profile: Address,
    separator: Address,
    target: Address,
}

#[derive(Clone)]
struct SeparatorTemplate {
    empty: Address,
    regular: Address,
    separator: Address,
}

#[derive(Clone)]
struct MissingCornerPaths {
    background: Address,
    marker: Address,
    path: Address,
}

#[derive(Clone)]
struct PartialCrosses {
    empty: Address,
    noise: Address,
    marker: Address,
    target: Address,
}

#[derive(Clone)]
struct MarkerNetwork {
    background: Address,
    marker: Address,
}

#[derive(Clone)]
struct SquarePartition {
    background: Address,
    source: Address,
    block: Address,
    remainder: Address,
    examples: Vec<([usize; 7], bool)>,
}

impl SparseRewrite {
    fn record_canonical_mask_composition(
        program: &Address,
        provenance: &[u8],
        relation: &[u8],
        geometry: &[u8],
        overwrite: &[u8],
    ) {
        let canonical = program.provision(b"canonical-mask-composition");
        canonical
            .provision(b"roles")
            .provision(b"source-bound-colours")
            .terminate();
        canonical
            .provision(b"relations")
            .provision(relation)
            .terminate();
        canonical
            .provision(b"geometry")
            .provision(geometry)
            .terminate();
        canonical
            .provision(b"masks")
            .provision(b"query-derived-cell-sets")
            .terminate();
        let composition = canonical.provision(b"composition");
        composition.provision(b"input-snapshot").terminate();
        composition
            .provision(b"ascending-explicit-priority")
            .terminate();
        canonical
            .provision(b"overwrite-priority")
            .provision(overwrite)
            .terminate();
        canonical
            .provision(b"old-schema-provenance")
            .provision(provenance)
            .terminate();
    }

    fn record_canonical_cardinal_segments(
        program: &Address,
        provenance: &[u8],
        relation: &[u8],
        composition: &[u8],
        overwrite: &[u8],
    ) {
        let canonical = program.provision(b"canonical-cardinal-segment-placement");
        canonical
            .provision(b"roles")
            .provision(b"source-bound-colours")
            .terminate();
        canonical
            .provision(b"relations")
            .provision(relation)
            .terminate();
        canonical
            .provision(b"geometry")
            .provision(b"inclusive-cardinal-segments-between-bound-points")
            .terminate();
        canonical
            .provision(b"masks")
            .provision(b"query-derived-segment-cell-sets")
            .terminate();
        canonical
            .provision(b"composition")
            .provision(composition)
            .terminate();
        canonical
            .provision(b"overwrite-priority")
            .provision(overwrite)
            .terminate();
        canonical
            .provision(b"old-schema-provenance")
            .provision(provenance)
            .terminate();
    }

    fn record_canonical_relative_stencil(
        program: &Address,
        provenance: &[u8],
        relation: &[u8],
        template: &[u8],
        anchors: &[u8],
        bounds: &[u8],
        overlap: &[u8],
    ) {
        let canonical = program.provision(b"canonical-relative-stencil-placement");
        canonical
            .provision(b"roles")
            .provision(b"source-bound-role-values")
            .terminate();
        canonical
            .provision(b"relations")
            .provision(relation)
            .terminate();
        canonical
            .provision(b"geometry")
            .provision(b"normalized-relative-cell-offsets-at-derived-anchors")
            .terminate();
        canonical
            .provision(b"template")
            .provision(template)
            .terminate();
        canonical
            .provision(b"placements")
            .provision(anchors)
            .terminate();
        canonical.provision(b"bounds").provision(bounds).terminate();
        canonical
            .provision(b"masks")
            .provision(b"query-derived-stencil-cell-sets")
            .terminate();
        canonical
            .provision(b"composition")
            .provision(b"input-snapshot-canonical-mask-layers")
            .terminate();
        canonical
            .provision(b"overlap-priority")
            .provision(overlap)
            .terminate();
        canonical
            .provision(b"old-schema-provenance")
            .provision(provenance)
            .terminate();
    }

    fn record_canonical_role_transform(
        program: &Address,
        provenance: &[u8],
        relation: &[u8],
        source: &[u8],
        transform: &[u8],
        composition: &[u8],
        overwrite: &[u8],
    ) {
        let canonical = program.provision(b"canonical-dihedral-translation-role-transform");
        canonical
            .provision(b"roles")
            .provision(b"source-bound-role-values")
            .terminate();
        canonical
            .provision(b"relations")
            .provision(relation)
            .terminate();
        canonical
            .provision(b"source-cell-set")
            .provision(source)
            .terminate();
        canonical
            .provision(b"geometry")
            .provision(b"dihedral-cell-transform-plus-translation")
            .terminate();
        canonical
            .provision(b"transform-selection")
            .provision(transform)
            .terminate();
        canonical
            .provision(b"ambiguity")
            .provision(b"require-unique-admissible-transformed-cell-set")
            .terminate();
        canonical
            .provision(b"masks")
            .provision(b"query-derived-transformed-cell-sets")
            .terminate();
        canonical
            .provision(b"composition")
            .provision(composition)
            .terminate();
        canonical
            .provision(b"overwrite-priority")
            .provision(overwrite)
            .terminate();
        canonical
            .provision(b"old-schema-provenance")
            .provision(provenance)
            .terminate();
    }

    fn record_canonical_periodic_lattice(
        program: &Address,
        semantics: CanonicalPeriodicLatticeSemantics<'_>,
    ) {
        let canonical = program.provision(b"canonical-periodic-lattice-coordinate-generator");
        canonical
            .provision(b"roles")
            .provision(b"explicit-phase-origin-and-period-bindings")
            .terminate();
        canonical
            .provision(b"relations")
            .provision(semantics.relation)
            .terminate();
        canonical
            .provision(b"geometry")
            .provision(b"bounded-two-axis-congruence-cartesian-product")
            .terminate();
        canonical
            .provision(b"phase-origin")
            .provision(semantics.origin)
            .terminate();
        canonical
            .provision(b"period-binding")
            .provision(semantics.period)
            .terminate();
        canonical
            .provision(b"bounds")
            .provision(semantics.bounds)
            .terminate();
        canonical
            .provision(b"ordering")
            .provision(b"ascending-row-major")
            .terminate();
        canonical
            .provision(b"masks")
            .provision(semantics.masks)
            .terminate();
        canonical
            .provision(b"composition")
            .provision(semantics.composition)
            .terminate();
        canonical
            .provision(b"overwrite-priority")
            .provision(semantics.overwrite)
            .terminate();
        canonical
            .provision(b"old-schema-provenance")
            .provision(semantics.provenance)
            .terminate();
    }

    fn record_canonical_bounding_envelope(
        program: &Address,
        semantics: CanonicalBoundingEnvelopeSemantics<'_>,
    ) {
        let canonical = program.provision(b"canonical-bounding-envelope-corner-geometry");
        canonical
            .provision(b"roles")
            .provision(b"source-bound-point-selection")
            .terminate();
        canonical
            .provision(b"relations")
            .provision(semantics.relation)
            .terminate();
        canonical
            .provision(b"source-points")
            .provision(semantics.source_points)
            .terminate();
        canonical
            .provision(b"geometry")
            .provision(b"inclusive-axis-aligned-minimum-maximum-envelope")
            .terminate();
        canonical
            .provision(b"corner-order")
            .provision(b"northwest-northeast-southwest-southeast")
            .terminate();
        canonical
            .provision(b"derived-geometry")
            .provision(semantics.derived_geometry)
            .terminate();
        canonical
            .provision(b"bounds")
            .provision(b"query-grid-bounded-source-coordinates")
            .terminate();
        canonical
            .provision(b"masks")
            .provision(semantics.masks)
            .terminate();
        canonical
            .provision(b"composition")
            .provision(semantics.composition)
            .terminate();
        canonical
            .provision(b"overwrite-priority")
            .provision(semantics.overwrite)
            .terminate();
        canonical
            .provision(b"old-schema-provenance")
            .provision(semantics.provenance)
            .terminate();
    }

    fn record_canonical_axis_projection(
        program: &Address,
        provenance: &[u8],
        relation: &[u8],
        axes: &[u8],
        overlays: &[u8],
        overwrite: &[u8],
    ) {
        let canonical = program.provision(b"canonical-axis-projection");
        canonical
            .provision(b"roles")
            .provision(b"source-bound-colours")
            .terminate();
        canonical
            .provision(b"relations")
            .provision(relation)
            .terminate();
        canonical
            .provision(b"geometry")
            .provision(b"complete-axis-through-source-points")
            .terminate();
        canonical.provision(b"axes").provision(axes).terminate();
        canonical
            .provision(b"masks")
            .provision(b"query-derived-axis-cell-sets")
            .terminate();
        canonical
            .provision(b"composition")
            .provision(b"input-snapshot-canonical-mask-layers")
            .terminate();
        canonical
            .provision(b"overlays")
            .provision(overlays)
            .terminate();
        canonical
            .provision(b"overwrite-priority")
            .provision(overwrite)
            .terminate();
        canonical
            .provision(b"old-schema-provenance")
            .provision(provenance)
            .terminate();
    }

    fn render_whole_grid_transform(
        height: usize,
        width: usize,
        input: &[Address],
        transform: CanonicalWholeGridTransform,
    ) -> Option<(usize, usize, Vec<Address>)> {
        match transform {
            CanonicalWholeGridTransform::Dihedral(kind) => {
                let (out_h, out_w) = if matches!(kind, 1 | 3 | 6 | 7) {
                    (width, height)
                } else {
                    (height, width)
                };
                let mut output = Vec::with_capacity(out_h * out_w);
                for y in 0..out_h {
                    for x in 0..out_w {
                        let (source_y, source_x) = match kind {
                            0 => (y, x),
                            1 => (height - 1 - x, y),
                            2 => (height - 1 - y, width - 1 - x),
                            3 => (x, width - 1 - y),
                            4 => (y, width - 1 - x),
                            5 => (height - 1 - y, x),
                            6 => (x, y),
                            7 => (height - 1 - x, width - 1 - y),
                            _ => return None,
                        };
                        output.push(input[source_y * width + source_x].clone());
                    }
                }
                Some((out_h, out_w, output))
            }
            CanonicalWholeGridTransform::Scale(scale_y, scale_x) => {
                if scale_y == 0 || scale_x == 0 {
                    return None;
                }
                let (out_h, out_w) = (height * scale_y, width * scale_x);
                let output = (0..out_h)
                    .flat_map(|y| {
                        (0..out_w).map(move |x| input[(y / scale_y) * width + x / scale_x].clone())
                    })
                    .collect();
                Some((out_h, out_w, output))
            }
            CanonicalWholeGridTransform::Tile(repeat_y, repeat_x) => {
                if repeat_y == 0 || repeat_x == 0 {
                    return None;
                }
                let (out_h, out_w) = (height * repeat_y, width * repeat_x);
                let output = (0..out_h)
                    .flat_map(|y| {
                        (0..out_w).map(move |x| input[(y % height) * width + x % width].clone())
                    })
                    .collect();
                Some((out_h, out_w, output))
            }
            CanonicalWholeGridTransform::CropMajority => {
                let background = Self::background(input)?;
                let points = input
                    .iter()
                    .enumerate()
                    .filter(|(_, value)| !value.same(&background))
                    .map(|(index, _)| (index / width, index % width))
                    .collect::<Vec<_>>();
                let min_y = points.iter().map(|(y, _)| *y).min()?;
                let max_y = points.iter().map(|(y, _)| *y).max()?;
                let min_x = points.iter().map(|(_, x)| *x).min()?;
                let max_x = points.iter().map(|(_, x)| *x).max()?;
                let (out_h, out_w) = (max_y - min_y + 1, max_x - min_x + 1);
                let output = (min_y..=max_y)
                    .flat_map(|y| (min_x..=max_x).map(move |x| input[y * width + x].clone()))
                    .collect();
                Some((out_h, out_w, output))
            }
        }
    }

    fn render_separator_compaction(
        height: usize,
        width: usize,
        input: &[Address],
    ) -> Option<(usize, usize, Vec<Address>)> {
        let mut roles: Vec<Address> = Vec::new();
        for value in input {
            if roles.iter().any(|seen| seen.same(value)) {
                continue;
            }
            if (0..height).any(|y| (0..width).all(|x| input[y * width + x].same(value)))
                || (0..width).any(|x| (0..height).all(|y| input[y * width + x].same(value)))
            {
                roles.push(value.clone());
            }
        }
        if roles.len() != 1 {
            return None;
        }
        let separator = &roles[0];
        let rows = (0..height)
            .filter(|y| !(0..width).all(|x| input[y * width + x].same(separator)))
            .collect::<Vec<_>>();
        let columns = (0..width)
            .filter(|x| !(0..height).all(|y| input[y * width + x].same(separator)))
            .collect::<Vec<_>>();
        if rows.is_empty() || columns.is_empty() || (rows.len() == height && columns.len() == width)
        {
            return None;
        }
        let output = rows
            .iter()
            .flat_map(|y| columns.iter().map(move |x| input[*y * width + *x].clone()))
            .collect();
        Some((rows.len(), columns.len(), output))
    }

    fn render_block_reduction(
        height: usize,
        width: usize,
        input: &[Address],
        block_y: usize,
        block_x: usize,
        uniform: bool,
    ) -> Option<(usize, usize, Vec<Address>)> {
        if block_y == 0
            || block_x == 0
            || !height.is_multiple_of(block_y)
            || !width.is_multiple_of(block_x)
        {
            return None;
        }
        let (out_h, out_w) = (height / block_y, width / block_x);
        let mut output = Vec::with_capacity(out_h * out_w);
        for oy in 0..out_h {
            for ox in 0..out_w {
                let mut counts: Vec<(Address, usize)> = Vec::new();
                for dy in 0..block_y {
                    for dx in 0..block_x {
                        let value = &input[(oy * block_y + dy) * width + ox * block_x + dx];
                        if let Some((_, count)) =
                            counts.iter_mut().find(|(seen, _)| seen.same(value))
                        {
                            *count += 1;
                        } else {
                            counts.push((value.clone(), 1));
                        }
                    }
                }
                if uniform && counts.len() != 1 {
                    return None;
                }
                let maximum = counts.iter().map(|(_, count)| *count).max()?;
                let winners = counts
                    .iter()
                    .filter(|(_, count)| *count == maximum)
                    .collect::<Vec<_>>();
                if winners.len() != 1 {
                    return None;
                }
                output.push(winners[0].0.clone());
            }
        }
        Some((out_h, out_w, output))
    }

    fn render_object_extraction(
        height: usize,
        width: usize,
        input: &[Address],
        background: &Address,
        diagonal: bool,
        smallest: bool,
    ) -> Option<(usize, usize, Vec<Address>)> {
        let components = if diagonal {
            Self::same_color_components8(height, width, input, background)
        } else {
            Self::same_color_components(height, width, input, background)
        };
        let wanted = if smallest {
            components.iter().map(|(_, cells)| cells.len()).min()?
        } else {
            components.iter().map(|(_, cells)| cells.len()).max()?
        };
        let selected = components
            .iter()
            .filter(|(_, cells)| cells.len() == wanted)
            .collect::<Vec<_>>();
        if selected.len() != 1 {
            return None;
        }
        let (_, cells) = selected[0];
        let min_y = cells.iter().map(|(y, _)| *y).min()?;
        let max_y = cells.iter().map(|(y, _)| *y).max()?;
        let min_x = cells.iter().map(|(_, x)| *x).min()?;
        let max_x = cells.iter().map(|(_, x)| *x).max()?;
        let (out_h, out_w) = (max_y - min_y + 1, max_x - min_x + 1);
        let output = (min_y..=max_y)
            .flat_map(|y| {
                (min_x..=max_x).map(move |x| {
                    if cells.contains(&(y, x)) {
                        input[y * width + x].clone()
                    } else {
                        background.clone()
                    }
                })
            })
            .collect();
        Some((out_h, out_w, output))
    }

    pub fn agree_canonical_whole_grid_transforms(
        &mut self,
        lessons: &[(Grid, Grid)],
    ) -> Result<Vec<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() {
            return Err(Error::NotReady);
        }
        let mut hypotheses = (1..8)
            .map(CanonicalWholeGridTransform::Dihedral)
            .collect::<Vec<_>>();
        hypotheses.extend((1..=5).flat_map(|y| {
            (1..=5).filter_map(move |x| {
                ((y, x) != (1, 1)).then_some(CanonicalWholeGridTransform::Scale(y, x))
            })
        }));
        hypotheses.extend((1..=5).flat_map(|y| {
            (1..=5).filter_map(move |x| {
                ((y, x) != (1, 1)).then_some(CanonicalWholeGridTransform::Tile(y, x))
            })
        }));
        hypotheses.push(CanonicalWholeGridTransform::CropMajority);
        let root = self
            .root
            .provision(b"canonical-whole-grid-transform-induction")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        let mut programs = Vec::new();
        for hypothesis in hypotheses {
            let mut valid = true;
            for (input, expected) in lessons {
                let values = Self::grid_values(input)?;
                let expected_values = Self::grid_values(expected)?;
                let Some((height, width, actual)) = Self::render_whole_grid_transform(
                    input.height(),
                    input.width(),
                    &values,
                    hypothesis,
                ) else {
                    valid = false;
                    break;
                };
                if height != expected.height()
                    || width != expected.width()
                    || actual.iter().zip(&expected_values).any(|(a, b)| !a.same(b))
                {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }
            let program = root
                .provision(b"replay-valid-candidates")
                .provision(&programs.len().to_be_bytes())
                .provision(b"program");
            program
                .provision(b"canonical-whole-grid-transform")
                .terminate();
            match hypothesis {
                CanonicalWholeGridTransform::Dihedral(kind) => {
                    program
                        .provision(b"dihedral")
                        .provision(&[kind])
                        .terminate();
                }
                CanonicalWholeGridTransform::Scale(y, x) => {
                    program
                        .provision(b"scale")
                        .provision(&y.to_be_bytes())
                        .provision(&x.to_be_bytes())
                        .terminate();
                }
                CanonicalWholeGridTransform::Tile(y, x) => {
                    program
                        .provision(b"tile")
                        .provision(&y.to_be_bytes())
                        .provision(&x.to_be_bytes())
                        .terminate();
                }
                CanonicalWholeGridTransform::CropMajority => {
                    program.provision(b"crop-majority").terminate();
                }
            }
            program
                .provision(b"groups")
                .provision(b"whole-grid-role-preserving-transform")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            programs.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }

        // A palette map is a whole-grid role substitution: every occurrence of one demonstrated
        // source role must resolve to exactly one target role across every demonstration.
        let mut mapping: Vec<(Address, Address)> = Vec::new();
        let mut palette_valid = true;
        let mut palette_changed = false;
        for (input, expected) in lessons {
            if input.height() != expected.height() || input.width() != expected.width() {
                palette_valid = false;
                break;
            }
            let before = Self::grid_values(input)?;
            let after = Self::grid_values(expected)?;
            for (source, target) in before.iter().zip(&after) {
                if let Some((_, bound)) = mapping.iter().find(|(seen, _)| seen.same(source)) {
                    if !bound.same(target) {
                        palette_valid = false;
                        break;
                    }
                } else {
                    mapping.push((source.clone(), target.clone()));
                }
                palette_changed |= !source.same(target);
            }
            if !palette_valid {
                break;
            }
        }
        if palette_valid && palette_changed {
            let program = root
                .provision(b"replay-valid-candidates")
                .provision(&programs.len().to_be_bytes())
                .provision(b"program");
            program
                .provision(b"canonical-whole-grid-transform")
                .terminate();
            let entries = program.provision(b"palette-map");
            for (index, (source, target)) in mapping.iter().enumerate() {
                let entry = entries.provision(&index.to_be_bytes());
                entry.provision(b"source").point_to(source);
                entry.provision(b"target").point_to(target);
            }
            program
                .provision(b"groups")
                .provision(b"whole-grid-palette-role-map")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            programs.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }

        let mut separator_valid = true;
        for (input, expected) in lessons {
            let before = Self::grid_values(input)?;
            let after = Self::grid_values(expected)?;
            let Some((height, width, actual)) =
                Self::render_separator_compaction(input.height(), input.width(), &before)
            else {
                separator_valid = false;
                break;
            };
            if height != expected.height()
                || width != expected.width()
                || actual.iter().zip(&after).any(|(a, b)| !a.same(b))
            {
                separator_valid = false;
                break;
            }
        }
        if separator_valid {
            let program = root
                .provision(b"replay-valid-candidates")
                .provision(&programs.len().to_be_bytes())
                .provision(b"program");
            program
                .provision(b"canonical-whole-grid-transform")
                .terminate();
            program
                .provision(b"separator-compaction")
                .provision(b"structurally-bound-role")
                .terminate();
            program
                .provision(b"groups")
                .provision(b"complete-uniform-row-column-removal")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            programs.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }

        for block_y in 2..=5 {
            for block_x in 2..=5 {
                for uniform in [true, false] {
                    let mut valid = true;
                    for (input, expected) in lessons {
                        let before = Self::grid_values(input)?;
                        let after = Self::grid_values(expected)?;
                        let Some((height, width, actual)) = Self::render_block_reduction(
                            input.height(),
                            input.width(),
                            &before,
                            block_y,
                            block_x,
                            uniform,
                        ) else {
                            valid = false;
                            break;
                        };
                        if height != expected.height()
                            || width != expected.width()
                            || actual.iter().zip(&after).any(|(a, b)| !a.same(b))
                        {
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        continue;
                    }
                    let program = root
                        .provision(b"replay-valid-candidates")
                        .provision(&programs.len().to_be_bytes())
                        .provision(b"program");
                    program
                        .provision(b"canonical-whole-grid-transform")
                        .terminate();
                    let reduction = program.provision(b"block-reduction");
                    reduction
                        .provision(b"height")
                        .provision(&block_y.to_be_bytes())
                        .terminate();
                    reduction
                        .provision(b"width")
                        .provision(&block_x.to_be_bytes())
                        .terminate();
                    reduction
                        .provision(if uniform { b"uniform" } else { b"unique-mode" })
                        .terminate();
                    program
                        .provision(b"groups")
                        .provision(b"integer-block-role-reduction")
                        .terminate();
                    program.provision(b"all-demonstrations-valid").terminate();
                    programs.push(GroupProgram {
                        root: program,
                        owner: self.root.clone(),
                    });
                }
            }
        }

        if let Some(background) = Self::background(&Self::grid_values(&lessons[0].0)?) {
            for diagonal in [false, true] {
                for smallest in [false, true] {
                    let mut valid = true;
                    for (input, expected) in lessons {
                        let before = Self::grid_values(input)?;
                        let after = Self::grid_values(expected)?;
                        let Some((height, width, actual)) = Self::render_object_extraction(
                            input.height(),
                            input.width(),
                            &before,
                            &background,
                            diagonal,
                            smallest,
                        ) else {
                            valid = false;
                            break;
                        };
                        if height != expected.height()
                            || width != expected.width()
                            || actual.iter().zip(&after).any(|(a, b)| !a.same(b))
                        {
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        continue;
                    }
                    let program = root
                        .provision(b"replay-valid-candidates")
                        .provision(&programs.len().to_be_bytes())
                        .provision(b"program");
                    program
                        .provision(b"canonical-whole-grid-transform")
                        .terminate();
                    let extraction = program.provision(b"object-extraction");
                    extraction.provision(b"background").point_to(&background);
                    extraction
                        .provision(if diagonal {
                            b"eight-connected"
                        } else {
                            b"four-connected"
                        })
                        .terminate();
                    extraction
                        .provision(if smallest { b"smallest" } else { b"largest" })
                        .terminate();
                    program
                        .provision(b"groups")
                        .provision(b"unique-extreme-component-bounding-crop")
                        .terminate();
                    program.provision(b"all-demonstrations-valid").terminate();
                    programs.push(GroupProgram {
                        root: program,
                        owner: self.root.clone(),
                    });
                }
            }
        }
        Ok(programs)
    }

    pub(super) fn learn_broad_structural_candidates(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Vec<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        let assembly = self
            .root
            .provision(b"broad-structural-induction")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        let candidate_root = assembly.provision(b"replay-valid-candidates");
        let mut candidates = Vec::new();
        macro_rules! broad_program {
            () => {{
                candidate_root
                    .provision(&candidates.len().to_be_bytes())
                    .provision(b"program")
            }};
        }
        if let Some(rule) = self.induce_progression(lessons)? {
            let program = broad_program!();
            program.provision(b"progression-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"source").point_to(&rule.source);
            program.provision(b"target").point_to(&rule.target);
            program
                .provision(if rule.reverse { b"reverse" } else { b"forward" })
                .terminate();
            program
                .provision(b"groups")
                .provision(b"arithmetic-progression-continuation")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_connector(lessons)? {
            let program = broad_program!();
            program
                .provision(b"orthogonal-connector-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"anchor").point_to(&rule.anchor);
            program.provision(b"first").point_to(&rule.first);
            program.provision(b"second").point_to(&rule.second);
            program.provision(b"target").point_to(&rule.target);
            if rule.first_horizontal {
                program.provision(b"first-horizontal").terminate();
            }
            if rule.second_horizontal {
                program.provision(b"second-horizontal").terminate();
            }
            program
                .provision(b"groups")
                .provision(b"two-role-orthogonal-connector")
                .terminate();
            Self::record_canonical_cardinal_segments(
                &program,
                b"orthogonal-connector-construction",
                b"anchor-to-two-bound-role-points-via-bound-bend-order",
                b"input-snapshot-canonical-mask-layers",
                b"first-background-only-segment-layer-wins",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_ordinal_column_fill(lessons)? {
            let program = broad_program!();
            program
                .provision(b"ordinal-column-fill-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"bottom-bound-column-ordinal-intervals")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_nearest_marker_envelope(lessons)? {
            let program = broad_program!();
            program
                .provision(b"nearest-marker-envelope-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"marker-row-and-nearest-border-envelope")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_lattice_separators(lessons)? {
            let program = broad_program!();
            program
                .provision(b"lattice-separator-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"inner").point_to(&rule.inner);
            program.provision(b"outer").point_to(&rule.outer);
            program
                .provision(b"groups")
                .provision(b"bounded-lattice-separators-and-extensions")
                .terminate();
            Self::record_canonical_bounding_envelope(
                &program,
                CanonicalBoundingEnvelopeSemantics {
                    provenance: b"lattice-separator-construction",
                    relation: b"complete-empty-envelope-axes-bind-separator-lines",
                    source_points: b"all-query-nonbackground-cells",
                    derived_geometry: b"inclusive-envelope-rows-columns-and-axis-gaps",
                    masks: b"complete-gap-axis-lines-extended-to-grid-bounds",
                    composition: b"input-snapshot-gap-rows-before-gap-columns",
                    overwrite: b"separator-lines-replace-input-values",
                },
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_corner_halo(lessons)? {
            let program = broad_program!();
            program.provision(b"corner-halo-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"target").point_to(&rule.target);
            program
                .provision(b"groups")
                .provision(b"cycle-convex-corner-outer-arms")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_corner_pixels(lessons)? {
            let program = broad_program!();
            program.provision(b"corner-pixel-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"convex").point_to(&rule.convex);
            program.provision(b"concave").point_to(&rule.concave);
            program
                .provision(b"groups")
                .provision(b"exterior-relative-polyline-corners")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_periodic_diamonds(lessons)? {
            let program = broad_program!();
            program
                .provision(b"periodic-diamond-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"period")
                .provision(&rule.period.to_be_bytes())
                .terminate();
            program
                .provision(b"groups")
                .provision(b"lattice-anomaly-diamond-stamps")
                .terminate();
            Self::record_canonical_periodic_lattice(
                &program,
                CanonicalPeriodicLatticeSemantics {
                    provenance: b"periodic-diamond-construction",
                    relation: b"grid-origin-lattice-markers-to-interior-diamond-stamps",
                    origin: b"fixed-grid-origin-phase",
                    period: b"demonstration-bound-shared-two-axis-period",
                    bounds: b"complete-grid-anchor-bounds-with-clipped-stamps",
                    masks: b"query-marker-selected-anchors-and-derived-diamond-interiors",
                    composition: b"clear-selected-anchors-then-row-major-stamps-on-input-snapshot",
                    overwrite: b"row-major-stamps-replace-existing-values",
                },
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_glyph_classifier(lessons)? {
            let program = broad_program!();
            program
                .provision(b"glyph-classifier-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            let mapping = program.provision(b"mapping");
            for (index, (pattern, color)) in rule.mapping.iter().enumerate() {
                let entry = mapping.provision(&index.to_be_bytes());
                entry.provision(b"pattern").provision(pattern).terminate();
                entry.provision(b"color").point_to(color);
            }
            program
                .provision(b"groups")
                .provision(b"normalized-three-square-glyph-classes")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        for rule in self.induce_marker_crosses(lessons)? {
            let program = broad_program!();
            program.provision(b"marker-cross-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"marker").point_to(&rule.marker);
            program.provision(b"center").point_to(&rule.center);
            program.provision(b"diagonal").point_to(&rule.diagonal);
            let overlays = if rule.complete_diagonal_neighborhoods {
                program
                    .provision(b"complete-diagonal-neighborhoods")
                    .terminate();
                b"center-and-complete-diagonal-neighbourhood".as_slice()
            } else {
                b"center-and-diagonal-neighbourhood".as_slice()
            };
            program
                .provision(b"groups")
                .provision(b"marker-axis-crosses-and-diagonal-corners")
                .terminate();
            Self::record_canonical_axis_projection(
                &program,
                b"marker-cross-construction",
                b"homogeneous-marker-sources",
                b"horizontal-and-vertical",
                overlays,
                b"axis-layers-then-source-neighbourhood-wins",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_arrow_rays(lessons)? {
            let program = broad_program!();
            program.provision(b"arrow-ray-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"empty").point_to(&rule.empty);
            program
                .provision(b"groups")
                .provision(b"component-arrow-rays-and-boundaries")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_template_rays(lessons)? {
            let program = broad_program!();
            program.provision(b"template-ray-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"partial-template-lattice-rays")
                .terminate();
            Self::record_canonical_relative_stencil(
                &program,
                b"template-ray-construction",
                b"partial-role-components-to-lattice-ray-anchors",
                b"largest-component-normalized-shape",
                b"repeated-lattice-ray-anchors",
                b"clip-border-visible-cells",
                b"later-ordered-stamp-wins",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_relocate_diagrams(lessons)? {
            let program = broad_program!();
            program
                .provision(b"relocate-diagram-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"role-bound-dihedral-diagram-relocation")
                .terminate();
            Self::record_canonical_role_transform(
                &program,
                b"relocate-diagram-construction",
                b"unique-role-correspondences-bind-destination-translation",
                b"complete-mixed-role-component",
                b"unique-admissible-dihedral-and-translation",
                b"clear-all-source-components-then-ordered-destinations",
                b"ordered-destination-wins-after-input-snapshot-compatibility",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_append_marker_period(lessons)? {
            let program = broad_program!();
            program
                .provision(b"append-marker-period-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"marker").point_to(&rule.marker);
            program
                .provision(b"groups")
                .provision(b"marker-length-prefix-period-append")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_square_partition(lessons)? {
            let program = broad_program!();
            program
                .provision(b"square-partition-construction")
                .terminate();
            for (name, value) in [
                (b"background".as_slice(), &rule.background),
                (b"source".as_slice(), &rule.source),
                (b"block".as_slice(), &rule.block),
                (b"remainder".as_slice(), &rule.remainder),
            ] {
                program.provision(name).point_to(value);
            }
            let examples = program.provision(b"examples");
            for (index, (features, selected)) in rule.examples.iter().enumerate() {
                let entry = examples.provision(&index.to_be_bytes());
                let mut token = Vec::new();
                for value in features {
                    token.extend_from_slice(&value.to_be_bytes());
                }
                entry.provision(b"features").provision(&token).terminate();
                if *selected {
                    entry.provision(b"selected").terminate();
                }
            }
            program
                .provision(b"groups")
                .provision(b"learned-local-two-square-partition")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_marker_network(lessons)? {
            let program = broad_program!();
            program
                .provision(b"marker-network-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"marker").point_to(&rule.marker);
            program
                .provision(b"groups")
                .provision(b"preferred-axis-marker-network-with-adjacent-components")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_axis_profile_projection(lessons)? {
            let program = broad_program!();
            program
                .provision(b"axis-profile-projection-construction")
                .terminate();
            for (name, value) in [
                (b"empty".as_slice(), &rule.empty),
                (b"object".as_slice(), &rule.object),
                (b"profile".as_slice(), &rule.profile),
                (b"separator".as_slice(), &rule.separator),
                (b"target".as_slice(), &rule.target),
            ] {
                program.provision(name).point_to(value);
            }
            program
                .provision(b"groups")
                .provision(b"axis-object-separator-profile-projection")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_separator_template(lessons)? {
            let program = broad_program!();
            program
                .provision(b"separator-template-construction")
                .terminate();
            program.provision(b"empty").point_to(&rule.empty);
            program.provision(b"regular").point_to(&rule.regular);
            program.provision(b"separator").point_to(&rule.separator);
            program
                .provision(b"groups")
                .provision(b"scaled-exemplar-mask-transfer")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_missing_corner_paths(lessons)? {
            let program = broad_program!();
            program
                .provision(b"missing-corner-path-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"marker").point_to(&rule.marker);
            program.provision(b"path").point_to(&rule.path);
            program
                .provision(b"groups")
                .provision(b"three-corner-envelope-and-diagonal")
                .terminate();
            Self::record_canonical_bounding_envelope(
                &program,
                CanonicalBoundingEnvelopeSemantics {
                    provenance: b"missing-corner-path-construction",
                    relation: b"three-occupied-envelope-corners-bind-missing-corner",
                    source_points: b"query-cells-equal-bound-marker-role",
                    derived_geometry:
                        b"ordered-envelope-corners-missing-corner-and-opposite-diagonal",
                    masks: b"missing-corner-axes-and-opposite-corner-diagonal",
                    composition: b"row-then-column-then-diagonal-then-marker-restore",
                    overwrite: b"later-path-layers-and-final-markers-win",
                },
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_partial_crosses(lessons)? {
            let program = broad_program!();
            program.provision(b"partial-cross-construction").terminate();
            program.provision(b"empty").point_to(&rule.empty);
            program.provision(b"noise").point_to(&rule.noise);
            program.provision(b"marker").point_to(&rule.marker);
            program.provision(b"target").point_to(&rule.target);
            program
                .provision(b"groups")
                .provision(b"partial-five-cell-cross-completion")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_center_marker_transfer(lessons)? {
            let program = broad_program!();
            program
                .provision(b"center-marker-transfer-construction")
                .terminate();
            program.provision(b"empty").point_to(&rule.empty);
            program.provision(b"frame").point_to(&rule.frame);
            program.provision(b"marker").point_to(&rule.marker);
            program.provision(b"target").point_to(&rule.target);
            program
                .provision(b"groups")
                .provision(b"panel-direction-marker-transfer")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_tiled_mode_denoise(lessons)? {
            let program = broad_program!();
            program
                .provision(b"tiled-mode-denoise-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"repeated-tile-residue-mode-reconstruction")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_adaptive_inset(lessons)? {
            let program = broad_program!();
            program
                .provision(b"adaptive-inset-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"target").point_to(&rule.target);
            program
                .provision(b"groups")
                .provision(b"extent-scaled-three-role-inset")
                .terminate();
            Self::record_canonical_bounding_envelope(
                &program,
                CanonicalBoundingEnvelopeSemantics {
                    provenance: b"adaptive-inset-construction",
                    relation: b"envelope-center-and-maximum-span-bind-role-insets",
                    source_points: b"exactly-three-distinct-query-nonbackground-roles",
                    derived_geometry: b"integral-envelope-center-and-one-third-maximum-span",
                    masks: b"inset-role-destinations-and-center-cell",
                    composition: b"input-snapshot-row-major-role-insets-then-center",
                    overwrite: b"later-role-insets-then-center-target-win",
                },
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_legend_blocks(lessons)? {
            let program = broad_program!();
            program.provision(b"legend-block-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"carrier").point_to(&rule.carrier);
            program
                .provision(b"groups")
                .provision(b"quarter-turned-legend-block-recoloring")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_reflect_internal_motif(lessons)? {
            let program = broad_program!();
            program
                .provision(b"reflect-internal-motif-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"frame").point_to(&rule.frame);
            program.provision(b"motif").point_to(&rule.motif);
            program
                .provision(b"groups")
                .provision(b"frame-centered-motif-reflection")
                .terminate();
            Self::record_canonical_role_transform(
                &program,
                b"reflect-internal-motif-construction",
                b"frame-bounds-bind-reflection-translation",
                b"frame-internal-motif-cell-set",
                b"unique-admissible-axis-reflection",
                b"input-snapshot-transformed-additions-only",
                b"motif-additions-replace-admissible-background",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_component_pair_fill(lessons)? {
            let program = broad_program!();
            program
                .provision(b"component-pair-fill-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"aligned-same-role-component-interpolation")
                .terminate();
            Self::record_canonical_mask_composition(
                &program,
                b"component-pair-fill-construction",
                b"same-role-equal-shape-pairs",
                b"axis-periodic-component-interpolation",
                b"earlier-background-only-layer-wins",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_paired_lines(lessons)? {
            let program = broad_program!();
            program.provision(b"paired-line-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"same-role-aligned-pair-segments")
                .terminate();
            Self::record_canonical_cardinal_segments(
                &program,
                b"paired-line-construction",
                b"same-role-aligned-point-pairs",
                b"input-snapshot-canonical-mask-layers",
                b"earlier-background-only-segment-layer-wins",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_component_stencil(lessons)? {
            // Bounds are unobservable when every demonstrated placement is complete. Retain both
            // canonical interpretations: the historical complete-placement contract remains first,
            // while clipping is an independently addressed replay-valid alternative for attempt 2.
            for clip_at_boundary in [false, true] {
                let program = broad_program!();
                program
                    .provision(b"component-stencil-construction")
                    .terminate();
                if clip_at_boundary {
                    program.provision(b"clip-at-grid-boundary").terminate();
                }
                program.provision(b"background").point_to(&rule.background);
                for (head, entries) in [
                    (b"template".as_slice(), &rule.template),
                    (b"effects".as_slice(), &rule.effects),
                ] {
                    let root = program.provision(head);
                    for (index, (dy, dx, role)) in entries.iter().enumerate() {
                        let entry = root.provision(&index.to_be_bytes());
                        entry
                            .provision(b"dy")
                            .provision(&(*dy as i64).to_be_bytes())
                            .terminate();
                        entry
                            .provision(b"dx")
                            .provision(&(*dx as i64).to_be_bytes())
                            .terminate();
                        entry
                            .provision(b"role")
                            .provision(&role.to_be_bytes())
                            .terminate();
                    }
                }
                program
                    .provision(b"groups")
                    .provision(b"normalized-component-relative-stencil")
                    .terminate();
                Self::record_canonical_relative_stencil(
                    &program,
                    b"component-stencil-construction",
                    b"equal-normalized-mixed-components",
                    b"component-relative-role-effect-stencil",
                    b"normalized-component-top-left-anchors",
                    if clip_at_boundary {
                        b"clip-out-of-bounds-placement"
                    } else {
                        b"reject-out-of-bounds-placement"
                    },
                    b"reject-conflicting-overlap",
                );
                program.provision(b"all-demonstrations-valid").terminate();
                candidates.push(GroupProgram {
                    root: program,
                    owner: self.root.clone(),
                });
            }
        }
        if let Some(rule) = self.induce_boundary_rays(lessons)? {
            let program = broad_program!();
            program.provision(b"boundary-ray-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"ordered-rays-to-complete-boundary")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_alternating_stripes(lessons)? {
            let program = broad_program!();
            program
                .provision(b"alternating-stripe-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"groups")
                .provision(b"two-marker-periodic-axis-fill")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_sparse_lattice(lessons)? {
            let program = broad_program!();
            program
                .provision(b"sparse-lattice-construction")
                .terminate();
            program.provision(b"background").point_to(&rule.background);
            program
                .provision(b"step")
                .provision(&rule.step.to_be_bytes())
                .terminate();
            program
                .provision(b"groups")
                .provision(b"least-frequent-noncentral-lattice-role")
                .terminate();
            Self::record_canonical_periodic_lattice(
                &program,
                CanonicalPeriodicLatticeSemantics {
                    provenance: b"sparse-lattice-construction",
                    relation: b"selected-noncentral-role-binds-lattice-phase",
                    origin: b"query-selected-role-position-phase",
                    period: b"demonstration-bound-shared-two-axis-step",
                    bounds: b"query-nonbackground-bounding-box-inclusive",
                    masks: b"background-only-congruent-lattice-cells",
                    composition: b"input-snapshot-background-only-additions",
                    overwrite: b"preserve-all-nonbackground-input-cells",
                },
            );
            Self::record_canonical_bounding_envelope(
                &program,
                CanonicalBoundingEnvelopeSemantics {
                    provenance: b"sparse-lattice-construction",
                    relation: b"nonbackground-envelope-binds-lattice-bounds-and-center-test",
                    source_points: b"all-query-nonbackground-cells",
                    derived_geometry: b"inclusive-envelope-and-doubled-center-coordinate",
                    masks: b"envelope-bounded-periodic-candidate-cells",
                    composition: b"input-snapshot-background-only-additions",
                    overwrite: b"preserve-all-nonbackground-input-cells",
                },
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_move_to_touch(lessons)? {
            let program = broad_program!();
            program.provision(b"move-to-touch-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            program.provision(b"moving").point_to(&rule.moving);
            program.provision(b"anchor").point_to(&rule.anchor);
            program
                .provision(b"groups")
                .provision(b"cardinal-until-adjacent")
                .terminate();
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if let Some(rule) = self.induce_beams(lessons)? {
            let program = broad_program!();
            program.provision(b"axis-beam-construction").terminate();
            program.provision(b"background").point_to(&rule.background);
            let roles = program.provision(b"beam-roles");
            for (index, (color, mode)) in rule.roles.iter().enumerate() {
                let role = roles.provision(&index.to_be_bytes());
                role.provision(b"color").point_to(color);
                role.provision(match mode {
                    BeamMode::None => b"none".as_slice(),
                    BeamMode::Horizontal => b"horizontal".as_slice(),
                    BeamMode::Vertical => b"vertical".as_slice(),
                })
                .terminate();
            }
            program
                .provision(b"groups")
                .provision(b"bound-axis-beam-roles")
                .terminate();
            Self::record_canonical_mask_composition(
                &program,
                b"axis-beam-construction",
                b"role-to-axis-bindings",
                b"complete-axis-through-source-points",
                b"later-axis-layer-then-source-mask-wins",
            );
            Self::record_canonical_axis_projection(
                &program,
                b"axis-beam-construction",
                b"role-to-axis-bindings",
                b"role-selected-horizontal-or-vertical",
                b"source-point-restoration",
                b"later-axis-layer-then-source-mask-wins",
            );
            program.provision(b"all-demonstrations-valid").terminate();
            candidates.push(GroupProgram {
                root: program,
                owner: self.root.clone(),
            });
        }
        if candidates.is_empty() {
            assembly.provision(b"no-supported-structure").terminate();
        } else {
            assembly.provision(b"all-candidates-complete").terminate();
        }
        Ok(candidates)
    }

    fn grid_values(grid: &Grid) -> Result<Vec<Address>, Error> {
        (0..grid.height())
            .flat_map(|y| (0..grid.width()).map(move |x| (y, x)))
            .map(|(y, x)| Ok(field(&grid.cell(y, x)?, b"value")?.follow()?))
            .collect()
    }

    fn background(values: &[Address]) -> Option<Address> {
        let mut groups: Vec<(Address, usize)> = Vec::new();
        for value in values {
            if let Some((_, count)) = groups.iter_mut().find(|(seen, _)| seen.same(value)) {
                *count += 1;
            } else {
                groups.push((value.clone(), 1));
            }
        }
        groups
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(value, _)| value)
    }

    fn points(
        values: &[Address],
        width: usize,
        background: &Address,
    ) -> Vec<(isize, isize, Address)> {
        values
            .iter()
            .enumerate()
            .filter(|(_, value)| !value.same(background))
            .map(|(index, value)| {
                (
                    (index / width) as isize,
                    (index % width) as isize,
                    value.clone(),
                )
            })
            .collect()
    }

    fn render_progression(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &Progression,
    ) -> Option<Vec<Address>> {
        let mut points = Self::points(input, width, &rule.background);
        if points.len() != 3 || points.iter().any(|(_, _, value)| !value.same(&rule.source)) {
            return None;
        }
        points.sort_by_key(|(y, x, _)| (*y, *x));
        if rule.reverse {
            points.reverse();
        }
        let dy = points[1].0 - points[0].0;
        let dx = points[1].1 - points[0].1;
        if (dy == 0 && dx == 0)
            || (points[2].0 - points[1].0, points[2].1 - points[1].1) != (dy, dx)
        {
            return None;
        }
        let mut output = input.to_vec();
        let (mut y, mut x) = (points[2].0 + dy, points[2].1 + dx);
        while y >= 0 && x >= 0 && y < height as isize && x < width as isize {
            let index = y as usize * width + x as usize;
            if !output[index].same(&rule.background) {
                return None;
            }
            output[index] = rule.target.clone();
            y += dy;
            x += dx;
        }
        Some(output)
    }

    fn induce_progression(&self, lessons: &[GroupLesson]) -> Result<Option<Progression>, Error> {
        let input = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&input) else {
            return Ok(None);
        };
        let points = Self::points(&input, lessons[0].input.width(), &background);
        if points.len() != 3 || points.iter().any(|p| !p.2.same(&points[0].2)) {
            return Ok(None);
        }
        for reverse in [false, true] {
            let provisional = Progression {
                background: background.clone(),
                source: points[0].2.clone(),
                target: points[0].2.clone(),
                reverse,
            };
            let Some(rendered) = Self::render_progression(
                lessons[0].input.height(),
                lessons[0].input.width(),
                &input,
                &provisional,
            ) else {
                continue;
            };
            let Some(index) = rendered
                .iter()
                .enumerate()
                .find_map(|(index, value)| (!value.same(&input[index])).then_some(index))
            else {
                continue;
            };
            let mut rule = provisional;
            rule.target = expected[index].clone();
            let mut valid = !rule.target.same(&rule.background);
            for lesson in lessons {
                let source = Self::grid_values(&lesson.input)?;
                let target = Self::grid_values(&lesson.output)?;
                let Some(actual) = Self::render_progression(
                    lesson.input.height(),
                    lesson.input.width(),
                    &source,
                    &rule,
                ) else {
                    valid = false;
                    break;
                };
                if actual.len() != target.len()
                    || actual.iter().zip(&target).any(|(a, b)| !a.same(b))
                {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Ok(Some(rule));
            }
        }
        Ok(None)
    }

    fn render_connector(width: usize, input: &[Address], rule: &Connector) -> Option<Vec<Address>> {
        let height = input.len().checked_div(width)?;
        let points = Self::points(input, width, &rule.background);
        if points.len() != 3 {
            return None;
        }
        let locate = |value: &Address| {
            points
                .iter()
                .find(|(_, _, observed)| observed.same(value))
                .map(|(y, x, _)| (*y as usize, *x as usize))
        };
        let anchor = locate(&rule.anchor)?;
        let first = locate(&rule.first)?;
        let second = locate(&rule.second)?;
        let mut segments = Vec::new();
        for (stop, horizontal_first) in [
            (first, rule.first_horizontal),
            (second, rule.second_horizontal),
        ] {
            let corner = if horizontal_first {
                (anchor.0, stop.1)
            } else {
                (stop.0, anchor.1)
            };
            for (start, end) in [(anchor, corner), (corner, stop)] {
                segments.push(CanonicalCardinalSegment {
                    priority: segments.len(),
                    start,
                    end,
                    value: rule.target.clone(),
                    overwrite: CanonicalOverwrite::IfCurrentMatches(rule.background.clone()),
                });
            }
        }
        let layers = Self::project_canonical_cardinal_segments(height, width, segments)?;
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_connector(&self, lessons: &[GroupLesson]) -> Result<Option<Connector>, Error> {
        let first_input = Self::grid_values(&lessons[0].input)?;
        let first_output = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first_input) else {
            return Ok(None);
        };
        let points = Self::points(&first_input, lessons[0].input.width(), &background);
        if points.len() != 3
            || points[0].2.same(&points[1].2)
            || points[0].2.same(&points[2].2)
            || points[1].2.same(&points[2].2)
        {
            return Ok(None);
        }
        let target = first_input
            .iter()
            .zip(&first_output)
            .find_map(|(before, after)| {
                (before.same(&background) && !after.same(&background)).then_some(after.clone())
            });
        let Some(target) = target else {
            return Ok(None);
        };
        for anchor_index in 0..3 {
            let other = (0..3).filter(|i| *i != anchor_index).collect::<Vec<_>>();
            for first_horizontal in [false, true] {
                for second_horizontal in [false, true] {
                    let rule = Connector {
                        background: background.clone(),
                        anchor: points[anchor_index].2.clone(),
                        first: points[other[0]].2.clone(),
                        second: points[other[1]].2.clone(),
                        target: target.clone(),
                        first_horizontal,
                        second_horizontal,
                    };
                    let mut valid = true;
                    for lesson in lessons {
                        let source = Self::grid_values(&lesson.input)?;
                        let expected = Self::grid_values(&lesson.output)?;
                        let Some(actual) =
                            Self::render_connector(lesson.input.width(), &source, &rule)
                        else {
                            valid = false;
                            break;
                        };
                        if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                            valid = false;
                            break;
                        }
                    }
                    if valid {
                        return Ok(Some(rule));
                    }
                }
            }
        }
        Ok(None)
    }

    fn render_nearest_marker_envelope(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &NearestMarkerEnvelope,
    ) -> Option<Vec<Address>> {
        let points = Self::points(input, width, &rule.background);
        if points.is_empty() {
            return None;
        }
        let mut output = input.to_vec();
        for y in 0..height {
            let nearest = points
                .iter()
                .min_by_key(|(py, px, _)| (py.abs_diff(y as isize), *py, *px))?;
            output[y * width] = nearest.2.clone();
            output[y * width + width - 1] = nearest.2.clone();
        }
        for (y, _, color) in &points {
            for x in 0..width {
                output[*y as usize * width + x] = color.clone();
            }
        }
        for x in 0..width {
            output[x] = points.first()?.2.clone();
            output[(height - 1) * width + x] = points.last()?.2.clone();
        }
        Some(output)
    }

    fn induce_nearest_marker_envelope(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<NearestMarkerEnvelope>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = NearestMarkerEnvelope { background };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            if Self::render_nearest_marker_envelope(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&expected).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_lattice_separators(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &LatticeSeparators,
    ) -> Option<Vec<Address>> {
        let points = Self::points(input, width, &rule.background);
        if points.is_empty() || points.iter().any(|point| !point.2.same(&points[0].2)) {
            return None;
        }
        let envelope = CanonicalBoundingEnvelope::from_points(
            points
                .iter()
                .map(|point| (point.0 as usize, point.1 as usize)),
        )?;
        let (y0, y1, x0, x1) = (envelope.top, envelope.bottom, envelope.left, envelope.right);
        let gap_rows = (y0..=y1)
            .filter(|y| (x0..=x1).all(|x| input[y * width + x].same(&rule.background)))
            .collect::<Vec<_>>();
        let gap_columns = (x0..=x1)
            .filter(|x| (y0..=y1).all(|y| input[y * width + x].same(&rule.background)))
            .collect::<Vec<_>>();
        if gap_rows.is_empty() || gap_columns.is_empty() {
            return None;
        }
        let mut output = input.to_vec();
        for y in 0..height {
            for x in 0..width {
                output[y * width + x] = if gap_rows.contains(&y) {
                    if (x0..=x1).contains(&x) {
                        rule.inner.clone()
                    } else {
                        rule.outer.clone()
                    }
                } else if gap_columns.contains(&x) {
                    if (y0..=y1).contains(&y) {
                        rule.inner.clone()
                    } else {
                        rule.outer.clone()
                    }
                } else {
                    output[y * width + x].clone()
                };
            }
        }
        Some(output)
    }

    fn induce_lattice_separators(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<LatticeSeparators>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let mut additions: Vec<(Address, usize)> = Vec::new();
        for (before, after) in first.iter().zip(&expected) {
            if before.same(after) {
                continue;
            }
            if let Some((_, count)) = additions.iter_mut().find(|(seen, _)| seen.same(after)) {
                *count += 1;
            } else {
                additions.push((after.clone(), 1));
            }
        }
        if additions.len() != 2 {
            return Ok(None);
        }
        additions.sort_by_key(|entry| entry.1);
        let rule = LatticeSeparators {
            background,
            inner: additions[1].0.clone(),
            outer: additions[0].0.clone(),
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let target = Self::grid_values(&lesson.output)?;
            if Self::render_lattice_separators(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&target).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn cycle_corner_halo_positions(
        height: usize,
        width: usize,
        input: &[Address],
        background: &Address,
    ) -> Vec<usize> {
        let mut result = Vec::new();
        for (_, cells) in Self::same_color_components(height, width, input, background) {
            let contains =
                |y: isize, x: isize| y >= 0 && x >= 0 && cells.contains(&(y as usize, x as usize));
            if cells.iter().any(|&(y, x)| {
                [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .filter(|(dy, dx)| contains(y as isize + dy, x as isize + dx))
                    .count()
                    != 2
            }) {
                continue;
            }
            for &(y, x) in &cells {
                let vertical = [-1isize, 1]
                    .into_iter()
                    .filter(|dy| contains(y as isize + dy, x as isize))
                    .collect::<Vec<_>>();
                let horizontal = [-1isize, 1]
                    .into_iter()
                    .filter(|dx| contains(y as isize, x as isize + dx))
                    .collect::<Vec<_>>();
                if vertical.len() == 1 && horizontal.len() == 1 {
                    for (ay, ax) in [
                        (y as isize - vertical[0], x as isize),
                        (y as isize, x as isize - horizontal[0]),
                    ] {
                        if ay >= 0 && ax >= 0 && ay < height as isize && ax < width as isize {
                            let index = ay as usize * width + ax as usize;
                            if input[index].same(background) && !result.contains(&index) {
                                result.push(index);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    fn render_corner_halo(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &CornerHalo,
    ) -> Option<Vec<Address>> {
        let positions = Self::cycle_corner_halo_positions(height, width, input, &rule.background);
        if positions.is_empty() {
            return None;
        }
        let mut output = input.to_vec();
        for index in positions {
            output[index] = rule.target.clone();
        }
        Some(output)
    }

    fn induce_corner_halo(&self, lessons: &[GroupLesson]) -> Result<Option<CornerHalo>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let positions = Self::cycle_corner_halo_positions(
            lessons[0].input.height(),
            lessons[0].input.width(),
            &first,
            &background,
        );
        let Some(target) = positions.first().map(|index| expected[*index].clone()) else {
            return Ok(None);
        };
        let rule = CornerHalo { background, target };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_corner_halo(lesson.input.height(), lesson.input.width(), &input, &rule)
                .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn corner_pixel_positions(
        height: usize,
        width: usize,
        input: &[Address],
        background: &Address,
    ) -> Vec<(usize, bool)> {
        let mut exterior = vec![false; input.len()];
        let mut queue = std::collections::VecDeque::new();
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                if (y == 0 || x == 0 || y + 1 == height || x + 1 == width)
                    && input[index].same(background)
                {
                    exterior[index] = true;
                    queue.push_back((y, x));
                }
            }
        }
        while let Some((y, x)) = queue.pop_front() {
            for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                let (ay, ax) = (y as isize + dy, x as isize + dx);
                if ay < 0 || ax < 0 || ay >= height as isize || ax >= width as isize {
                    continue;
                }
                let index = ay as usize * width + ax as usize;
                if !exterior[index] && input[index].same(background) {
                    exterior[index] = true;
                    queue.push_back((ay as usize, ax as usize));
                }
            }
        }
        let mut result = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if input[y * width + x].same(background) {
                    continue;
                }
                let neighbors = [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .filter(|(dy, dx)| {
                        let (ay, ax) = (y as isize + dy, x as isize + dx);
                        ay >= 0
                            && ax >= 0
                            && ay < height as isize
                            && ax < width as isize
                            && !input[ay as usize * width + ax as usize].same(background)
                    })
                    .collect::<Vec<_>>();
                if neighbors.len() == 2
                    && neighbors[0].0 * neighbors[1].0 + neighbors[0].1 * neighbors[1].1 == 0
                {
                    let ay = (y as isize + neighbors[0].0 + neighbors[1].0) as usize;
                    let ax = (x as isize + neighbors[0].1 + neighbors[1].1) as usize;
                    result.push((y * width + x, !exterior[ay * width + ax]));
                }
            }
        }
        result
    }

    fn render_corner_pixels(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &CornerPixels,
    ) -> Option<Vec<Address>> {
        let positions = Self::corner_pixel_positions(height, width, input, &rule.background);
        if positions.is_empty() {
            return None;
        }
        let mut output = input.to_vec();
        for (index, convex) in positions {
            output[index] = if convex {
                rule.convex.clone()
            } else {
                rule.concave.clone()
            };
        }
        Some(output)
    }

    fn induce_corner_pixels(&self, lessons: &[GroupLesson]) -> Result<Option<CornerPixels>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let positions = Self::corner_pixel_positions(
            lessons[0].input.height(),
            lessons[0].input.width(),
            &first,
            &background,
        );
        let Some(convex) = positions
            .iter()
            .find(|(_, kind)| *kind)
            .map(|(index, _)| expected[*index].clone())
        else {
            return Ok(None);
        };
        let Some(concave) = positions
            .iter()
            .find(|(_, kind)| !*kind)
            .map(|(index, _)| expected[*index].clone())
        else {
            return Ok(None);
        };
        let rule = CornerPixels {
            background,
            convex,
            concave,
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_corner_pixels(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_periodic_diamonds(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &PeriodicDiamonds,
    ) -> Option<Vec<Address>> {
        let anchors = Self::generate_canonical_periodic_lattice(CanonicalPeriodicLattice {
            rows: CanonicalPeriodicAxis {
                lower: 0,
                upper: isize::try_from(height.checked_sub(1)?).ok()?,
                phase: 0,
                step: rule.period,
            },
            columns: CanonicalPeriodicAxis {
                lower: 0,
                upper: isize::try_from(width.checked_sub(1)?).ok()?,
                phase: 0,
                step: rule.period,
            },
        })?;
        let mut output = input.to_vec();
        let mut changed = false;
        for (y, x) in anchors {
            let marker = input[y * width + x].clone();
            if marker.same(&rule.background) {
                continue;
            }
            output[y * width + x] = rule.background.clone();
            changed = true;
            for dy in 1..rule.period {
                for dx in 1..rule.period {
                    let center = rule.period as isize;
                    if ((2 * dy) as isize - center).abs() + ((2 * dx) as isize - center).abs()
                        < center
                        && y + dy < height
                        && x + dx < width
                    {
                        output[(y + dy) * width + x + dx] = marker.clone();
                    }
                }
            }
        }
        changed.then_some(output)
    }

    fn induce_periodic_diamonds(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<PeriodicDiamonds>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        for period in 3..=8 {
            let mut lattice = Vec::new();
            for y in (0..lessons[0].input.height()).step_by(period) {
                for x in (0..lessons[0].input.width()).step_by(period) {
                    lattice.push(first[y * lessons[0].input.width() + x].clone());
                }
            }
            let Some(background) = Self::background(&lattice) else {
                continue;
            };
            let rule = PeriodicDiamonds { background, period };
            let mut valid = true;
            for lesson in lessons {
                let input = Self::grid_values(&lesson.input)?;
                let wanted = Self::grid_values(&lesson.output)?;
                if Self::render_periodic_diamonds(
                    lesson.input.height(),
                    lesson.input.width(),
                    &input,
                    &rule,
                )
                .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
                {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Ok(Some(rule));
            }
        }
        Ok(None)
    }

    fn render_glyph_classifier(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &GlyphClassifier,
    ) -> Option<Vec<Address>> {
        if height != 3 || !width.is_multiple_of(3) {
            return None;
        }
        let mut output = input.to_vec();
        for x0 in (0..width).step_by(3) {
            let pattern = (0..3)
                .flat_map(|y| {
                    (x0..x0 + 3)
                        .map(move |x| u8::from(!input[y * width + x].same(&rule.background)))
                })
                .collect::<Vec<_>>();
            let color = rule
                .mapping
                .iter()
                .find(|(seen, _)| seen == &pattern)?
                .1
                .clone();
            for y in 0..3 {
                for x in x0..x0 + 3 {
                    output[y * width + x] = color.clone();
                }
            }
        }
        Some(output)
    }

    fn induce_glyph_classifier(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GlyphClassifier>, Error> {
        let all = lessons
            .iter()
            .map(|lesson| Self::grid_values(&lesson.input))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let Some(background) = Self::background(&all) else {
            return Ok(None);
        };
        let mut mapping: Vec<(Vec<u8>, Address)> = Vec::new();
        for lesson in lessons {
            if lesson.input.height() != 3 || !lesson.input.width().is_multiple_of(3) {
                return Ok(None);
            }
            let input = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            for x0 in (0..lesson.input.width()).step_by(3) {
                let mut pattern = Vec::new();
                for y in 0..3 {
                    for x in x0..x0 + 3 {
                        pattern.push(u8::from(
                            !input[y * lesson.input.width() + x].same(&background),
                        ));
                    }
                }
                let color = expected[x0].clone();
                if (0..3).any(|y| {
                    (x0..x0 + 3).any(|x| !expected[y * lesson.input.width() + x].same(&color))
                }) {
                    return Ok(None);
                }
                if let Some((_, prior)) = mapping.iter().find(|(seen, _)| seen == &pattern) {
                    if !prior.same(&color) {
                        return Ok(None);
                    }
                } else {
                    mapping.push((pattern, color));
                }
            }
        }
        if mapping.len() < 2 {
            return Ok(None);
        }
        let rule = GlyphClassifier {
            background,
            mapping,
        };
        Ok(Some(rule))
    }

    fn same_color_components8(
        height: usize,
        width: usize,
        input: &[Address],
        background: &Address,
    ) -> Vec<(Address, Vec<(usize, usize)>)> {
        let mut seen = vec![false; input.len()];
        let mut result = Vec::new();
        for start in 0..input.len() {
            if seen[start] || input[start].same(background) {
                continue;
            }
            let color = input[start].clone();
            seen[start] = true;
            let mut queue = std::collections::VecDeque::from([(start / width, start % width)]);
            let mut cells = Vec::new();
            while let Some((y, x)) = queue.pop_front() {
                cells.push((y, x));
                for dy in -1isize..=1 {
                    for dx in -1isize..=1 {
                        if dy == 0 && dx == 0 {
                            continue;
                        }
                        let (ay, ax) = (y as isize + dy, x as isize + dx);
                        if ay < 0 || ax < 0 || ay >= height as isize || ax >= width as isize {
                            continue;
                        }
                        let index = ay as usize * width + ax as usize;
                        if !seen[index] && input[index].same(&color) {
                            seen[index] = true;
                            queue.push_back((ay as usize, ax as usize));
                        }
                    }
                }
            }
            result.push((color, cells));
        }
        result
    }

    fn render_template_rays(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &TemplateRays,
    ) -> Option<Vec<Address>> {
        let components = Self::same_color_components8(height, width, input, &rule.background);
        if components.len() < 2 {
            return None;
        }
        let template_index =
            (0..components.len()).max_by_key(|index| components[*index].1.len())?;
        let (_, template) = &components[template_index];
        let y0 = template.iter().map(|p| p.0).min()?;
        let x0 = template.iter().map(|p| p.1).min()?;
        let shape = template
            .iter()
            .map(|(y, x)| (y - y0, x - x0))
            .collect::<Vec<_>>();
        let sh = shape.iter().map(|p| p.0).max()? + 1;
        let sw = shape.iter().map(|p| p.1).max()? + 1;
        let mut placements = Vec::new();
        for (index, (marker, points)) in components.iter().enumerate() {
            if index == template_index {
                continue;
            }
            if marker.same(&components[template_index].0) || points.len() >= template.len() {
                return None;
            }
            let mut options = Vec::new();
            let ky_limit = height / (sh + 1) + 2;
            let kx_limit = width / (sw + 1) + 2;
            for ky in -(ky_limit as isize)..=ky_limit as isize {
                for kx in -(kx_limit as isize)..=kx_limit as isize {
                    if ky == 0 && kx == 0 {
                        continue;
                    }
                    let ay = y0 as isize + ky * (sh + 1) as isize;
                    let ax = x0 as isize + kx * (sw + 1) as isize;
                    if points.iter().all(|&(y, x)| {
                        let (dy, dx) = (y as isize - ay, x as isize - ax);
                        dy >= 0 && dx >= 0 && shape.contains(&(dy as usize, dx as usize))
                    }) {
                        options.push((ky, kx));
                    }
                }
            }
            if options.len() != 1 {
                return None;
            }
            let (ky, kx) = options[0];
            let mut step = 1isize;
            loop {
                let anchor = (
                    y0 as isize + ky * (sh + 1) as isize * step,
                    x0 as isize + kx * (sw + 1) as isize * step,
                );
                let visible = shape.iter().any(|&(dy, dx)| {
                    let (y, x) = (anchor.0 + dy as isize, anchor.1 + dx as isize);
                    y >= 0 && x >= 0 && y < height as isize && x < width as isize
                });
                if !visible {
                    break;
                }
                placements.push(CanonicalStencilPlacement {
                    priority: placements.len(),
                    anchor,
                    cells: shape
                        .iter()
                        .map(|&(dy, dx)| (dy as isize, dx as isize, marker.clone()))
                        .collect(),
                    overwrite: CanonicalOverwrite::Replace,
                });
                step += 1;
            }
        }
        if placements.is_empty() {
            return None;
        }
        let layers = Self::project_canonical_stencils(
            height,
            width,
            placements,
            CanonicalStencilBounds::Clip,
            CanonicalStencilOverlap::Ordered,
        )?;
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_template_rays(&self, lessons: &[GroupLesson]) -> Result<Option<TemplateRays>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = TemplateRays { background };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_template_rays(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_relocate_diagrams(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &RelocateDiagrams,
    ) -> Option<Vec<Address>> {
        let components = Self::mixed_components(height, width, input, &rule.background);
        let templates = components
            .iter()
            .filter_map(|cells| {
                let mut colors: Vec<(Address, usize)> = Vec::new();
                for &(y, x) in cells {
                    let value = &input[y * width + x];
                    if let Some((_, count)) = colors.iter_mut().find(|(seen, _)| seen.same(value)) {
                        *count += 1;
                    } else {
                        colors.push((value.clone(), 1));
                    }
                }
                (cells.len() >= 5 && colors.len() >= 3).then_some((cells, colors))
            })
            .collect::<Vec<_>>();
        if templates.is_empty() {
            return None;
        }
        let mut plans: Vec<CanonicalRoleTransformProjection> = Vec::new();
        for (cells, colors) in templates {
            let roles = cells
                .iter()
                .filter_map(|&(y, x)| {
                    let value = &input[y * width + x];
                    (colors.iter().find(|(seen, _)| seen.same(value))?.1 == 1).then_some((
                        y,
                        x,
                        value.clone(),
                    ))
                })
                .collect::<Vec<_>>();
            if roles.len() < 2 {
                return None;
            }
            let role_cells = roles
                .iter()
                .map(|(y, x, value)| (*y as isize, *x as isize, value.clone()))
                .collect::<Vec<_>>();
            let component_cells = cells
                .iter()
                .map(|&(y, x)| (y as isize, x as isize, input[y * width + x].clone()))
                .collect::<Vec<_>>();
            let mut unique: Vec<CanonicalRoleTransformProjection> = Vec::new();
            for transform in 0..8 {
                let first =
                    Self::canonical_dihedral(transform, roles[0].0 as isize, roles[0].1 as isize)?;
                for (target_index, target) in input.iter().enumerate() {
                    let (ty, tx) = (target_index / width, target_index % width);
                    if !target.same(&roles[0].2) || cells.contains(&(ty, tx)) {
                        continue;
                    }
                    let translation = (
                        (ty as isize).checked_sub(first.0)?,
                        (tx as isize).checked_sub(first.1)?,
                    );
                    let Some(role_projection) = Self::project_canonical_role_transforms(
                        height,
                        width,
                        vec![CanonicalRoleTransform {
                            priority: 0,
                            dihedral: transform,
                            translation,
                            cells: role_cells.clone(),
                            overwrite: CanonicalOverwrite::Replace,
                        }],
                    )
                    .and_then(|mut projected| projected.pop()) else {
                        continue;
                    };
                    if !role_projection
                        .cells
                        .iter()
                        .all(|cell| input[cell.target].same(&cell.value))
                    {
                        continue;
                    }
                    let Some(projection) = Self::project_canonical_role_transforms(
                        height,
                        width,
                        vec![CanonicalRoleTransform {
                            priority: 0,
                            dihedral: transform,
                            translation,
                            cells: component_cells.clone(),
                            overwrite: CanonicalOverwrite::Replace,
                        }],
                    )
                    .and_then(|mut projected| projected.pop()) else {
                        continue;
                    };
                    if projection.cells.iter().any(|cell| {
                        let observed = &input[cell.target];
                        !observed.same(&rule.background) && !observed.same(&cell.value)
                    }) {
                        continue;
                    }
                    if !unique.iter().any(|prior| {
                        prior.cells.len() == projection.cells.len()
                            && projection.cells.iter().all(|cell| {
                                prior.cells.iter().any(|prior_cell| {
                                    cell.target == prior_cell.target
                                        && cell.value.same(&prior_cell.value)
                                })
                            })
                    }) {
                        unique.push(projection);
                    }
                }
            }
            if unique.len() != 1 {
                return None;
            }
            plans.push(unique.pop()?);
        }
        let mut layers = Vec::new();
        for projection in &plans {
            for cell in &projection.cells {
                layers.push(CanonicalMaskLayer {
                    priority: layers.len(),
                    mask: vec![cell.source],
                    value: rule.background.clone(),
                    overwrite: CanonicalOverwrite::Replace,
                });
            }
        }
        for projection in plans {
            for cell in projection.cells {
                layers.push(CanonicalMaskLayer {
                    priority: layers.len(),
                    mask: vec![cell.target],
                    value: cell.value,
                    overwrite: projection.overwrite.clone(),
                });
            }
        }
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_relocate_diagrams(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<RelocateDiagrams>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = RelocateDiagrams { background };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_relocate_diagrams(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_append_marker_period(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &AppendMarkerPeriod,
    ) -> Option<Vec<Address>> {
        if !input[0].same(&rule.marker) {
            return None;
        }
        let period = (0..height)
            .take_while(|y| input[*y * width].same(&rule.marker))
            .count();
        if period == 0 {
            return None;
        }
        let last = (0..height)
            .rev()
            .find(|y| (0..width).any(|x| !input[*y * width + x].same(&rule.background)))?;
        if last + 1 >= height {
            return None;
        }
        let mut output = input.to_vec();
        for y in last + 1..height {
            let source_y = (y - last - 1) % period;
            for x in 0..width {
                let value = &input[source_y * width + x];
                output[y * width + x] = if value.same(&rule.marker) {
                    rule.background.clone()
                } else {
                    value.clone()
                };
            }
        }
        Some(output)
    }

    fn induce_append_marker_period(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<AppendMarkerPeriod>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = AppendMarkerPeriod {
            background,
            marker: first[0].clone(),
        };
        if rule.marker.same(&rule.background) {
            return Ok(None);
        }
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_append_marker_period(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn square_candidates(
        height: usize,
        width: usize,
        input: &[Address],
        source: &Address,
    ) -> Vec<(usize, usize, [usize; 7])> {
        let occupied = |y: usize, x: usize| input[y * width + x].same(source);
        let positions = (0..height.saturating_sub(1))
            .flat_map(|y| {
                (0..width.saturating_sub(1)).filter_map(move |x| {
                    ([
                        occupied(y, x),
                        occupied(y + 1, x),
                        occupied(y, x + 1),
                        occupied(y + 1, x + 1),
                    ]
                    .into_iter()
                    .all(|value| value))
                    .then_some((y, x))
                })
            })
            .collect::<Vec<_>>();
        positions
            .iter()
            .map(|&(y, x)| {
                let cells = [(y, x), (y + 1, x), (y, x + 1), (y + 1, x + 1)];
                let perimeter = cells
                    .iter()
                    .flat_map(|&(ay, ax)| {
                        [
                            (ay as isize - 1, ax as isize),
                            (ay as isize + 1, ax as isize),
                            (ay as isize, ax as isize - 1),
                            (ay as isize, ax as isize + 1),
                        ]
                    })
                    .filter(|&(ay, ax)| {
                        ay < 0
                            || ax < 0
                            || ay >= height as isize
                            || ax >= width as isize
                            || !occupied(ay as usize, ax as usize)
                    })
                    .count();
                let overlaps = positions
                    .iter()
                    .filter(|&&(ay, ax)| ay.abs_diff(y) <= 1 && ax.abs_diff(x) <= 1)
                    .count()
                    - 1;
                let rows = positions
                    .iter()
                    .filter_map(|&(ay, ax)| (ay == y).then_some(ax))
                    .collect::<Vec<_>>();
                let columns = positions
                    .iter()
                    .filter_map(|&(ay, ax)| (ax == x).then_some(ay))
                    .collect::<Vec<_>>();
                let row_rank = rows.iter().filter(|&&ax| ax < x).count();
                let column_rank = columns.iter().filter(|&&ay| ay < y).count();
                let neighborhood = (y.saturating_sub(1)..=(y + 2).min(height - 1))
                    .flat_map(|ay| {
                        (x.saturating_sub(1)..=(x + 2).min(width - 1)).map(move |ax| (ay, ax))
                    })
                    .filter(|&(ay, ax)| occupied(ay, ax))
                    .count();
                (
                    y,
                    x,
                    [
                        perimeter,
                        overlaps,
                        rows.len(),
                        row_rank,
                        columns.len(),
                        column_rank,
                        neighborhood,
                    ],
                )
            })
            .collect()
    }

    fn render_square_partition(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &SquarePartition,
    ) -> Option<Vec<Address>> {
        let candidates = Self::square_candidates(height, width, input, &rule.source);
        if candidates.is_empty() {
            return None;
        }
        let mut output = input
            .iter()
            .map(|value| {
                if value.same(&rule.source) {
                    rule.remainder.clone()
                } else {
                    value.clone()
                }
            })
            .collect::<Vec<_>>();
        let mut used = std::collections::BTreeSet::new();
        for (y, x, features) in candidates {
            let learned = rule
                .examples
                .iter()
                .find_map(|(known, selected)| (known == &features).then_some(*selected));
            let cells = [(y, x), (y + 1, x), (y, x + 1), (y + 1, x + 1)];
            let selected = learned.unwrap_or_else(|| cells.iter().all(|cell| !used.contains(cell)));
            if selected {
                if cells.iter().any(|cell| used.contains(cell)) {
                    return None;
                }
                for cell in cells {
                    used.insert(cell);
                    output[cell.0 * width + cell.1] = rule.block.clone();
                }
            }
        }
        Some(output)
    }

    fn induce_square_partition(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<SquarePartition>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let mut sources = Vec::new();
        for value in &first {
            if !value.same(&background) && !sources.iter().any(|seen: &Address| seen.same(value)) {
                sources.push(value.clone());
            }
        }
        if sources.len() != 1 {
            return Ok(None);
        }
        let source = sources.remove(0);
        let first_expected = Self::grid_values(&lessons[0].output)?;
        let mut outputs = Vec::new();
        for value in &first_expected {
            if !value.same(&background) && !outputs.iter().any(|seen: &Address| seen.same(value)) {
                outputs.push(value.clone());
            }
        }
        if outputs.len() != 2 {
            return Ok(None);
        }
        for order in [[0usize, 1usize], [1, 0]] {
            let block = outputs[order[0]].clone();
            let remainder = outputs[order[1]].clone();
            let mut examples = Vec::new();
            let mut consistent = true;
            for lesson in lessons {
                let input = Self::grid_values(&lesson.input)?;
                let expected = Self::grid_values(&lesson.output)?;
                for (y, x, features) in Self::square_candidates(
                    lesson.input.height(),
                    lesson.input.width(),
                    &input,
                    &source,
                ) {
                    let cells = [(y, x), (y + 1, x), (y, x + 1), (y + 1, x + 1)];
                    let selected = cells
                        .iter()
                        .all(|&(ay, ax)| expected[ay * lesson.input.width() + ax].same(&block));
                    if let Some((_, known)) = examples
                        .iter()
                        .find(|(known, _): &&([usize; 7], bool)| known == &features)
                    {
                        if *known != selected {
                            consistent = false;
                            break;
                        }
                    } else {
                        examples.push((features, selected));
                    }
                }
                if !consistent {
                    break;
                }
            }
            if !consistent {
                continue;
            }
            let rule = SquarePartition {
                background: background.clone(),
                source: source.clone(),
                block,
                remainder,
                examples,
            };
            let mut valid = true;
            for lesson in lessons {
                let input = Self::grid_values(&lesson.input)?;
                let expected = Self::grid_values(&lesson.output)?;
                if Self::render_square_partition(
                    lesson.input.height(),
                    lesson.input.width(),
                    &input,
                    &rule,
                )
                .is_none_or(|actual| actual.iter().zip(&expected).any(|(a, b)| !a.same(b)))
                {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Ok(Some(rule));
            }
        }
        Ok(None)
    }

    fn render_marker_network(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &MarkerNetwork,
    ) -> Option<Vec<Address>> {
        let points = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(&rule.marker)
                    .then_some((index / width, index % width))
            })
            .collect::<std::collections::BTreeSet<_>>();
        if points.len() < 2 {
            return None;
        }
        let mut line = points.clone();
        let mut used = std::collections::BTreeSet::new();
        let mut horizontal_groups = Vec::new();
        for y in 0..height {
            let xs = points
                .iter()
                .filter_map(|&(ay, x)| (ay == y).then_some(x))
                .collect::<Vec<_>>();
            if xs.len() >= 2 {
                horizontal_groups.push((y, xs));
            }
        }
        if !horizontal_groups.is_empty() {
            for (y, xs) in &horizontal_groups {
                for x in xs[0]..=*xs.last()? {
                    line.insert((*y, x));
                }
                for &x in xs {
                    used.insert((*y, x));
                }
            }
        }
        for x in 0..width {
            let ys = points
                .iter()
                .filter_map(|&(y, ax)| (ax == x && !used.contains(&(y, ax))).then_some(y))
                .collect::<Vec<_>>();
            if ys.len() >= 2 {
                for y in ys[0]..=*ys.last()? {
                    line.insert((y, x));
                }
                for y in ys {
                    used.insert((y, x));
                }
            }
        }
        if horizontal_groups.is_empty() {
            for x in 0..width {
                let ys = points
                    .iter()
                    .filter_map(|&(y, ax)| (ax == x).then_some(y))
                    .collect::<Vec<_>>();
                if ys.len() >= 2 {
                    for y in ys[0]..=*ys.last()? {
                        line.insert((y, x));
                    }
                    for y in ys {
                        used.insert((y, x));
                    }
                }
            }
        }
        for &(y, x) in points.difference(&used) {
            let ys = line
                .iter()
                .filter_map(|&(ay, ax)| (ax == x).then_some(ay))
                .collect::<Vec<_>>();
            let xs = line
                .iter()
                .filter_map(|&(ay, ax)| (ay == y).then_some(ax))
                .collect::<Vec<_>>();
            if let Some(nearest) = ys.into_iter().min_by_key(|ay| ay.abs_diff(y)) {
                for ay in y.min(nearest)..=y.max(nearest) {
                    line.insert((ay, x));
                }
            } else if let Some(nearest) = xs.into_iter().min_by_key(|ax| ax.abs_diff(x)) {
                for ax in x.min(nearest)..=x.max(nearest) {
                    line.insert((y, ax));
                }
            }
        }
        let masked = input
            .iter()
            .map(|value| {
                if value.same(&rule.marker) {
                    rule.background.clone()
                } else {
                    value.clone()
                }
            })
            .collect::<Vec<_>>();
        let near = line
            .iter()
            .flat_map(|&(y, x)| {
                [
                    (y as isize, x as isize),
                    (y as isize - 1, x as isize),
                    (y as isize + 1, x as isize),
                    (y as isize, x as isize - 1),
                    (y as isize, x as isize + 1),
                ]
            })
            .filter(|&(y, x)| y >= 0 && x >= 0 && y < height as isize && x < width as isize)
            .map(|(y, x)| (y as usize, x as usize))
            .collect::<std::collections::BTreeSet<_>>();
        let mut output = input.to_vec();
        for component in Self::mixed_components(height, width, &masked, &rule.background) {
            if component.iter().any(|point| near.contains(point)) {
                for (y, x) in component {
                    output[y * width + x] = rule.marker.clone();
                }
            }
        }
        for (y, x) in line {
            output[y * width + x] = rule.marker.clone();
        }
        Some(output)
    }

    fn induce_marker_network(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<MarkerNetwork>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let mut counts: Vec<(Address, usize)> = Vec::new();
        for value in &first {
            if value.same(&background) {
                continue;
            }
            if let Some((_, count)) = counts.iter_mut().find(|(seen, _)| seen.same(value)) {
                *count += 1;
            } else {
                counts.push((value.clone(), 1));
            }
        }
        let marker = counts
            .into_iter()
            .min_by_key(|entry| entry.1)
            .map(|entry| entry.0);
        let Some(marker) = marker else {
            return Ok(None);
        };
        let rule = MarkerNetwork { background, marker };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_marker_network(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_axis_profile_projection(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &AxisProfileProjection,
    ) -> Option<Vec<Address>> {
        let rows = (0..height)
            .filter(|y| (0..width).all(|x| input[y * width + x].same(&rule.separator)))
            .collect::<Vec<_>>();
        let columns = (0..width)
            .filter(|x| (0..height).all(|y| input[y * width + x].same(&rule.separator)))
            .collect::<Vec<_>>();
        if rows.len() + columns.len() != 1 {
            return None;
        }
        let horizontal = !rows.is_empty();
        let at = if horizontal { rows[0] } else { columns[0] };
        let line_count = if horizontal { width } else { height };
        let axis_limit = if horizontal { height } else { width };
        let index = |line: usize, axis: usize| {
            if horizontal {
                axis * width + line
            } else {
                line * width + axis
            }
        };
        let mut output = input.to_vec();
        let mut changed = false;
        for line in 0..line_count {
            let objects = (0..axis_limit)
                .filter(|axis| input[index(line, *axis)].same(&rule.object))
                .collect::<Vec<_>>();
            if objects.is_empty() {
                continue;
            }
            let profiles = (0..axis_limit)
                .filter(|axis| input[index(line, *axis)].same(&rule.profile))
                .count();
            if profiles == 0 {
                return None;
            }
            for axis in 0..axis_limit {
                output[index(line, axis)] = rule.empty.clone();
            }
            for &axis in &objects {
                output[index(line, axis)] = rule.target.clone();
            }
            let low = *objects.iter().max()? < at;
            if low {
                for axis in objects.iter().max()? + 1..at {
                    output[index(line, axis)] = rule.object.clone();
                }
                output[index(line, at)] = rule.separator.clone();
                for axis in at + 1..axis_limit - profiles {
                    output[index(line, axis)] = rule.separator.clone();
                }
                for axis in axis_limit - profiles..axis_limit {
                    output[index(line, axis)] = rule.profile.clone();
                }
            } else {
                for axis in 0..profiles {
                    output[index(line, axis)] = rule.profile.clone();
                }
                for axis in profiles..=at {
                    output[index(line, axis)] = rule.separator.clone();
                }
                for axis in at + 1..*objects.iter().min()? {
                    output[index(line, axis)] = rule.object.clone();
                }
            }
            changed = true;
        }
        changed.then_some(output)
    }

    fn induce_axis_profile_projection(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<AxisProfileProjection>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(empty) = Self::background(&first) else {
            return Ok(None);
        };
        let mut palette: Vec<Address> = Vec::new();
        for value in &first {
            if !palette.iter().any(|seen| seen.same(value)) {
                palette.push(value.clone());
            }
        }
        let separator = palette
            .iter()
            .filter(|value| !value.same(&empty))
            .find(|value| {
                (0..lessons[0].input.height()).any(|y| {
                    (0..lessons[0].input.width())
                        .all(|x| first[y * lessons[0].input.width() + x].same(value))
                }) || (0..lessons[0].input.width()).any(|x| {
                    (0..lessons[0].input.height())
                        .all(|y| first[y * lessons[0].input.width() + x].same(value))
                })
            })
            .cloned();
        let Some(separator) = separator else {
            return Ok(None);
        };
        let changed = first.iter().zip(&expected).find(|(a, b)| {
            !a.same(b)
                && !expected
                    .iter()
                    .any(|value| value.same(a) && !first.iter().any(|seen| seen.same(value)))
        });
        let object_target = first.iter().zip(&expected).find_map(|(a, b)| {
            (!a.same(b)
                && !a.same(&empty)
                && !a.same(&separator)
                && !first.iter().any(|seen| seen.same(b)))
            .then_some((a.clone(), b.clone()))
        });
        let Some((object, target)) = object_target else {
            let _ = changed;
            return Ok(None);
        };
        let profile = palette
            .iter()
            .find(|value| !value.same(&empty) && !value.same(&separator) && !value.same(&object))
            .cloned();
        let Some(profile) = profile else {
            return Ok(None);
        };
        let rule = AxisProfileProjection {
            empty,
            object,
            profile,
            separator,
            target,
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_axis_profile_projection(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_separator_template(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &SeparatorTemplate,
    ) -> Option<Vec<Address>> {
        let mut palette: Vec<Address> = Vec::new();
        for value in input {
            if !palette.iter().any(|seen| seen.same(value)) {
                palette.push(value.clone());
            }
        }
        let specials = palette
            .into_iter()
            .filter(|value| {
                !value.same(&rule.empty)
                    && !value.same(&rule.regular)
                    && !value.same(&rule.separator)
            })
            .collect::<Vec<_>>();
        if specials.len() != 1 {
            return None;
        }
        let special = &specials[0];
        let points = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(special)
                    .then_some((index / width, index % width))
            })
            .collect::<Vec<_>>();
        let (y0, x0) = (
            points.iter().map(|p| p.0).min()?,
            points.iter().map(|p| p.1).min()?,
        );
        let mut mask = Vec::new();
        for y in (y0..height.saturating_sub(1)).step_by(2) {
            for x in (x0..width.saturating_sub(1)).step_by(2) {
                if [0usize, 1].into_iter().all(|dy| {
                    [0usize, 1]
                        .into_iter()
                        .all(|dx| input[(y + dy) * width + x + dx].same(special))
                }) {
                    let point = ((y - y0) / 2, (x - x0) / 2);
                    if point.0 >= 3 || point.1 >= 3 {
                        return None;
                    }
                    mask.push(point);
                }
            }
        }
        if mask.is_empty() {
            return None;
        }
        let mut output = input.to_vec();
        let mut changed = false;
        for y in (0..height).step_by(4) {
            for x in (0..width).step_by(4) {
                if mask.iter().all(|(dy, dx)| {
                    y + dy < height
                        && x + dx < width
                        && input[(y + dy) * width + x + dx].same(&rule.regular)
                }) {
                    for &(dy, dx) in &mask {
                        output[(y + dy) * width + x + dx] = special.clone();
                        changed = true;
                    }
                }
            }
        }
        changed.then_some(output)
    }

    fn induce_separator_template(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<SeparatorTemplate>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let mut colors: Vec<Address> = Vec::new();
        for value in &first {
            if !colors.iter().any(|seen| seen.same(value)) {
                colors.push(value.clone());
            }
        }
        for empty in &colors {
            for regular in &colors {
                for separator in &colors {
                    if empty.same(regular) || empty.same(separator) || regular.same(separator) {
                        continue;
                    }
                    let rule = SeparatorTemplate {
                        empty: empty.clone(),
                        regular: regular.clone(),
                        separator: separator.clone(),
                    };
                    if Self::render_separator_template(
                        lessons[0].input.height(),
                        lessons[0].input.width(),
                        &first,
                        &rule,
                    )
                    .is_some_and(|actual| actual.iter().zip(&expected).all(|(a, b)| a.same(b)))
                    {
                        let mut valid = true;
                        for lesson in lessons.iter().skip(1) {
                            let input = Self::grid_values(&lesson.input)?;
                            let wanted = Self::grid_values(&lesson.output)?;
                            if Self::render_separator_template(
                                lesson.input.height(),
                                lesson.input.width(),
                                &input,
                                &rule,
                            )
                            .is_none_or(|actual| {
                                actual.iter().zip(&wanted).any(|(a, b)| !a.same(b))
                            }) {
                                valid = false;
                                break;
                            }
                        }
                        if valid {
                            return Ok(Some(rule));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn render_missing_corner_paths(
        _height: usize,
        width: usize,
        input: &[Address],
        rule: &MissingCornerPaths,
    ) -> Option<Vec<Address>> {
        let points = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(&rule.marker)
                    .then_some((index / width, index % width))
            })
            .collect::<std::collections::BTreeSet<_>>();
        if points.len() < 3 {
            return None;
        }
        let envelope = CanonicalBoundingEnvelope::from_points(points.iter().copied())?;
        let corners = envelope.corners();
        let present = corners
            .into_iter()
            .filter(|point| points.contains(point))
            .collect::<Vec<_>>();
        if present.len() != 3 {
            return None;
        }
        let missing = corners.into_iter().find(|point| !points.contains(point))?;
        let mut output = input.to_vec();
        for x in envelope.left..=envelope.right {
            output[missing.0 * width + x] = rule.path.clone();
        }
        for y in envelope.top..=envelope.bottom {
            output[y * width + missing.1] = rule.path.clone();
        }
        let pairs = present
            .iter()
            .enumerate()
            .flat_map(|(i, a)| present.iter().skip(i + 1).map(move |b| (*a, *b)))
            .filter(|(a, b)| a.0 != b.0 && a.1 != b.1)
            .collect::<Vec<_>>();
        if pairs.len() != 1 {
            return None;
        }
        let (a, b) = pairs[0];
        let steps = a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
        for i in 0..=steps {
            let y = (a.0 as isize + (b.0 as isize - a.0 as isize) * i as isize / steps as isize)
                as usize;
            let x = (a.1 as isize + (b.1 as isize - a.1 as isize) * i as isize / steps as isize)
                as usize;
            output[y * width + x] = rule.path.clone();
        }
        for (y, x) in points {
            output[y * width + x] = rule.marker.clone();
        }
        Some(output)
    }

    fn induce_missing_corner_paths(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<MissingCornerPaths>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let marker = first.iter().find(|value| !value.same(&background)).cloned();
        let path = first
            .iter()
            .zip(&expected)
            .find_map(|(a, b)| (!a.same(b)).then_some(b.clone()));
        let (Some(marker), Some(path)) = (marker, path) else {
            return Ok(None);
        };
        let rule = MissingCornerPaths {
            background,
            marker,
            path,
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_missing_corner_paths(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_partial_crosses(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &PartialCrosses,
    ) -> Option<Vec<Address>> {
        let marker_grid = input
            .iter()
            .map(|value| {
                if value.same(&rule.marker) {
                    rule.marker.clone()
                } else {
                    rule.empty.clone()
                }
            })
            .collect::<Vec<_>>();
        let components = Self::same_color_components8(height, width, &marker_grid, &rule.empty);
        let mut output = input.to_vec();
        let mut changed = false;
        for (_, cells) in components {
            let markers = cells.into_iter().collect::<std::collections::BTreeSet<_>>();
            let ymin = markers.iter().map(|p| p.0).min()?;
            let ymax = markers.iter().map(|p| p.0).max()?;
            let xmin = markers.iter().map(|p| p.1).min()?;
            let xmax = markers.iter().map(|p| p.1).max()?;
            let mut options = Vec::new();
            for cy in ymin.saturating_sub(1)..=(ymax + 1).min(height - 1) {
                for cx in xmin.saturating_sub(1)..=(xmax + 1).min(width - 1) {
                    let cross = [
                        (cy as isize, cx as isize),
                        (cy as isize - 1, cx as isize),
                        (cy as isize + 1, cx as isize),
                        (cy as isize, cx as isize - 1),
                        (cy as isize, cx as isize + 1),
                    ];
                    if markers
                        .iter()
                        .all(|&(y, x)| cross.contains(&(y as isize, x as isize)))
                        && cross.iter().all(|&(y, x)| {
                            y < 0
                                || x < 0
                                || y >= height as isize
                                || x >= width as isize
                                || input[y as usize * width + x as usize].same(&rule.marker)
                                || input[y as usize * width + x as usize].same(&rule.noise)
                        })
                    {
                        options.push(cross);
                    }
                }
            }
            if options.len() != 1 {
                return None;
            }
            for &(y, x) in &options[0] {
                if y >= 0 && x >= 0 && y < height as isize && x < width as isize {
                    let index = y as usize * width + x as usize;
                    if input[index].same(&rule.noise) {
                        output[index] = rule.target.clone();
                        changed = true;
                    }
                }
            }
        }
        changed.then_some(output)
    }

    fn induce_partial_crosses(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<PartialCrosses>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(empty) = Self::background(&first) else {
            return Ok(None);
        };
        let change = first.iter().zip(&expected).find(|(a, b)| !a.same(b));
        let Some((noise, target)) = change else {
            return Ok(None);
        };
        let mut colors: Vec<(Address, usize)> = Vec::new();
        for value in &first {
            if value.same(&empty) {
                continue;
            }
            if let Some((_, n)) = colors.iter_mut().find(|(seen, _)| seen.same(value)) {
                *n += 1;
            } else {
                colors.push((value.clone(), 1));
            }
        }
        let marker = colors
            .into_iter()
            .min_by_key(|entry| entry.1)
            .map(|entry| entry.0);
        let Some(marker) = marker else {
            return Ok(None);
        };
        let rule = PartialCrosses {
            empty,
            noise: noise.clone(),
            marker,
            target: target.clone(),
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_partial_crosses(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_center_marker_transfer(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &CenterMarkerTransfer,
    ) -> Option<Vec<Address>> {
        let separator_rows = (0..height)
            .filter(|y| (0..width).all(|x| input[y * width + x].same(&rule.empty)))
            .collect::<Vec<_>>();
        let separator_columns = (0..width)
            .filter(|x| (0..height).all(|y| input[y * width + x].same(&rule.empty)))
            .collect::<Vec<_>>();
        if separator_rows.len() < 2 || separator_columns.len() < 2 {
            return None;
        }
        let groups = |limit: usize, separators: &[usize]| {
            let mut result = Vec::new();
            let mut start = 0;
            for stop in separators.iter().copied().chain(std::iter::once(limit)) {
                if start < stop {
                    result.push((start, stop));
                }
                start = stop + 1;
            }
            result
        };
        let rows = groups(height, &separator_rows);
        let columns = groups(width, &separator_columns);
        let targets = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(&rule.target)
                    .then_some((index / width, index % width))
            })
            .collect::<Vec<_>>();
        let markers = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(&rule.marker)
                    .then_some((index / width, index % width))
            })
            .collect::<Vec<_>>();
        if targets.len() != 1 || markers.len() != 1 {
            return None;
        }
        let (ty, tx) = targets[0];
        let (my, mx) = markers[0];
        let row_index = rows.iter().position(|(a, b)| (*a..*b).contains(&ty))?;
        let column_index = columns.iter().position(|(a, b)| (*a..*b).contains(&tx))?;
        let dy = (row_index as isize - (rows.len() / 2) as isize).signum();
        let dx = (column_index as isize - (columns.len() / 2) as isize).signum();
        let (ny, nx) = (my as isize + dy, mx as isize + dx);
        if ny < 0 || nx < 0 || ny >= height as isize || nx >= width as isize {
            return None;
        }
        let mut output = input.to_vec();
        output[my * width + mx] = rule.frame.clone();
        output[ty * width + tx] = rule.marker.clone();
        output[ny as usize * width + nx as usize] = rule.marker.clone();
        Some(output)
    }

    fn induce_center_marker_transfer(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<CenterMarkerTransfer>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let mut colors: Vec<Address> = Vec::new();
        for value in &first {
            if !colors.iter().any(|seen| seen.same(value)) {
                colors.push(value.clone());
            }
        }
        for marker in &colors {
            let marker_positions = first
                .iter()
                .enumerate()
                .filter_map(|(index, value)| value.same(marker).then_some(index))
                .collect::<Vec<_>>();
            if marker_positions.len() != 1 {
                continue;
            }
            let marker_index = marker_positions[0];
            if expected[marker_index].same(marker) {
                continue;
            }
            let frame = expected[marker_index].clone();
            for target in &colors {
                if target.same(marker) {
                    continue;
                }
                let target_positions = first
                    .iter()
                    .enumerate()
                    .filter_map(|(index, value)| value.same(target).then_some(index))
                    .collect::<Vec<_>>();
                if target_positions.len() != 1 || !expected[target_positions[0]].same(marker) {
                    continue;
                }
                for empty in &colors {
                    let rule = CenterMarkerTransfer {
                        empty: empty.clone(),
                        frame: frame.clone(),
                        marker: marker.clone(),
                        target: target.clone(),
                    };
                    let mut valid = true;
                    for lesson in lessons {
                        let input = Self::grid_values(&lesson.input)?;
                        let wanted = Self::grid_values(&lesson.output)?;
                        if Self::render_center_marker_transfer(
                            lesson.input.height(),
                            lesson.input.width(),
                            &input,
                            &rule,
                        )
                        .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
                        {
                            valid = false;
                            break;
                        }
                    }
                    if valid {
                        return Ok(Some(rule));
                    }
                }
            }
        }
        Ok(None)
    }

    fn render_tiled_mode_denoise(
        height: usize,
        width: usize,
        input: &[Address],
        background: &Address,
    ) -> Option<Vec<Address>> {
        if height < 20 || width < 20 {
            return None;
        }
        type TiledScore = (
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
        );
        let mut best: Option<(TiledScore, Vec<Address>)> = None;
        let py_end = 10.min(height / 2 + 1);
        let px_end = 10.min(width / 2 + 1);
        for py in 4..py_end {
            for px in 4..px_end {
                for oy in 0..py.min(height) {
                    for ox in 0..px.min(width) {
                        for nr in 2..=(height - oy) / py {
                            for nc in 2..=(width - ox) / px {
                                let mut template = Vec::with_capacity(py * px);
                                let mut confidence = 0;
                                for dy in 0..py {
                                    for dx in 0..px {
                                        let mut groups: Vec<(Address, usize)> = Vec::new();
                                        for ry in 0..nr {
                                            for rx in 0..nc {
                                                let value = &input[(oy + ry * py + dy) * width
                                                    + ox
                                                    + rx * px
                                                    + dx];
                                                if let Some((_, count)) = groups
                                                    .iter_mut()
                                                    .find(|(seen, _)| seen.same(value))
                                                {
                                                    *count += 1;
                                                } else {
                                                    groups.push((value.clone(), 1));
                                                }
                                            }
                                        }
                                        let mut groups = groups.into_iter();
                                        let mut selected = groups.next()?;
                                        for candidate in groups {
                                            if candidate.1 > selected.1 {
                                                selected = candidate;
                                            }
                                        }
                                        template.push(selected.0);
                                        confidence += selected.1;
                                    }
                                }
                                if template
                                    .iter()
                                    .filter(|value| !value.same(background))
                                    .count()
                                    < 4
                                {
                                    continue;
                                }
                                let mut output = vec![background.clone(); input.len()];
                                for ry in 0..nr {
                                    for rx in 0..nc {
                                        for dy in 0..py {
                                            for dx in 0..px {
                                                output[(oy + ry * py + dy) * width
                                                    + ox
                                                    + rx * px
                                                    + dx] = template[dy * px + dx].clone();
                                            }
                                        }
                                    }
                                }
                                let mismatch =
                                    output.iter().zip(input).filter(|(a, b)| !a.same(b)).count();
                                let key = (
                                    mismatch,
                                    usize::MAX - confidence,
                                    usize::MAX - nr * nc,
                                    py * px,
                                    oy,
                                    ox,
                                    py,
                                    px,
                                    nr,
                                    nc,
                                );
                                if best.as_ref().is_none_or(|(prior, _)| key < *prior) {
                                    best = Some((key, output));
                                }
                            }
                        }
                    }
                }
            }
        }
        let output = best?.1;
        output
            .iter()
            .zip(input)
            .any(|(a, b)| !a.same(b))
            .then_some(output)
    }

    fn induce_tiled_mode_denoise(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<TiledModeDenoise>, Error> {
        let first_output = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first_output) else {
            return Ok(None);
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_tiled_mode_denoise(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &background,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(TiledModeDenoise { background }))
    }

    fn render_adaptive_inset(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &AdaptiveInset,
    ) -> Option<Vec<Address>> {
        let points = Self::points(input, width, &rule.background);
        if points.len() != 3
            || points[0].2.same(&points[1].2)
            || points[0].2.same(&points[2].2)
            || points[1].2.same(&points[2].2)
        {
            return None;
        }
        let envelope = CanonicalBoundingEnvelope::from_points(
            points
                .iter()
                .map(|point| (point.0 as usize, point.1 as usize)),
        )?;
        let (cy, cx) = envelope.integral_center()?;
        let (cy, cx) = (cy as isize, cx as isize);
        let distance = (envelope.maximum_span() + 1) / 3;
        let mut output = input.to_vec();
        for (y, x, value) in points {
            let (ny, nx) = (
                y + (cy - y).signum() * distance as isize,
                x + (cx - x).signum() * distance as isize,
            );
            if ny < 0 || nx < 0 || ny >= height as isize || nx >= width as isize {
                return None;
            }
            output[ny as usize * width + nx as usize] = value;
        }
        output[cy as usize * width + cx as usize] = rule.target.clone();
        Some(output)
    }

    fn induce_adaptive_inset(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<AdaptiveInset>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let target = expected
            .iter()
            .find(|value| !first.iter().any(|seen| seen.same(value)))
            .cloned();
        let Some(target) = target else {
            return Ok(None);
        };
        let rule = AdaptiveInset { background, target };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_adaptive_inset(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_marker_crosses(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &MarkerCrosses,
    ) -> Option<Vec<Address>> {
        let background = Self::background(input)?;
        let points = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                (!value.same(&background)).then_some((index / width, index % width, value))
            })
            .collect::<Vec<_>>();
        if points.is_empty() || points.iter().any(|point| !point.2.same(&rule.marker)) {
            return None;
        }
        let projections = points
            .iter()
            .enumerate()
            .map(|(priority, (y, x, _))| CanonicalAxisProjection {
                priority,
                source: (*y, *x),
                axes: vec![CanonicalAxis::Horizontal, CanonicalAxis::Vertical],
                value: rule.marker.clone(),
                overwrite: CanonicalOverwrite::Replace,
            })
            .collect();
        let mut layers = Self::project_canonical_axes(height, width, projections)?;
        for &(y, x, _) in &points {
            layers.push(CanonicalMaskLayer {
                priority: layers.len(),
                mask: vec![y * width + x],
                value: rule.center.clone(),
                overwrite: CanonicalOverwrite::Replace,
            });
            let mut mask = [(-1isize, -1isize), (-1, 1), (1, -1), (1, 1)]
                .into_iter()
                .filter_map(|(dy, dx)| {
                    let (ay, ax) = (y as isize + dy, x as isize + dx);
                    (ay >= 0 && ax >= 0 && ay < height as isize && ax < width as isize)
                        .then(|| ay as usize * width + ax as usize)
                })
                .collect::<Vec<_>>();
            if rule.complete_diagonal_neighborhoods && mask.len() != 4 {
                mask.clear();
            }
            layers.push(CanonicalMaskLayer {
                priority: layers.len(),
                mask,
                value: rule.diagonal.clone(),
                overwrite: CanonicalOverwrite::Replace,
            });
        }
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_marker_crosses(&self, lessons: &[GroupLesson]) -> Result<Vec<MarkerCrosses>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first) else {
            return Ok(Vec::new());
        };
        let points = Self::points(&first, lessons[0].input.width(), &background);
        let Some((y, x, marker)) = points.first() else {
            return Ok(Vec::new());
        };
        if points.iter().any(|point| !point.2.same(marker)) {
            return Ok(Vec::new());
        }
        let center = expected[*y as usize * lessons[0].input.width() + *x as usize].clone();
        let diagonal_index = [(-1isize, -1isize), (-1, 1), (1, -1), (1, 1)]
            .into_iter()
            .find_map(|(dy, dx)| {
                let (ay, ax) = (*y + dy, *x + dx);
                (ay >= 0
                    && ax >= 0
                    && ay < lessons[0].input.height() as isize
                    && ax < lessons[0].input.width() as isize)
                    .then(|| ay as usize * lessons[0].input.width() + ax as usize)
            });
        let Some(diagonal) = diagonal_index.map(|index| expected[index].clone()) else {
            return Ok(Vec::new());
        };
        let mut candidates = Vec::new();
        for complete_diagonal_neighborhoods in [false, true] {
            let rule = MarkerCrosses {
                background: background.clone(),
                marker: marker.clone(),
                center: center.clone(),
                diagonal: diagonal.clone(),
                complete_diagonal_neighborhoods,
            };
            let mut valid = true;
            for lesson in lessons {
                let input = Self::grid_values(&lesson.input)?;
                let wanted = Self::grid_values(&lesson.output)?;
                if Self::render_marker_crosses(
                    lesson.input.height(),
                    lesson.input.width(),
                    &input,
                    &rule,
                )
                .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
                {
                    valid = false;
                    break;
                }
            }
            if valid {
                candidates.push(rule);
            }
        }
        Ok(candidates)
    }

    fn render_arrow_rays(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &ArrowRays,
    ) -> Option<Vec<Address>> {
        let background = Self::background(input)?;
        let components = Self::mixed_components(height, width, input, &background);
        let mut output = input.to_vec();
        let mut edges = Vec::new();
        for cells in components {
            let mut colors: Vec<(Address, usize)> = Vec::new();
            for &(y, x) in &cells {
                let value = &input[y * width + x];
                if let Some((_, count)) = colors.iter_mut().find(|(seen, _)| seen.same(value)) {
                    *count += 1;
                } else {
                    colors.push((value.clone(), 1));
                }
            }
            if colors.len() != 2 {
                continue;
            }
            colors.sort_by_key(|entry| entry.1);
            if colors[0].1 != 1 {
                continue;
            }
            let target = colors[0].0.clone();
            let (y, x) = cells
                .iter()
                .find(|(y, x)| input[*y * width + *x].same(&target))
                .copied()?;
            let mut options = Vec::new();
            for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                let (ay, ax, by, bx) = (
                    y as isize + dy,
                    x as isize + dx,
                    y as isize - dy,
                    x as isize - dx,
                );
                if ay >= 0
                    && ax >= 0
                    && by >= 0
                    && bx >= 0
                    && ay < height as isize
                    && by < height as isize
                    && ax < width as isize
                    && bx < width as isize
                    && input[ay as usize * width + ax as usize].same(&background)
                    && !input[by as usize * width + bx as usize].same(&background)
                {
                    options.push((dy, dx));
                }
            }
            if options.len() != 1 {
                return None;
            }
            let (dy, dx) = options[0];
            let (mut ay, mut ax) = (y as isize + 2 * dy, x as isize + 2 * dx);
            while ay >= 0 && ax >= 0 && ay < height as isize && ax < width as isize {
                output[ay as usize * width + ax as usize] = target.clone();
                ay += 2 * dy;
                ax += 2 * dx;
            }
            if dy != 0 {
                let edge = if dy < 0 { 0 } else { height - 1 };
                for bx in 0..width {
                    output[edge * width + bx] = target.clone();
                }
            } else {
                let edge = if dx < 0 { 0 } else { width - 1 };
                for by in 0..height {
                    output[by * width + edge] = target.clone();
                }
            }
            edges.push((dy, dx));
        }
        if edges.is_empty() {
            return None;
        }
        if edges.len() > 1 {
            for &(dy, _) in edges.iter().filter(|edge| edge.0 != 0) {
                for &(_, dx) in edges.iter().filter(|edge| edge.1 != 0) {
                    let y = if dy < 0 { 0 } else { height - 1 };
                    let x = if dx < 0 { 0 } else { width - 1 };
                    output[y * width + x] = rule.empty.clone();
                }
            }
        }
        Some(output)
    }

    fn induce_arrow_rays(&self, lessons: &[GroupLesson]) -> Result<Option<ArrowRays>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let expected = Self::grid_values(&lessons[0].output)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let empty = expected
            .iter()
            .find(|value| !first.iter().any(|seen| seen.same(value)))
            .cloned();
        let Some(empty) = empty else { return Ok(None) };
        let rule = ArrowRays { background, empty };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_arrow_rays(lesson.input.height(), lesson.input.width(), &input, &rule)
                .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_legend_blocks(
        _height: usize,
        width: usize,
        input: &[Address],
        rule: &LegendBlocks,
    ) -> Option<Vec<Address>> {
        let legend = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                (!value.same(&rule.background) && !value.same(&rule.carrier)).then_some((
                    index / width,
                    index % width,
                    value.clone(),
                ))
            })
            .collect::<Vec<_>>();
        let mask = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(&rule.carrier)
                    .then_some((index / width, index % width))
            })
            .collect::<Vec<_>>();
        if legend.is_empty() || mask.is_empty() {
            return None;
        }
        let (ly0, ly1) = (
            legend.iter().map(|p| p.0).min()?,
            legend.iter().map(|p| p.0).max()?,
        );
        let (lx0, lx1) = (
            legend.iter().map(|p| p.1).min()?,
            legend.iter().map(|p| p.1).max()?,
        );
        let (my0, my1) = (
            mask.iter().map(|p| p.0).min()?,
            mask.iter().map(|p| p.0).max()?,
        );
        let (mx0, mx1) = (
            mask.iter().map(|p| p.1).min()?,
            mask.iter().map(|p| p.1).max()?,
        );
        let mut size = 0;
        for y in my0..=my1 {
            let mut start = None;
            for x in mx0..=mx1 + 1 {
                let occupied = x <= mx1 && input[y * width + x].same(&rule.carrier);
                if occupied && start.is_none() {
                    start = Some(x);
                }
                if !occupied && let Some(at) = start.take() {
                    size = Self::gcd(size, x - at);
                }
            }
        }
        for x in mx0..=mx1 {
            let mut start = None;
            for y in my0..=my1 + 1 {
                let occupied = y <= my1 && input[y * width + x].same(&rule.carrier);
                if occupied && start.is_none() {
                    start = Some(y);
                }
                if !occupied && let Some(at) = start.take() {
                    size = Self::gcd(size, y - at);
                }
            }
        }
        if size < 2 {
            return None;
        }
        let (lh, lw) = (ly1 - ly0 + 1, lx1 - lx0 + 1);
        let (mh, mw) = ((my1 - my0 + 1) / size, (mx1 - mx0 + 1) / size);
        if mh != lw || mw != lh {
            return None;
        }
        let mut output = input.to_vec();
        let mut nodes = 0;
        for by in 0..mh {
            for bx in 0..mw {
                let block = (my0 + by * size..my0 + (by + 1) * size).all(|y| {
                    (mx0 + bx * size..mx0 + (bx + 1) * size)
                        .all(|x| input[y * width + x].same(&rule.carrier))
                });
                let source_y = ly1.checked_sub(bx)?;
                let source_x = lx0 + by;
                let color = &input[source_y * width + source_x];
                let present = !color.same(&rule.background) && !color.same(&rule.carrier);
                if block != present {
                    return None;
                }
                if block {
                    nodes += 1;
                    for y in my0 + by * size..my0 + (by + 1) * size {
                        for x in mx0 + bx * size..mx0 + (bx + 1) * size {
                            output[y * width + x] = color.clone();
                        }
                    }
                }
            }
        }
        (nodes == legend.len()).then_some(output)
    }

    fn induce_legend_blocks(&self, lessons: &[GroupLesson]) -> Result<Option<LegendBlocks>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let mut groups: Vec<(Address, usize)> = Vec::new();
        for value in &first {
            if value.same(&background) {
                continue;
            }
            if let Some((_, count)) = groups.iter_mut().find(|(seen, _)| seen.same(value)) {
                *count += 1;
            } else {
                groups.push((value.clone(), 1));
            }
        }
        let Some(carrier) = groups
            .into_iter()
            .max_by_key(|entry| entry.1)
            .map(|entry| entry.0)
        else {
            return Ok(None);
        };
        let rule = LegendBlocks {
            background,
            carrier,
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_legend_blocks(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_reflect_internal_motif(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &ReflectInternalMotif,
    ) -> Option<Vec<Address>> {
        let mut layers = Vec::new();
        for (color, cells) in Self::same_color_components(height, width, input, &rule.background) {
            if !color.same(&rule.frame) {
                continue;
            }
            let (y0, y1) = (
                cells.iter().map(|p| p.0).min()?,
                cells.iter().map(|p| p.0).max()?,
            );
            let (x0, x1) = (
                cells.iter().map(|p| p.1).min()?,
                cells.iter().map(|p| p.1).max()?,
            );
            let motif = (y0 + 1..y1)
                .flat_map(|y| {
                    (x0 + 1..x1).filter_map(move |x| {
                        input[y * width + x].same(&rule.motif).then_some((y, x))
                    })
                })
                .collect::<Vec<_>>();
            if motif.is_empty() {
                continue;
            }
            let motif_cells = motif
                .iter()
                .map(|&(y, x)| (y as isize, x as isize, rule.motif.clone()))
                .collect::<Vec<_>>();
            let mut options = Vec::new();
            for (dihedral, translation) in
                [(2, ((y0 + y1) as isize, 0)), (1, (0, (x0 + x1) as isize))]
            {
                let projection = Self::project_canonical_role_transforms(
                    height,
                    width,
                    vec![CanonicalRoleTransform {
                        priority: 0,
                        dihedral,
                        translation,
                        cells: motif_cells.clone(),
                        overwrite: CanonicalOverwrite::Replace,
                    }],
                )?
                .pop()?;
                let additions = projection
                    .cells
                    .into_iter()
                    .filter(|cell| {
                        let point = (cell.target / width, cell.target % width);
                        !motif.contains(&point)
                    })
                    .collect::<Vec<_>>();
                if !additions.is_empty()
                    && additions.iter().all(|cell| {
                        input[cell.target].same(&rule.background)
                            || input[cell.target].same(&rule.motif)
                    })
                {
                    options.push((additions, projection.overwrite));
                }
            }
            if options.len() != 1 {
                return None;
            }
            let (additions, overwrite) = options.pop()?;
            for cell in additions {
                layers.push(CanonicalMaskLayer {
                    priority: layers.len(),
                    mask: vec![cell.target],
                    value: cell.value,
                    overwrite: overwrite.clone(),
                });
            }
        }
        if layers.is_empty() {
            return None;
        }
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_reflect_internal_motif(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<ReflectInternalMotif>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let mut groups: Vec<(Address, usize)> = Vec::new();
        for value in &first {
            if value.same(&background) {
                continue;
            }
            if let Some((_, count)) = groups.iter_mut().find(|(seen, _)| seen.same(value)) {
                *count += 1;
            } else {
                groups.push((value.clone(), 1));
            }
        }
        if groups.len() != 2 {
            return Ok(None);
        }
        groups.sort_by_key(|group| group.1);
        let rule = ReflectInternalMotif {
            background,
            motif: groups[0].0.clone(),
            frame: groups[1].0.clone(),
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let wanted = Self::grid_values(&lesson.output)?;
            if Self::render_reflect_internal_motif(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            )
            .is_none_or(|actual| actual.iter().zip(&wanted).any(|(a, b)| !a.same(b)))
            {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_ordinal_column_fill(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &OrdinalColumnFill,
    ) -> Option<Vec<Address>> {
        let selected = (0..width)
            .filter(|x| !input[(height - 1) * width + x].same(&rule.background))
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return None;
        }
        let mut output = input.to_vec();
        let mut changed = false;
        for x in selected {
            let markers = (0..height)
                .filter_map(|y| {
                    let value = &input[y * width + x];
                    (!value.same(&rule.background)).then_some((y, value.clone()))
                })
                .collect::<Vec<_>>();
            let mut previous = None;
            for (y, color) in markers {
                for ay in previous.map_or(0, |value| value + 1)..=y {
                    let index = ay * width + x;
                    if output[index].same(&rule.background) {
                        output[index] = color.clone();
                        changed = true;
                    }
                }
                previous = Some(y);
            }
        }
        changed.then_some(output)
    }

    fn induce_ordinal_column_fill(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<OrdinalColumnFill>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = OrdinalColumnFill { background };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            let Some(actual) = Self::render_ordinal_column_fill(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            ) else {
                return Ok(None);
            };
            if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn same_color_components(
        height: usize,
        width: usize,
        input: &[Address],
        background: &Address,
    ) -> Vec<(Address, Vec<(usize, usize)>)> {
        let mut seen = vec![false; input.len()];
        let mut result = Vec::new();
        for start in 0..input.len() {
            if seen[start] || input[start].same(background) {
                continue;
            }
            let color = input[start].clone();
            seen[start] = true;
            let mut queue = std::collections::VecDeque::from([(start / width, start % width)]);
            let mut cells = Vec::new();
            while let Some((y, x)) = queue.pop_front() {
                cells.push((y, x));
                for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let (ay, ax) = (y as isize + dy, x as isize + dx);
                    if ay < 0 || ax < 0 || ay >= height as isize || ax >= width as isize {
                        continue;
                    }
                    let index = ay as usize * width + ax as usize;
                    if !seen[index] && input[index].same(&color) {
                        seen[index] = true;
                        queue.push_back((ay as usize, ax as usize));
                    }
                }
            }
            result.push((color, cells));
        }
        result
    }

    fn gcd(mut a: usize, mut b: usize) -> usize {
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    }

    fn project_canonical_cardinal_segments(
        height: usize,
        width: usize,
        segments: Vec<CanonicalCardinalSegment>,
    ) -> Option<Vec<CanonicalMaskLayer>> {
        segments
            .into_iter()
            .map(|segment| {
                if segment.start.0 >= height
                    || segment.start.1 >= width
                    || segment.end.0 >= height
                    || segment.end.1 >= width
                    || (segment.start.0 != segment.end.0 && segment.start.1 != segment.end.1)
                {
                    return None;
                }
                let mut mask = Vec::new();
                let (mut y, mut x) = (segment.start.0 as isize, segment.start.1 as isize);
                let end = (segment.end.0 as isize, segment.end.1 as isize);
                let step = ((end.0 - y).signum(), (end.1 - x).signum());
                loop {
                    mask.push(y as usize * width + x as usize);
                    if (y, x) == end {
                        break;
                    }
                    y += step.0;
                    x += step.1;
                }
                Some(CanonicalMaskLayer {
                    priority: segment.priority,
                    mask,
                    value: segment.value,
                    overwrite: segment.overwrite,
                })
            })
            .collect()
    }

    fn project_canonical_stencils(
        height: usize,
        width: usize,
        mut placements: Vec<CanonicalStencilPlacement>,
        bounds: CanonicalStencilBounds,
        overlap: CanonicalStencilOverlap,
    ) -> Option<Vec<CanonicalMaskLayer>> {
        placements.sort_by_key(|placement| placement.priority);
        if placements
            .windows(2)
            .any(|pair| pair[0].priority == pair[1].priority)
        {
            return None;
        }
        let mut ordered = Vec::new();
        let mut non_conflicting =
            std::collections::BTreeMap::<usize, (Address, CanonicalOverwrite)>::new();
        for placement in placements {
            for (dy, dx, value) in placement.cells {
                let y = placement.anchor.0.checked_add(dy)?;
                let x = placement.anchor.1.checked_add(dx)?;
                if y < 0 || x < 0 || y >= height as isize || x >= width as isize {
                    match bounds {
                        CanonicalStencilBounds::Reject => return None,
                        CanonicalStencilBounds::Clip => continue,
                    }
                }
                let index = y as usize * width + x as usize;
                match overlap {
                    CanonicalStencilOverlap::Ordered => ordered.push(CanonicalMaskLayer {
                        priority: ordered.len(),
                        mask: vec![index],
                        value,
                        overwrite: placement.overwrite.clone(),
                    }),
                    CanonicalStencilOverlap::RejectDifferentValues => {
                        if let Some((prior, _)) = non_conflicting.get(&index) {
                            if !prior.same(&value) {
                                return None;
                            }
                        } else {
                            non_conflicting.insert(index, (value, placement.overwrite.clone()));
                        }
                    }
                }
            }
        }
        if matches!(overlap, CanonicalStencilOverlap::RejectDifferentValues) {
            ordered.extend(
                non_conflicting
                    .into_iter()
                    .map(|(index, (value, overwrite))| CanonicalMaskLayer {
                        priority: index,
                        mask: vec![index],
                        value,
                        overwrite,
                    }),
            );
        }
        Some(ordered)
    }

    fn canonical_dihedral(index: usize, y: isize, x: isize) -> Option<(isize, isize)> {
        match index {
            0 => Some((y, x)),
            1 => Some((y, x.checked_neg()?)),
            2 => Some((y.checked_neg()?, x)),
            3 => Some((y.checked_neg()?, x.checked_neg()?)),
            4 => Some((x, y)),
            5 => Some((x, y.checked_neg()?)),
            6 => Some((x.checked_neg()?, y)),
            7 => Some((x.checked_neg()?, y.checked_neg()?)),
            _ => None,
        }
    }

    fn project_canonical_role_transforms(
        height: usize,
        width: usize,
        mut transforms: Vec<CanonicalRoleTransform>,
    ) -> Option<Vec<CanonicalRoleTransformProjection>> {
        transforms.sort_by_key(|transform| transform.priority);
        if transforms
            .windows(2)
            .any(|pair| pair[0].priority == pair[1].priority)
        {
            return None;
        }
        transforms
            .into_iter()
            .map(|transform| {
                let cells = transform
                    .cells
                    .into_iter()
                    .map(|(y, x, value)| {
                        if y < 0 || x < 0 || y >= height as isize || x >= width as isize {
                            return None;
                        }
                        let transformed = Self::canonical_dihedral(transform.dihedral, y, x)?;
                        let target_y = transformed.0.checked_add(transform.translation.0)?;
                        let target_x = transformed.1.checked_add(transform.translation.1)?;
                        if target_y < 0
                            || target_x < 0
                            || target_y >= height as isize
                            || target_x >= width as isize
                        {
                            return None;
                        }
                        Some(CanonicalTransformedRoleCell {
                            source: y as usize * width + x as usize,
                            target: target_y as usize * width + target_x as usize,
                            value,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(CanonicalRoleTransformProjection {
                    cells,
                    overwrite: transform.overwrite,
                })
            })
            .collect()
    }

    fn project_canonical_axes(
        height: usize,
        width: usize,
        projections: Vec<CanonicalAxisProjection>,
    ) -> Option<Vec<CanonicalMaskLayer>> {
        projections
            .into_iter()
            .map(|projection| {
                let (y, x) = projection.source;
                if y >= height || x >= width || projection.axes.is_empty() {
                    return None;
                }
                let mut mask = Vec::new();
                for axis in projection.axes {
                    match axis {
                        CanonicalAxis::Horizontal => {
                            mask.extend((0..width).map(|at| y * width + at));
                        }
                        CanonicalAxis::Vertical => {
                            mask.extend((0..height).map(|at| at * width + x));
                        }
                    }
                }
                mask.sort_unstable();
                mask.dedup();
                Some(CanonicalMaskLayer {
                    priority: projection.priority,
                    mask,
                    value: projection.value,
                    overwrite: projection.overwrite,
                })
            })
            .collect()
    }

    fn compose_canonical_masks(
        input_snapshot: &[Address],
        mut layers: Vec<CanonicalMaskLayer>,
    ) -> Option<Vec<Address>> {
        layers.sort_by_key(|layer| layer.priority);
        if layers
            .windows(2)
            .any(|pair| pair[0].priority == pair[1].priority)
        {
            return None;
        }
        let mut output = input_snapshot.to_vec();
        for layer in layers {
            for index in layer.mask {
                let current = output.get_mut(index)?;
                let applies = match &layer.overwrite {
                    CanonicalOverwrite::Replace => true,
                    CanonicalOverwrite::IfCurrentMatches(expected) => current.same(expected),
                };
                if applies {
                    *current = layer.value.clone();
                }
            }
        }
        Some(output)
    }

    fn render_component_pair_fill(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &ComponentPairFill,
    ) -> Option<Vec<Address>> {
        let mut items = Vec::new();
        for (color, cells) in Self::same_color_components(height, width, input, &rule.background) {
            if cells.len() > 16 {
                continue;
            }
            let y0 = cells.iter().map(|p| p.0).min()?;
            let x0 = cells.iter().map(|p| p.1).min()?;
            let mut shape = cells
                .iter()
                .map(|(y, x)| (y - y0, x - x0))
                .collect::<Vec<_>>();
            shape.sort_unstable();
            items.push((color, shape, (y0, x0)));
        }
        if items.len() < 2 {
            return None;
        }
        let mut ys = items.iter().map(|item| item.2.0).collect::<Vec<_>>();
        let mut xs = items.iter().map(|item| item.2.1).collect::<Vec<_>>();
        ys.sort_unstable();
        ys.dedup();
        xs.sort_unstable();
        xs.dedup();
        let mut step = 0;
        for values in [&ys, &xs] {
            for pair in values.windows(2) {
                step = Self::gcd(step, pair[1] - pair[0]);
            }
        }
        if step == 0 {
            return None;
        }
        let mut layers = Vec::new();
        for i in 0..items.len() {
            for j in i + 1..items.len() {
                if !items[i].0.same(&items[j].0) || items[i].1 != items[j].1 {
                    continue;
                }
                let (first, second) = (items[i].2, items[j].2);
                let anchors = if first.0 == second.0 && first.1.abs_diff(second.1) > step {
                    (first.1.min(second.1) + step..first.1.max(second.1))
                        .step_by(step)
                        .map(|x| (first.0, x))
                        .collect::<Vec<_>>()
                } else if first.1 == second.1 && first.0.abs_diff(second.0) > step {
                    (first.0.min(second.0) + step..first.0.max(second.0))
                        .step_by(step)
                        .map(|y| (y, first.1))
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                for (y0, x0) in anchors {
                    let mask = items[i]
                        .1
                        .iter()
                        .map(|&(dy, dx)| {
                            let (y, x) = (y0 + dy, x0 + dx);
                            (y < height && x < width).then_some(y * width + x)
                        })
                        .collect::<Option<Vec<_>>>()?;
                    layers.push(CanonicalMaskLayer {
                        priority: layers.len(),
                        mask,
                        value: items[i].0.clone(),
                        overwrite: CanonicalOverwrite::IfCurrentMatches(rule.background.clone()),
                    });
                }
            }
        }
        let output = Self::compose_canonical_masks(input, layers)?;
        output
            .iter()
            .zip(input)
            .any(|(after, before)| !after.same(before))
            .then_some(output)
    }

    fn induce_component_pair_fill(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<ComponentPairFill>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let background = first.first().cloned().ok_or(Error::Incomplete)?;
        let rule = ComponentPairFill { background };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            let Some(actual) = Self::render_component_pair_fill(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            ) else {
                return Ok(None);
            };
            if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_paired_lines(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &PairedLines,
    ) -> Option<Vec<Address>> {
        let points = Self::points(input, width, &rule.background);
        let mut groups: Vec<(Address, Vec<(usize, usize)>)> = Vec::new();
        for (y, x, color) in points {
            if let Some((_, positions)) = groups.iter_mut().find(|(seen, _)| seen.same(&color)) {
                positions.push((y as usize, x as usize));
            } else {
                groups.push((color, vec![(y as usize, x as usize)]));
            }
        }
        if groups.is_empty() || groups.iter().any(|(_, positions)| positions.len() != 2) {
            return None;
        }
        let segments = groups
            .into_iter()
            .enumerate()
            .map(|(priority, (color, positions))| CanonicalCardinalSegment {
                priority,
                start: positions[0],
                end: positions[1],
                value: color,
                overwrite: CanonicalOverwrite::IfCurrentMatches(rule.background.clone()),
            })
            .collect();
        let layers = Self::project_canonical_cardinal_segments(height, width, segments)?;
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_paired_lines(&self, lessons: &[GroupLesson]) -> Result<Option<PairedLines>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = PairedLines { background };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            let Some(actual) = Self::render_paired_lines(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
            ) else {
                return Ok(None);
            };
            if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn mixed_components(
        height: usize,
        width: usize,
        input: &[Address],
        background: &Address,
    ) -> Vec<Vec<(usize, usize)>> {
        let mut seen = vec![false; input.len()];
        let mut result = Vec::new();
        for start in 0..input.len() {
            if seen[start] || input[start].same(background) {
                continue;
            }
            seen[start] = true;
            let mut queue = std::collections::VecDeque::from([(start / width, start % width)]);
            let mut component = Vec::new();
            while let Some((y, x)) = queue.pop_front() {
                component.push((y, x));
                for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let (ay, ax) = (y as isize + dy, x as isize + dx);
                    if ay < 0 || ax < 0 || ay >= height as isize || ax >= width as isize {
                        continue;
                    }
                    let index = ay as usize * width + ax as usize;
                    if !seen[index] && !input[index].same(background) {
                        seen[index] = true;
                        queue.push_back((ay as usize, ax as usize));
                    }
                }
            }
            component.sort_unstable();
            result.push(component);
        }
        result
    }

    fn component_signature(
        width: usize,
        input: &[Address],
        component: &[(usize, usize)],
    ) -> ComponentSignature {
        let anchor = (
            component.iter().map(|(y, _)| *y).min().unwrap_or(0),
            component.iter().map(|(_, x)| *x).min().unwrap_or(0),
        );
        let mut roles = Vec::<Address>::new();
        let mut body = Vec::new();
        for &(y, x) in component {
            let value = input[y * width + x].clone();
            let role = roles
                .iter()
                .position(|seen| seen.same(&value))
                .unwrap_or_else(|| {
                    roles.push(value);
                    roles.len() - 1
                });
            body.push((
                y as isize - anchor.0 as isize,
                x as isize - anchor.1 as isize,
                role,
            ));
        }
        (body, roles, anchor)
    }

    fn render_component_stencil(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &ComponentStencil,
        bounds: CanonicalStencilBounds,
    ) -> Option<Vec<Address>> {
        let components = Self::mixed_components(height, width, input, &rule.background);
        if components.is_empty() {
            return None;
        }
        let signatures = components
            .iter()
            .map(|component| Self::component_signature(width, input, component))
            .collect::<Vec<_>>();
        if signatures.iter().any(|(body, _, _)| body != &rule.template) {
            return None;
        }
        let placements = signatures
            .into_iter()
            .enumerate()
            .map(|(priority, (_, roles, anchor))| {
                let cells = rule
                    .effects
                    .iter()
                    .map(|&(dy, dx, role)| Some((dy, dx, roles.get(role)?.clone())))
                    .collect::<Option<Vec<_>>>()?;
                Some(CanonicalStencilPlacement {
                    priority,
                    anchor: (anchor.0 as isize, anchor.1 as isize),
                    cells,
                    overwrite: CanonicalOverwrite::Replace,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let layers = Self::project_canonical_stencils(
            height,
            width,
            placements,
            bounds,
            CanonicalStencilOverlap::RejectDifferentValues,
        )?;
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_component_stencil(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<ComponentStencil>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let mut template = None;
        let mut effects = None;
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            let components = Self::mixed_components(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &background,
            );
            if components.is_empty() {
                return Ok(None);
            }
            let signatures = components
                .iter()
                .map(|component| Self::component_signature(lesson.input.width(), &input, component))
                .collect::<Vec<_>>();
            if signatures
                .iter()
                .skip(1)
                .any(|signature| signature.0 != signatures[0].0)
            {
                return Ok(None);
            }
            if let Some(current) = &template {
                if current != &signatures[0].0 {
                    return Ok(None);
                }
            } else {
                template = Some(signatures[0].0.clone());
            }
            let mut assigned = vec![Vec::new(); components.len()];
            for index in 0..input.len() {
                if input[index].same(&expected[index]) {
                    continue;
                }
                let point = (index / lesson.input.width(), index % lesson.input.width());
                let distances = components
                    .iter()
                    .map(|component| {
                        component
                            .iter()
                            .map(|(y, x)| y.abs_diff(point.0) + x.abs_diff(point.1))
                            .min()
                            .unwrap_or(usize::MAX)
                    })
                    .collect::<Vec<_>>();
                let minimum = *distances.iter().min().unwrap_or(&usize::MAX);
                if distances
                    .iter()
                    .filter(|distance| **distance == minimum)
                    .count()
                    != 1
                {
                    return Ok(None);
                }
                let owner = distances
                    .iter()
                    .position(|distance| *distance == minimum)
                    .unwrap();
                let (_, roles, anchor) = &signatures[owner];
                let Some(role) = roles.iter().position(|value| value.same(&expected[index])) else {
                    return Ok(None);
                };
                assigned[owner].push((
                    point.0 as isize - anchor.0 as isize,
                    point.1 as isize - anchor.1 as isize,
                    role,
                ));
            }
            for body in &mut assigned {
                body.sort_unstable();
            }
            if assigned.iter().skip(1).any(|body| body != &assigned[0]) || assigned[0].is_empty() {
                return Ok(None);
            }
            if let Some(current) = &effects {
                if current != &assigned[0] {
                    return Ok(None);
                }
            } else {
                effects = Some(assigned[0].clone());
            }
        }
        let rule = ComponentStencil {
            background,
            template: template.ok_or(Error::Incomplete)?,
            effects: effects.ok_or(Error::Incomplete)?,
        };
        for lesson in lessons {
            let input = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            let Some(actual) = Self::render_component_stencil(
                lesson.input.height(),
                lesson.input.width(),
                &input,
                &rule,
                CanonicalStencilBounds::Reject,
            ) else {
                return Ok(None);
            };
            if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_boundary_rays(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &BoundaryRays,
    ) -> Option<Vec<Address>> {
        let edge_cells = [
            (-1isize, 0isize, (0..width).collect::<Vec<_>>()),
            (1, 0, (0..width).map(|x| (height - 1) * width + x).collect()),
            (0, -1, (0..height).map(|y| y * width).collect()),
            (0, 1, (0..height).map(|y| y * width + width - 1).collect()),
        ];
        let mut edges = Vec::new();
        for (dy, dx, cells) in edge_cells {
            let color = input[cells[0]].clone();
            if !color.same(&rule.background) && cells.iter().all(|index| input[*index].same(&color))
            {
                edges.push((dy, dx, color));
            }
        }
        if edges.len() != 1 {
            return None;
        }
        let (dy, dx, boundary) = edges.pop()?;
        let mut points = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                (!value.same(&rule.background) && !value.same(&boundary)).then_some((
                    index / width,
                    index % width,
                    value.clone(),
                ))
            })
            .collect::<Vec<_>>();
        if points.is_empty() {
            return None;
        }
        let distance = |y: usize, x: usize| match (dy, dx) {
            (-1, 0) => y,
            (1, 0) => height - 1 - y,
            (0, -1) => x,
            _ => width - 1 - x,
        };
        points.sort_by_key(|(y, x, _)| std::cmp::Reverse(distance(*y, *x)));
        let originals = points.iter().map(|(y, x, _)| (*y, *x)).collect::<Vec<_>>();
        let mut output = input.to_vec();
        for (y, x, value) in points {
            let (mut ay, mut ax) = (y as isize + dy, x as isize + dx);
            while ay >= 0 && ax >= 0 && ay < height as isize && ax < width as isize {
                let index = ay as usize * width + ax as usize;
                if input[index].same(&boundary) {
                    break;
                }
                if !originals.contains(&(ay as usize, ax as usize)) {
                    output[index] = value.clone();
                }
                ay += dy;
                ax += dx;
            }
        }
        Some(output)
    }

    fn induce_boundary_rays(&self, lessons: &[GroupLesson]) -> Result<Option<BoundaryRays>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = BoundaryRays { background };
        for lesson in lessons {
            let source = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            let Some(actual) = Self::render_boundary_rays(
                lesson.input.height(),
                lesson.input.width(),
                &source,
                &rule,
            ) else {
                return Ok(None);
            };
            if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn render_alternating_stripes(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &AlternatingStripes,
    ) -> Option<Vec<Address>> {
        let mut points = Self::points(input, width, &rule.background);
        if points.len() != 2 || points[0].2.same(&points[1].2) {
            return None;
        }
        let y_difference = (points[0].0 - points[1].0).unsigned_abs();
        let x_difference = (points[0].1 - points[1].1).unsigned_abs();
        let row_axis = y_difference != 0 && (x_difference == 0 || y_difference < x_difference);
        if row_axis {
            points.sort_by_key(|point| point.0);
        } else {
            points.sort_by_key(|point| point.1);
        }
        let first = if row_axis { points[0].0 } else { points[0].1 };
        let second = if row_axis { points[1].0 } else { points[1].1 };
        let step = (second - first) as usize;
        if step == 0 {
            return None;
        }
        let limit = if row_axis { height } else { width };
        let mut output = input.to_vec();
        let mut at = first as usize;
        let mut role = 0usize;
        while at < limit {
            if row_axis {
                for x in 0..width {
                    output[at * width + x] = points[role % 2].2.clone();
                }
            } else {
                for y in 0..height {
                    output[y * width + at] = points[role % 2].2.clone();
                }
            }
            at += step;
            role += 1;
        }
        Some(output)
    }

    fn induce_alternating_stripes(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<AlternatingStripes>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let rule = AlternatingStripes { background };
        for lesson in lessons {
            let source = Self::grid_values(&lesson.input)?;
            let expected = Self::grid_values(&lesson.output)?;
            let Some(actual) = Self::render_alternating_stripes(
                lesson.input.height(),
                lesson.input.width(),
                &source,
                &rule,
            ) else {
                return Ok(None);
            };
            if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                return Ok(None);
            }
        }
        Ok(Some(rule))
    }

    fn generate_canonical_periodic_axis(axis: CanonicalPeriodicAxis) -> Option<Vec<usize>> {
        let step = isize::try_from(axis.step).ok()?;
        if step == 0 || axis.lower < 0 || axis.upper < axis.lower {
            return None;
        }
        let mut coordinates = Vec::new();
        for coordinate in axis.lower..=axis.upper {
            if coordinate.checked_sub(axis.phase)?.rem_euclid(step) == 0 {
                coordinates.push(usize::try_from(coordinate).ok()?);
            }
        }
        Some(coordinates)
    }

    fn generate_canonical_periodic_lattice(
        lattice: CanonicalPeriodicLattice,
    ) -> Option<Vec<(usize, usize)>> {
        let rows = Self::generate_canonical_periodic_axis(lattice.rows)?;
        let columns = Self::generate_canonical_periodic_axis(lattice.columns)?;
        Some(
            rows.into_iter()
                .flat_map(|y| columns.iter().copied().map(move |x| (y, x)))
                .collect(),
        )
    }

    fn render_sparse_lattice(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &SparseLattice,
    ) -> Option<Vec<Address>> {
        let points = Self::points(input, width, &rule.background);
        if points.is_empty() {
            return None;
        }
        let envelope = CanonicalBoundingEnvelope::from_points(
            points
                .iter()
                .map(|point| (point.0 as usize, point.1 as usize)),
        )?;
        let (y0, y1, x0, x1) = (
            envelope.top as isize,
            envelope.bottom as isize,
            envelope.left as isize,
            envelope.right as isize,
        );
        let mut groups: Vec<(Address, Vec<(isize, isize)>)> = Vec::new();
        for (y, x, color) in points {
            if let Some((_, positions)) = groups.iter_mut().find(|(seen, _)| seen.same(&color)) {
                positions.push((y, x));
            } else {
                groups.push((color, vec![(y, x)]));
            }
        }
        let least = groups.iter().map(|(_, positions)| positions.len()).min()?;
        let center_twice = (y0 + y1, x0 + x1);
        let selected = groups
            .into_iter()
            .filter_map(|(color, positions)| {
                if positions.len() != least {
                    return None;
                }
                positions
                    .into_iter()
                    .find(|(y, x)| (2 * *y, 2 * *x) != center_twice)
                    .map(|position| (color, position))
            })
            .collect::<Vec<_>>();
        if selected.len() != 1 {
            return None;
        }
        let (color, (sy, sx)) = &selected[0];
        let coordinates = Self::generate_canonical_periodic_lattice(CanonicalPeriodicLattice {
            rows: CanonicalPeriodicAxis {
                lower: y0,
                upper: y1,
                phase: *sy,
                step: rule.step,
            },
            columns: CanonicalPeriodicAxis {
                lower: x0,
                upper: x1,
                phase: *sx,
                step: rule.step,
            },
        })?;
        let mut output = input.to_vec();
        let mut changed = false;
        for (y, x) in coordinates {
            let index = y * width + x;
            if output[index].same(&rule.background) {
                output[index] = color.clone();
                changed = true;
            }
        }
        (changed && output.len() == height * width).then_some(output)
    }

    fn induce_sparse_lattice(
        &self,
        lessons: &[GroupLesson],
    ) -> Result<Option<SparseLattice>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        for step in 2..=5 {
            let rule = SparseLattice {
                background: background.clone(),
                step,
            };
            let mut valid = true;
            for lesson in lessons {
                let source = Self::grid_values(&lesson.input)?;
                let expected = Self::grid_values(&lesson.output)?;
                let Some(actual) = Self::render_sparse_lattice(
                    lesson.input.height(),
                    lesson.input.width(),
                    &source,
                    &rule,
                ) else {
                    valid = false;
                    break;
                };
                if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Ok(Some(rule));
            }
        }
        Ok(None)
    }

    fn render_move_to_touch(
        height: usize,
        width: usize,
        input: &[Address],
        rule: &MoveToTouch,
    ) -> Option<Vec<Address>> {
        let moving = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(&rule.moving)
                    .then_some((index / width, index % width))
            })
            .collect::<Vec<_>>();
        let anchors = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value
                    .same(&rule.anchor)
                    .then_some((index / width, index % width))
            })
            .collect::<Vec<_>>();
        if moving.is_empty() || anchors.is_empty() {
            return None;
        }
        let occupied = input
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                (!value.same(&rule.background) && !value.same(&rule.moving))
                    .then_some((index / width, index % width))
            })
            .collect::<Vec<_>>();
        let mut options = Vec::new();
        for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
            for distance in 1..=height.max(width) {
                let shifted = moving
                    .iter()
                    .map(|(y, x)| {
                        (
                            *y as isize + dy * distance as isize,
                            *x as isize + dx * distance as isize,
                        )
                    })
                    .collect::<Vec<_>>();
                if shifted
                    .iter()
                    .any(|(y, x)| *y < 0 || *x < 0 || *y >= height as isize || *x >= width as isize)
                {
                    break;
                }
                if shifted
                    .iter()
                    .any(|(y, x)| occupied.contains(&(*y as usize, *x as usize)))
                {
                    break;
                }
                let adjacent = shifted.iter().any(|(y, x)| {
                    [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
                        .iter()
                        .any(|(ay, ax)| {
                            let point = (*y + ay, *x + ax);
                            point.0 >= 0
                                && point.1 >= 0
                                && anchors.contains(&(point.0 as usize, point.1 as usize))
                        })
                });
                if adjacent {
                    options.push(shifted);
                    break;
                }
            }
        }
        if options.len() != 1 {
            return None;
        }
        let mut output = input.to_vec();
        for (y, x) in moving {
            output[y * width + x] = rule.background.clone();
        }
        for (y, x) in options.pop()? {
            output[y as usize * width + x as usize] = rule.moving.clone();
        }
        Some(output)
    }

    fn induce_move_to_touch(&self, lessons: &[GroupLesson]) -> Result<Option<MoveToTouch>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let mut colors = Vec::<Address>::new();
        for value in &first {
            if !value.same(&background) && !colors.iter().any(|seen| seen.same(value)) {
                colors.push(value.clone());
            }
        }
        for moving in &colors {
            for anchor in &colors {
                if moving.same(anchor) {
                    continue;
                }
                let rule = MoveToTouch {
                    background: background.clone(),
                    moving: moving.clone(),
                    anchor: anchor.clone(),
                };
                let mut valid = true;
                for lesson in lessons {
                    let source = Self::grid_values(&lesson.input)?;
                    let expected = Self::grid_values(&lesson.output)?;
                    let Some(actual) = Self::render_move_to_touch(
                        lesson.input.height(),
                        lesson.input.width(),
                        &source,
                        &rule,
                    ) else {
                        valid = false;
                        break;
                    };
                    if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    return Ok(Some(rule));
                }
            }
        }
        Ok(None)
    }

    fn render_beams(width: usize, input: &[Address], rule: &Beams) -> Option<Vec<Address>> {
        let height = input.len().checked_div(width)?;
        let points = Self::points(input, width, &rule.background);
        if points.is_empty()
            || points
                .iter()
                .any(|(_, _, value)| !rule.roles.iter().any(|(role, _)| role.same(value)))
            || rule
                .roles
                .iter()
                .any(|(role, _)| !points.iter().any(|(_, _, value)| role.same(value)))
        {
            return None;
        }
        let mut projections = Vec::new();
        for wanted in [BeamMode::Vertical, BeamMode::Horizontal] {
            for (y, x, value) in &points {
                let mode = rule.roles.iter().find(|(role, _)| role.same(value))?.1;
                if mode != wanted {
                    continue;
                }
                let axis = match mode {
                    BeamMode::Vertical => CanonicalAxis::Vertical,
                    BeamMode::Horizontal => CanonicalAxis::Horizontal,
                    BeamMode::None => continue,
                };
                projections.push(CanonicalAxisProjection {
                    priority: projections.len(),
                    source: (*y as usize, *x as usize),
                    axes: vec![axis],
                    value: value.clone(),
                    overwrite: CanonicalOverwrite::Replace,
                });
            }
        }
        let mut layers = Self::project_canonical_axes(height, width, projections)?;
        for (y, x, value) in points {
            layers.push(CanonicalMaskLayer {
                priority: layers.len(),
                mask: vec![y as usize * width + x as usize],
                value,
                overwrite: CanonicalOverwrite::Replace,
            });
        }
        Self::compose_canonical_masks(input, layers)
    }

    fn induce_beams(&self, lessons: &[GroupLesson]) -> Result<Option<Beams>, Error> {
        let first = Self::grid_values(&lessons[0].input)?;
        let Some(background) = Self::background(&first) else {
            return Ok(None);
        };
        let points = Self::points(&first, lessons[0].input.width(), &background);
        let mut colors = Vec::<Address>::new();
        for (_, _, color) in points {
            if !colors.iter().any(|seen| seen.same(&color)) {
                colors.push(color);
            }
        }
        if colors.is_empty() || colors.len() > 4 {
            return Ok(None);
        }
        let combinations = 3usize.pow(colors.len() as u32);
        for encoded in 1..combinations {
            let mut value = encoded;
            let mut roles = Vec::with_capacity(colors.len());
            for color in &colors {
                let mode = match value % 3 {
                    0 => BeamMode::None,
                    1 => BeamMode::Horizontal,
                    _ => BeamMode::Vertical,
                };
                roles.push((color.clone(), mode));
                value /= 3;
            }
            let rule = Beams {
                background: background.clone(),
                roles,
            };
            let mut valid = true;
            for lesson in lessons {
                let source = Self::grid_values(&lesson.input)?;
                let expected = Self::grid_values(&lesson.output)?;
                let Some(actual) = Self::render_beams(lesson.input.width(), &source, &rule) else {
                    valid = false;
                    break;
                };
                if actual.iter().zip(&expected).any(|(a, b)| !a.same(b)) {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Ok(Some(rule));
            }
        }
        Ok(None)
    }

    fn structural_entries(root: &Address) -> Result<Vec<StructuralEntry>, Error> {
        let integer = |entry: &Address, field_name: &[u8]| -> Result<Vec<u8>, Error> {
            field(entry, field_name)?
                .inspect_children()
                .into_iter()
                .next()
                .and_then(|address| address.token().map(ToOwned::to_owned))
                .ok_or(Error::Incomplete)
        };
        root.inspect_children()
            .iter()
            .map(|entry| {
                let dy = i64::from_be_bytes(
                    integer(entry, b"dy")?
                        .try_into()
                        .map_err(|_| Error::Incomplete)?,
                ) as isize;
                let dx = i64::from_be_bytes(
                    integer(entry, b"dx")?
                        .try_into()
                        .map_err(|_| Error::Incomplete)?,
                ) as isize;
                let role = usize::from_be_bytes(
                    integer(entry, b"role")?
                        .try_into()
                        .map_err(|_| Error::Incomplete)?,
                );
                Ok((dy, dx, role))
            })
            .collect()
    }

    fn materialize_broad_grid(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
        height: usize,
        width: usize,
        output: &[Address],
    ) -> Result<Grid, Error> {
        if output.len() != height.saturating_mul(width) {
            return Err(Error::Shape);
        }
        let root = self
            .root
            .provision(b"broad-structural-results")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        let mut rows = Vec::with_capacity(height);
        for y in 0..height {
            let row = root.provision(b"rows").provision(&y.to_be_bytes());
            for x in 0..width {
                row.provision(b"cells")
                    .provision(&x.to_be_bytes())
                    .provision(b"value")
                    .point_to(&output[y * width + x]);
            }
            rows.push(row);
        }
        self.frozen = true;
        Ok(s.derived_extent(input, &program.root, &rows, width)?)
    }

    pub(super) fn apply_broad_structural(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<Grid, Error> {
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        let values = Self::grid_values(input)?;
        if program
            .root
            .child(b"canonical-whole-grid-transform")
            .is_some()
        {
            if program.root.child(b"separator-compaction").is_some() {
                let (height, width, output) =
                    Self::render_separator_compaction(input.height(), input.width(), &values)
                        .ok_or(Error::NotReady)?;
                return self.materialize_broad_grid(s, input, program, height, width, &output);
            }
            if let Some(reduction) = program.root.child(b"block-reduction") {
                let read = |name: &[u8]| -> Result<usize, Error> {
                    let wrapper = field(&reduction, name)?;
                    let bytes = wrapper
                        .inspect_children()
                        .first()
                        .and_then(|node| node.token().map(ToOwned::to_owned))
                        .ok_or(Error::Incomplete)?;
                    Ok(usize::from_be_bytes(
                        bytes.as_slice().try_into().map_err(|_| Error::Incomplete)?,
                    ))
                };
                let block_y = read(b"height")?;
                let block_x = read(b"width")?;
                let uniform = reduction.child(b"uniform").is_some();
                let (height, width, output) = Self::render_block_reduction(
                    input.height(),
                    input.width(),
                    &values,
                    block_y,
                    block_x,
                    uniform,
                )
                .ok_or(Error::NotReady)?;
                return self.materialize_broad_grid(s, input, program, height, width, &output);
            }
            if let Some(extraction) = program.root.child(b"object-extraction") {
                let background = field(&extraction, b"background")?.follow()?;
                let diagonal = extraction.child(b"eight-connected").is_some();
                let smallest = extraction.child(b"smallest").is_some();
                let (height, width, output) = Self::render_object_extraction(
                    input.height(),
                    input.width(),
                    &values,
                    &background,
                    diagonal,
                    smallest,
                )
                .ok_or(Error::NotReady)?;
                return self.materialize_broad_grid(s, input, program, height, width, &output);
            }
            if let Some(entries) = program.root.child(b"palette-map") {
                let mapping = entries
                    .inspect_children()
                    .into_iter()
                    .map(|entry| {
                        Ok((
                            field(&entry, b"source")?.follow()?,
                            field(&entry, b"target")?.follow()?,
                        ))
                    })
                    .collect::<Result<Vec<(Address, Address)>, Error>>()?;
                let output = values
                    .iter()
                    .map(|source| {
                        mapping
                            .iter()
                            .find(|(seen, _)| seen.same(source))
                            .map(|(_, target)| target.clone())
                            .ok_or(Error::NotReady)
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                return self.materialize_broad_grid(
                    s,
                    input,
                    program,
                    input.height(),
                    input.width(),
                    &output,
                );
            }
            let transform = if let Some(root) = program.root.child(b"dihedral") {
                let kind = root
                    .inspect_children()
                    .first()
                    .and_then(|node| node.token())
                    .and_then(|bytes| bytes.first())
                    .copied()
                    .ok_or(Error::Incomplete)?;
                CanonicalWholeGridTransform::Dihedral(kind)
            } else if let Some(root) = program.root.child(b"scale") {
                let y = root
                    .inspect_children()
                    .first()
                    .cloned()
                    .ok_or(Error::Incomplete)?;
                let x = y
                    .inspect_children()
                    .first()
                    .cloned()
                    .ok_or(Error::Incomplete)?;
                let sy = usize::from_be_bytes(
                    y.token()
                        .ok_or(Error::Incomplete)?
                        .try_into()
                        .map_err(|_| Error::Incomplete)?,
                );
                let sx = usize::from_be_bytes(
                    x.token()
                        .ok_or(Error::Incomplete)?
                        .try_into()
                        .map_err(|_| Error::Incomplete)?,
                );
                CanonicalWholeGridTransform::Scale(sy, sx)
            } else if let Some(root) = program.root.child(b"tile") {
                let y = root
                    .inspect_children()
                    .first()
                    .cloned()
                    .ok_or(Error::Incomplete)?;
                let x = y
                    .inspect_children()
                    .first()
                    .cloned()
                    .ok_or(Error::Incomplete)?;
                let ry = usize::from_be_bytes(
                    y.token()
                        .ok_or(Error::Incomplete)?
                        .try_into()
                        .map_err(|_| Error::Incomplete)?,
                );
                let rx = usize::from_be_bytes(
                    x.token()
                        .ok_or(Error::Incomplete)?
                        .try_into()
                        .map_err(|_| Error::Incomplete)?,
                );
                CanonicalWholeGridTransform::Tile(ry, rx)
            } else if program.root.child(b"crop-majority").is_some() {
                CanonicalWholeGridTransform::CropMajority
            } else {
                return Err(Error::Incomplete);
            };
            let (height, width, output) = Self::render_whole_grid_transform(
                input.height(),
                input.width(),
                &values,
                transform,
            )
            .ok_or(Error::NotReady)?;
            return self.materialize_broad_grid(s, input, program, height, width, &output);
        }
        let output = if program.root.child(b"progression-construction").is_some() {
            let rule = Progression {
                background: field(&program.root, b"background")?.follow()?,
                source: field(&program.root, b"source")?.follow()?,
                target: field(&program.root, b"target")?.follow()?,
                reverse: program.root.child(b"reverse").is_some(),
            };
            Self::render_progression(input.height(), input.width(), &values, &rule)
        } else if program
            .root
            .child(b"orthogonal-connector-construction")
            .is_some()
        {
            let rule = Connector {
                background: field(&program.root, b"background")?.follow()?,
                anchor: field(&program.root, b"anchor")?.follow()?,
                first: field(&program.root, b"first")?.follow()?,
                second: field(&program.root, b"second")?.follow()?,
                target: field(&program.root, b"target")?.follow()?,
                first_horizontal: program.root.child(b"first-horizontal").is_some(),
                second_horizontal: program.root.child(b"second-horizontal").is_some(),
            };
            Self::render_connector(input.width(), &values, &rule)
        } else if program
            .root
            .child(b"nearest-marker-envelope-construction")
            .is_some()
        {
            Self::render_nearest_marker_envelope(
                input.height(),
                input.width(),
                &values,
                &NearestMarkerEnvelope {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"lattice-separator-construction")
            .is_some()
        {
            Self::render_lattice_separators(
                input.height(),
                input.width(),
                &values,
                &LatticeSeparators {
                    background: field(&program.root, b"background")?.follow()?,
                    inner: field(&program.root, b"inner")?.follow()?,
                    outer: field(&program.root, b"outer")?.follow()?,
                },
            )
        } else if program.root.child(b"corner-halo-construction").is_some() {
            Self::render_corner_halo(
                input.height(),
                input.width(),
                &values,
                &CornerHalo {
                    background: field(&program.root, b"background")?.follow()?,
                    target: field(&program.root, b"target")?.follow()?,
                },
            )
        } else if program.root.child(b"corner-pixel-construction").is_some() {
            Self::render_corner_pixels(
                input.height(),
                input.width(),
                &values,
                &CornerPixels {
                    background: field(&program.root, b"background")?.follow()?,
                    convex: field(&program.root, b"convex")?.follow()?,
                    concave: field(&program.root, b"concave")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"periodic-diamond-construction")
            .is_some()
        {
            let period = field(&program.root, b"period")?
                .inspect_children()
                .into_iter()
                .next()
                .and_then(|address| address.token().map(ToOwned::to_owned))
                .and_then(|bytes| bytes.try_into().ok())
                .map(usize::from_be_bytes)
                .ok_or(Error::Incomplete)?;
            Self::render_periodic_diamonds(
                input.height(),
                input.width(),
                &values,
                &PeriodicDiamonds {
                    background: field(&program.root, b"background")?.follow()?,
                    period,
                },
            )
        } else if program
            .root
            .child(b"glyph-classifier-construction")
            .is_some()
        {
            let mut mapping = Vec::new();
            for entry in field(&program.root, b"mapping")?.inspect_children() {
                let pattern = field(&entry, b"pattern")?
                    .inspect_children()
                    .into_iter()
                    .next()
                    .and_then(|address| address.token().map(ToOwned::to_owned))
                    .ok_or(Error::Incomplete)?;
                mapping.push((pattern, field(&entry, b"color")?.follow()?));
            }
            Self::render_glyph_classifier(
                input.height(),
                input.width(),
                &values,
                &GlyphClassifier {
                    background: field(&program.root, b"background")?.follow()?,
                    mapping,
                },
            )
        } else if program.root.child(b"marker-cross-construction").is_some() {
            Self::render_marker_crosses(
                input.height(),
                input.width(),
                &values,
                &MarkerCrosses {
                    background: field(&program.root, b"background")?.follow()?,
                    marker: field(&program.root, b"marker")?.follow()?,
                    center: field(&program.root, b"center")?.follow()?,
                    diagonal: field(&program.root, b"diagonal")?.follow()?,
                    complete_diagonal_neighborhoods: program
                        .root
                        .child(b"complete-diagonal-neighborhoods")
                        .is_some(),
                },
            )
        } else if program.root.child(b"arrow-ray-construction").is_some() {
            Self::render_arrow_rays(
                input.height(),
                input.width(),
                &values,
                &ArrowRays {
                    background: field(&program.root, b"background")?.follow()?,
                    empty: field(&program.root, b"empty")?.follow()?,
                },
            )
        } else if program.root.child(b"legend-block-construction").is_some() {
            Self::render_legend_blocks(
                input.height(),
                input.width(),
                &values,
                &LegendBlocks {
                    background: field(&program.root, b"background")?.follow()?,
                    carrier: field(&program.root, b"carrier")?.follow()?,
                },
            )
        } else if program.root.child(b"template-ray-construction").is_some() {
            Self::render_template_rays(
                input.height(),
                input.width(),
                &values,
                &TemplateRays {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"relocate-diagram-construction")
            .is_some()
        {
            Self::render_relocate_diagrams(
                input.height(),
                input.width(),
                &values,
                &RelocateDiagrams {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"append-marker-period-construction")
            .is_some()
        {
            Self::render_append_marker_period(
                input.height(),
                input.width(),
                &values,
                &AppendMarkerPeriod {
                    background: field(&program.root, b"background")?.follow()?,
                    marker: field(&program.root, b"marker")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"square-partition-construction")
            .is_some()
        {
            let mut examples = Vec::new();
            for entry in field(&program.root, b"examples")?.inspect_children() {
                let token = field(&entry, b"features")?
                    .inspect_children()
                    .into_iter()
                    .next()
                    .and_then(|address| address.token().map(ToOwned::to_owned))
                    .ok_or(Error::Incomplete)?;
                if token.len() != 7 * std::mem::size_of::<usize>() {
                    return Err(Error::Incomplete);
                }
                let mut features = [0usize; 7];
                for (index, chunk) in token.chunks_exact(std::mem::size_of::<usize>()).enumerate() {
                    features[index] =
                        usize::from_be_bytes(chunk.try_into().map_err(|_| Error::Incomplete)?);
                }
                examples.push((features, entry.child(b"selected").is_some()));
            }
            Self::render_square_partition(
                input.height(),
                input.width(),
                &values,
                &SquarePartition {
                    background: field(&program.root, b"background")?.follow()?,
                    source: field(&program.root, b"source")?.follow()?,
                    block: field(&program.root, b"block")?.follow()?,
                    remainder: field(&program.root, b"remainder")?.follow()?,
                    examples,
                },
            )
        } else if program.root.child(b"marker-network-construction").is_some() {
            Self::render_marker_network(
                input.height(),
                input.width(),
                &values,
                &MarkerNetwork {
                    background: field(&program.root, b"background")?.follow()?,
                    marker: field(&program.root, b"marker")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"axis-profile-projection-construction")
            .is_some()
        {
            Self::render_axis_profile_projection(
                input.height(),
                input.width(),
                &values,
                &AxisProfileProjection {
                    empty: field(&program.root, b"empty")?.follow()?,
                    object: field(&program.root, b"object")?.follow()?,
                    profile: field(&program.root, b"profile")?.follow()?,
                    separator: field(&program.root, b"separator")?.follow()?,
                    target: field(&program.root, b"target")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"separator-template-construction")
            .is_some()
        {
            Self::render_separator_template(
                input.height(),
                input.width(),
                &values,
                &SeparatorTemplate {
                    empty: field(&program.root, b"empty")?.follow()?,
                    regular: field(&program.root, b"regular")?.follow()?,
                    separator: field(&program.root, b"separator")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"missing-corner-path-construction")
            .is_some()
        {
            Self::render_missing_corner_paths(
                input.height(),
                input.width(),
                &values,
                &MissingCornerPaths {
                    background: field(&program.root, b"background")?.follow()?,
                    marker: field(&program.root, b"marker")?.follow()?,
                    path: field(&program.root, b"path")?.follow()?,
                },
            )
        } else if program.root.child(b"partial-cross-construction").is_some() {
            Self::render_partial_crosses(
                input.height(),
                input.width(),
                &values,
                &PartialCrosses {
                    empty: field(&program.root, b"empty")?.follow()?,
                    noise: field(&program.root, b"noise")?.follow()?,
                    marker: field(&program.root, b"marker")?.follow()?,
                    target: field(&program.root, b"target")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"center-marker-transfer-construction")
            .is_some()
        {
            Self::render_center_marker_transfer(
                input.height(),
                input.width(),
                &values,
                &CenterMarkerTransfer {
                    empty: field(&program.root, b"empty")?.follow()?,
                    frame: field(&program.root, b"frame")?.follow()?,
                    marker: field(&program.root, b"marker")?.follow()?,
                    target: field(&program.root, b"target")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"tiled-mode-denoise-construction")
            .is_some()
        {
            Self::render_tiled_mode_denoise(
                input.height(),
                input.width(),
                &values,
                &field(&program.root, b"background")?.follow()?,
            )
        } else if program.root.child(b"adaptive-inset-construction").is_some() {
            Self::render_adaptive_inset(
                input.height(),
                input.width(),
                &values,
                &AdaptiveInset {
                    background: field(&program.root, b"background")?.follow()?,
                    target: field(&program.root, b"target")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"reflect-internal-motif-construction")
            .is_some()
        {
            Self::render_reflect_internal_motif(
                input.height(),
                input.width(),
                &values,
                &ReflectInternalMotif {
                    background: field(&program.root, b"background")?.follow()?,
                    frame: field(&program.root, b"frame")?.follow()?,
                    motif: field(&program.root, b"motif")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"ordinal-column-fill-construction")
            .is_some()
        {
            Self::render_ordinal_column_fill(
                input.height(),
                input.width(),
                &values,
                &OrdinalColumnFill {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"component-pair-fill-construction")
            .is_some()
        {
            Self::render_component_pair_fill(
                input.height(),
                input.width(),
                &values,
                &ComponentPairFill {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program.root.child(b"paired-line-construction").is_some() {
            Self::render_paired_lines(
                input.height(),
                input.width(),
                &values,
                &PairedLines {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"component-stencil-construction")
            .is_some()
        {
            Self::render_component_stencil(
                input.height(),
                input.width(),
                &values,
                &ComponentStencil {
                    background: field(&program.root, b"background")?.follow()?,
                    template: Self::structural_entries(&field(&program.root, b"template")?)?,
                    effects: Self::structural_entries(&field(&program.root, b"effects")?)?,
                },
                if program.root.child(b"clip-at-grid-boundary").is_some() {
                    CanonicalStencilBounds::Clip
                } else {
                    CanonicalStencilBounds::Reject
                },
            )
        } else if program.root.child(b"boundary-ray-construction").is_some() {
            Self::render_boundary_rays(
                input.height(),
                input.width(),
                &values,
                &BoundaryRays {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program
            .root
            .child(b"alternating-stripe-construction")
            .is_some()
        {
            Self::render_alternating_stripes(
                input.height(),
                input.width(),
                &values,
                &AlternatingStripes {
                    background: field(&program.root, b"background")?.follow()?,
                },
            )
        } else if program.root.child(b"sparse-lattice-construction").is_some() {
            let step = field(&program.root, b"step")?
                .inspect_children()
                .into_iter()
                .next()
                .and_then(|address| address.token().map(ToOwned::to_owned))
                .and_then(|bytes| bytes.try_into().ok())
                .map(usize::from_be_bytes)
                .ok_or(Error::Incomplete)?;
            Self::render_sparse_lattice(
                input.height(),
                input.width(),
                &values,
                &SparseLattice {
                    background: field(&program.root, b"background")?.follow()?,
                    step,
                },
            )
        } else if program.root.child(b"move-to-touch-construction").is_some() {
            Self::render_move_to_touch(
                input.height(),
                input.width(),
                &values,
                &MoveToTouch {
                    background: field(&program.root, b"background")?.follow()?,
                    moving: field(&program.root, b"moving")?.follow()?,
                    anchor: field(&program.root, b"anchor")?.follow()?,
                },
            )
        } else if program.root.child(b"axis-beam-construction").is_some() {
            let mut roles = Vec::new();
            for role in field(&program.root, b"beam-roles")?.inspect_children() {
                let mode = if role.child(b"horizontal").is_some() {
                    BeamMode::Horizontal
                } else if role.child(b"vertical").is_some() {
                    BeamMode::Vertical
                } else {
                    BeamMode::None
                };
                roles.push((field(&role, b"color")?.follow()?, mode));
            }
            Self::render_beams(
                input.width(),
                &values,
                &Beams {
                    background: field(&program.root, b"background")?.follow()?,
                    roles,
                },
            )
        } else {
            return Err(Error::Incomplete);
        }
        .ok_or(Error::NotReady)?;
        self.materialize_broad_grid(s, input, program, input.height(), input.width(), &output)
    }
}
