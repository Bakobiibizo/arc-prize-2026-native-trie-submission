use native_trie_seed::{
    Address,
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GeneralizedCandidateOrigin, SparseRewrite},
};

type Pixels = Vec<Vec<Option<u8>>>;
type PairedRole = (u8, (usize, usize), (usize, usize));

fn admit(s: &Grids, id: &str, pixels: &Pixels, side: Side) -> Grid {
    let source = s
        .source(id.as_bytes(), b"broad structural induction fixture")
        .unwrap();
    s.admit(id.as_bytes(), side, pixels, &source).unwrap()
}

fn assert_canonical_mask_composition(
    root: &Address,
    provenance: &[u8],
    relation: &[u8],
    geometry: &[u8],
    overwrite: &[u8],
) {
    let canonical = root
        .child(b"canonical-mask-composition")
        .expect("old schema must expose its canonical representation");
    assert!(
        canonical
            .child(b"roles")
            .and_then(|roles| roles.child(b"source-bound-colours"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"relations")
            .and_then(|relations| relations.child(relation))
            .is_some()
    );
    assert!(
        canonical
            .child(b"geometry")
            .and_then(|geometry_root| geometry_root.child(geometry))
            .is_some()
    );
    assert!(
        canonical
            .child(b"masks")
            .and_then(|masks| masks.child(b"query-derived-cell-sets"))
            .is_some()
    );
    let composition = canonical.child(b"composition").unwrap();
    assert!(composition.child(b"input-snapshot").is_some());
    assert!(composition.child(b"ascending-explicit-priority").is_some());
    assert!(
        canonical
            .child(b"overwrite-priority")
            .and_then(|priority| priority.child(overwrite))
            .is_some()
    );
    assert!(
        canonical
            .child(b"old-schema-provenance")
            .and_then(|old| old.child(provenance))
            .is_some()
    );
}

fn assert_canonical_axis_projection(
    root: &Address,
    provenance: &[u8],
    relation: &[u8],
    axes: &[u8],
    overlays: &[u8],
    overwrite: &[u8],
) {
    let canonical = root
        .child(b"canonical-axis-projection")
        .expect("old schema must expose its canonical axis representation");
    assert!(
        canonical
            .child(b"roles")
            .and_then(|roles| roles.child(b"source-bound-colours"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"relations")
            .and_then(|relations| relations.child(relation))
            .is_some()
    );
    assert!(
        canonical
            .child(b"geometry")
            .and_then(|geometry| geometry.child(b"complete-axis-through-source-points"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"axes")
            .and_then(|axis_root| axis_root.child(axes))
            .is_some()
    );
    assert!(
        canonical
            .child(b"masks")
            .and_then(|masks| masks.child(b"query-derived-axis-cell-sets"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"composition")
            .and_then(|composition| { composition.child(b"input-snapshot-canonical-mask-layers") })
            .is_some()
    );
    assert!(
        canonical
            .child(b"overlays")
            .and_then(|overlay_root| overlay_root.child(overlays))
            .is_some()
    );
    assert!(
        canonical
            .child(b"overwrite-priority")
            .and_then(|priority| priority.child(overwrite))
            .is_some()
    );
    assert!(
        canonical
            .child(b"old-schema-provenance")
            .and_then(|old| old.child(provenance))
            .is_some()
    );
}

fn assert_canonical_cardinal_segment_placement(
    root: &Address,
    provenance: &[u8],
    relation: &[u8],
    composition: &[u8],
    overwrite: &[u8],
) {
    let canonical = root
        .child(b"canonical-cardinal-segment-placement")
        .expect("old schema must expose its canonical segment representation");
    assert!(
        canonical
            .child(b"roles")
            .and_then(|roles| roles.child(b"source-bound-colours"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"relations")
            .and_then(|relations| relations.child(relation))
            .is_some()
    );
    assert!(
        canonical
            .child(b"geometry")
            .and_then(|geometry| {
                geometry.child(b"inclusive-cardinal-segments-between-bound-points")
            })
            .is_some()
    );
    assert!(
        canonical
            .child(b"masks")
            .and_then(|masks| masks.child(b"query-derived-segment-cell-sets"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"composition")
            .and_then(|root| root.child(composition))
            .is_some()
    );
    assert!(
        canonical
            .child(b"overwrite-priority")
            .and_then(|priority| priority.child(overwrite))
            .is_some()
    );
    assert!(
        canonical
            .child(b"old-schema-provenance")
            .and_then(|old| old.child(provenance))
            .is_some()
    );
}

fn assert_canonical_relative_stencil_placement(
    root: &Address,
    provenance: &[u8],
    relation: &[u8],
    template: &[u8],
    placements: &[u8],
    bounds: &[u8],
    overlap: &[u8],
) {
    let canonical = root
        .child(b"canonical-relative-stencil-placement")
        .expect("old schema must expose its canonical stencil representation");
    assert!(
        canonical
            .child(b"roles")
            .and_then(|roles| roles.child(b"source-bound-role-values"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"relations")
            .and_then(|relations| relations.child(relation))
            .is_some()
    );
    assert!(
        canonical
            .child(b"geometry")
            .and_then(|geometry| {
                geometry.child(b"normalized-relative-cell-offsets-at-derived-anchors")
            })
            .is_some()
    );
    assert!(
        canonical
            .child(b"template")
            .and_then(|template_root| template_root.child(template))
            .is_some()
    );
    assert!(
        canonical
            .child(b"placements")
            .and_then(|placement_root| placement_root.child(placements))
            .is_some()
    );
    assert!(
        canonical
            .child(b"bounds")
            .and_then(|bounds_root| bounds_root.child(bounds))
            .is_some()
    );
    assert!(
        canonical
            .child(b"masks")
            .and_then(|masks| masks.child(b"query-derived-stencil-cell-sets"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"composition")
            .and_then(|composition| { composition.child(b"input-snapshot-canonical-mask-layers") })
            .is_some()
    );
    assert!(
        canonical
            .child(b"overlap-priority")
            .and_then(|priority| priority.child(overlap))
            .is_some()
    );
    assert!(
        canonical
            .child(b"old-schema-provenance")
            .and_then(|old| old.child(provenance))
            .is_some()
    );
}

fn assert_canonical_dihedral_translation_role_transform(
    root: &Address,
    provenance: &[u8],
    relation: &[u8],
    source: &[u8],
    transform: &[u8],
    composition: &[u8],
    overwrite: &[u8],
) {
    let canonical = root
        .child(b"canonical-dihedral-translation-role-transform")
        .expect("old schema must expose its canonical role transform");
    assert!(
        canonical
            .child(b"roles")
            .and_then(|roles| roles.child(b"source-bound-role-values"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"relations")
            .and_then(|relations| relations.child(relation))
            .is_some()
    );
    assert!(
        canonical
            .child(b"source-cell-set")
            .and_then(|source_root| source_root.child(source))
            .is_some()
    );
    assert!(
        canonical
            .child(b"geometry")
            .and_then(|geometry| geometry.child(b"dihedral-cell-transform-plus-translation"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"transform-selection")
            .and_then(|selection| selection.child(transform))
            .is_some()
    );
    assert!(
        canonical
            .child(b"ambiguity")
            .and_then(|ambiguity| {
                ambiguity.child(b"require-unique-admissible-transformed-cell-set")
            })
            .is_some()
    );
    assert!(
        canonical
            .child(b"masks")
            .and_then(|masks| masks.child(b"query-derived-transformed-cell-sets"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"composition")
            .and_then(|composition_root| composition_root.child(composition))
            .is_some()
    );
    assert!(
        canonical
            .child(b"overwrite-priority")
            .and_then(|priority| priority.child(overwrite))
            .is_some()
    );
    assert!(
        canonical
            .child(b"old-schema-provenance")
            .and_then(|old| old.child(provenance))
            .is_some()
    );
}

struct PeriodicLatticeCharacterization<'a> {
    provenance: &'a [u8],
    relation: &'a [u8],
    origin: &'a [u8],
    period: &'a [u8],
    bounds: &'a [u8],
    masks: &'a [u8],
    composition: &'a [u8],
    overwrite: &'a [u8],
}

fn assert_canonical_periodic_lattice_coordinate_generator(
    root: &Address,
    expected: PeriodicLatticeCharacterization<'_>,
) {
    let canonical = root
        .child(b"canonical-periodic-lattice-coordinate-generator")
        .expect("old schema must expose its canonical periodic lattice generator");
    assert!(
        canonical
            .child(b"roles")
            .and_then(|roles| roles.child(b"explicit-phase-origin-and-period-bindings"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"relations")
            .and_then(|relations| relations.child(expected.relation))
            .is_some()
    );
    assert!(
        canonical
            .child(b"geometry")
            .and_then(|geometry| {
                geometry.child(b"bounded-two-axis-congruence-cartesian-product")
            })
            .is_some()
    );
    for (field, value) in [
        (b"phase-origin".as_slice(), expected.origin),
        (b"period-binding".as_slice(), expected.period),
        (b"bounds".as_slice(), expected.bounds),
        (b"masks".as_slice(), expected.masks),
        (b"composition".as_slice(), expected.composition),
        (b"overwrite-priority".as_slice(), expected.overwrite),
    ] {
        assert!(
            canonical
                .child(field)
                .and_then(|field_root| field_root.child(value))
                .is_some()
        );
    }
    assert!(
        canonical
            .child(b"ordering")
            .and_then(|ordering| ordering.child(b"ascending-row-major"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"old-schema-provenance")
            .and_then(|old| old.child(expected.provenance))
            .is_some()
    );
}

struct BoundingEnvelopeCharacterization<'a> {
    provenance: &'a [u8],
    relation: &'a [u8],
    source_points: &'a [u8],
    derived_geometry: &'a [u8],
    masks: &'a [u8],
    composition: &'a [u8],
    overwrite: &'a [u8],
}

fn assert_canonical_bounding_envelope_corner_geometry(
    root: &Address,
    expected: BoundingEnvelopeCharacterization<'_>,
) {
    let canonical = root
        .child(b"canonical-bounding-envelope-corner-geometry")
        .expect("old schema must expose its canonical bounding envelope geometry");
    assert!(
        canonical
            .child(b"roles")
            .and_then(|roles| roles.child(b"source-bound-point-selection"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"geometry")
            .and_then(|geometry| {
                geometry.child(b"inclusive-axis-aligned-minimum-maximum-envelope")
            })
            .is_some()
    );
    assert!(
        canonical
            .child(b"corner-order")
            .and_then(|order| order.child(b"northwest-northeast-southwest-southeast"))
            .is_some()
    );
    assert!(
        canonical
            .child(b"bounds")
            .and_then(|bounds| bounds.child(b"query-grid-bounded-source-coordinates"))
            .is_some()
    );
    for (field, value) in [
        (b"relations".as_slice(), expected.relation),
        (b"source-points".as_slice(), expected.source_points),
        (b"derived-geometry".as_slice(), expected.derived_geometry),
        (b"masks".as_slice(), expected.masks),
        (b"composition".as_slice(), expected.composition),
        (b"overwrite-priority".as_slice(), expected.overwrite),
        (b"old-schema-provenance".as_slice(), expected.provenance),
    ] {
        assert!(
            canonical
                .child(field)
                .and_then(|field_root| field_root.child(value))
                .is_some()
        );
    }
}

fn progression(size: usize, start: usize, step: usize) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); size]; size];
    for i in 0..3 {
        input[start + i * step][start + i * step] = Some(1);
    }
    let mut output = input.clone();
    let mut at = start + 3 * step;
    while at < size {
        output[at][at] = Some(2);
        at += step;
    }
    (input, output)
}

#[test]
fn complete_difference_evidence_induces_arithmetic_progression_continuation() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let mut lessons = Vec::new();
    for (index, (size, start, step)) in [(15, 0, 4), (15, 1, 2)].into_iter().enumerate() {
        let (input, output) = progression(size, start, step);
        let input = admit(&grids, &format!("pi{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("po{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"progression-construction").is_some());
    let (query, expected) = progression(15, 2, 3);
    let query = admit(&grids, "pq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn connector(points: [(usize, usize, u8); 3], h: usize, w: usize) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    for (y, x, value) in points {
        input[y][x] = Some(value);
    }
    let mut output = input.clone();
    let by_color = |color| {
        let (y, x, _) = points.into_iter().find(|p| p.2 == color).unwrap();
        (y, x)
    };
    let anchor = by_color(4);
    for (color, horizontal_first) in [(2, false), (3, true)] {
        let stop = by_color(color);
        let corner = if horizontal_first {
            (anchor.0, stop.1)
        } else {
            (stop.0, anchor.1)
        };
        for (from, to) in [(anchor, corner), (corner, stop)] {
            let (mut y, mut x) = from;
            loop {
                if output[y][x] == Some(0) {
                    output[y][x] = Some(5);
                }
                if (y, x) == to {
                    break;
                }
                y = (y as isize + (to.0 as isize - y as isize).signum()) as usize;
                x = (x as isize + (to.1 as isize - x as isize).signum()) as usize;
            }
        }
    }
    for (y, x, value) in points {
        output[y][x] = Some(value);
    }
    (input, output)
}

#[test]
fn three_role_masks_induce_bound_orthogonal_connector() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        [(10, 5, 2), (4, 11, 3), (1, 1, 4)],
        [(8, 11, 2), (1, 5, 3), (10, 2, 4)],
    ];
    let mut lessons = Vec::new();
    for (index, points) in fixtures.into_iter().enumerate() {
        let (input, output) = connector(points, 13, 13);
        let input = admit(&grids, &format!("ci{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("co{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"orthogonal-connector-construction")
            .is_some()
    );
    assert_canonical_cardinal_segment_placement(
        &program.root,
        b"orthogonal-connector-construction",
        b"anchor-to-two-bound-role-points-via-bound-bend-order",
        b"input-snapshot-canonical-mask-layers",
        b"first-background-only-segment-layer-wins",
    );

    let (mut malformed, _) = connector([(11, 9, 2), (3, 2, 3), (6, 12, 4)], 14, 15);
    malformed[1][13] = Some(9);
    let malformed = admit(&grids, "cx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = connector([(11, 9, 2), (3, 2, 3), (6, 12, 4)], 14, 15);
    let query = admit(&grids, "cq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn lattice_separators(
    h: usize,
    w: usize,
    occupied_rows: &[usize],
    occupied_columns: &[usize],
) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    for &y in occupied_rows {
        for &x in occupied_columns {
            input[y][x] = Some(2);
        }
    }
    let (y0, y1) = (
        *occupied_rows.iter().min().unwrap(),
        *occupied_rows.iter().max().unwrap(),
    );
    let (x0, x1) = (
        *occupied_columns.iter().min().unwrap(),
        *occupied_columns.iter().max().unwrap(),
    );
    let mut output = input.clone();
    for (y, row) in output.iter_mut().enumerate() {
        for (x, value) in row.iter_mut().enumerate() {
            if (y0..=y1).contains(&y) && !occupied_rows.contains(&y) {
                *value = Some(if (x0..=x1).contains(&x) { 5 } else { 8 });
            } else if (x0..=x1).contains(&x) && !occupied_columns.contains(&x) {
                *value = Some(if (y0..=y1).contains(&y) { 5 } else { 8 });
            }
        }
    }
    (input, output)
}

#[test]
fn occupied_point_bounds_induce_canonical_lattice_separator_envelope() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        lattice_separators(9, 10, &[2, 4, 6], &[2, 5, 7]),
        lattice_separators(10, 10, &[1, 4, 8], &[1, 5, 8]),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("lsi{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("lso{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"lattice-separator-construction")
            .is_some()
    );
    assert_canonical_bounding_envelope_corner_geometry(
        &program.root,
        BoundingEnvelopeCharacterization {
            provenance: b"lattice-separator-construction",
            relation: b"complete-empty-envelope-axes-bind-separator-lines",
            source_points: b"all-query-nonbackground-cells",
            derived_geometry: b"inclusive-envelope-rows-columns-and-axis-gaps",
            masks: b"complete-gap-axis-lines-extended-to-grid-bounds",
            composition: b"input-snapshot-gap-rows-before-gap-columns",
            overwrite: b"separator-lines-replace-input-values",
        },
    );

    let (malformed, _) = lattice_separators(8, 8, &[2, 3, 4], &[2, 3, 4]);
    let malformed = admit(&grids, "lsx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = lattice_separators(12, 13, &[2, 6, 10], &[1, 5, 11]);
    let query = admit(&grids, "lsq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn missing_corner_paths(
    h: usize,
    w: usize,
    bounds: (usize, usize, usize, usize),
    missing_index: usize,
) -> (Pixels, Pixels) {
    let (y0, y1, x0, x1) = bounds;
    let corners = [(y0, x0), (y0, x1), (y1, x0), (y1, x1)];
    let present = corners
        .into_iter()
        .enumerate()
        .filter_map(|(index, point)| (index != missing_index).then_some(point))
        .collect::<Vec<_>>();
    let missing = corners[missing_index];
    let mut input = vec![vec![Some(0); w]; h];
    for &(y, x) in &present {
        input[y][x] = Some(2);
    }
    let mut output = input.clone();
    output[missing.0][x0..=x1].fill(Some(7));
    for row in output.iter_mut().take(y1 + 1).skip(y0) {
        row[missing.1] = Some(7);
    }
    let diagonal = present
        .iter()
        .enumerate()
        .flat_map(|(index, a)| present.iter().skip(index + 1).map(move |b| (*a, *b)))
        .find(|(a, b)| a.0 != b.0 && a.1 != b.1)
        .unwrap();
    let steps = diagonal
        .0
        .0
        .abs_diff(diagonal.1.0)
        .max(diagonal.0.1.abs_diff(diagonal.1.1));
    for step in 0..=steps {
        let y = (diagonal.0.0 as isize
            + (diagonal.1.0 as isize - diagonal.0.0 as isize) * step as isize / steps as isize)
            as usize;
        let x = (diagonal.0.1 as isize
            + (diagonal.1.1 as isize - diagonal.0.1 as isize) * step as isize / steps as isize)
            as usize;
        output[y][x] = Some(7);
    }
    for &(y, x) in &present {
        output[y][x] = Some(2);
    }
    (input, output)
}

#[test]
fn three_marker_bounds_induce_canonical_missing_corner_geometry() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        missing_corner_paths(10, 12, (2, 7, 2, 9), 1),
        missing_corner_paths(12, 13, (1, 9, 3, 10), 2),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("mci{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("mco{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"missing-corner-path-construction")
            .is_some()
    );
    assert_canonical_bounding_envelope_corner_geometry(
        &program.root,
        BoundingEnvelopeCharacterization {
            provenance: b"missing-corner-path-construction",
            relation: b"three-occupied-envelope-corners-bind-missing-corner",
            source_points: b"query-cells-equal-bound-marker-role",
            derived_geometry: b"ordered-envelope-corners-missing-corner-and-opposite-diagonal",
            masks: b"missing-corner-axes-and-opposite-corner-diagonal",
            composition: b"row-then-column-then-diagonal-then-marker-restore",
            overwrite: b"later-path-layers-and-final-markers-win",
        },
    );

    let (mut malformed, _) = missing_corner_paths(10, 11, (2, 8, 1, 9), 0);
    malformed[2][1] = Some(2);
    let malformed = admit(&grids, "mcx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = missing_corner_paths(13, 15, (3, 11, 2, 12), 3);
    let query = admit(&grids, "mcq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn adaptive_inset(h: usize, w: usize, points: [(usize, usize, u8); 3]) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    for &(y, x, value) in &points {
        input[y][x] = Some(value);
    }
    let y0 = points.iter().map(|point| point.0).min().unwrap();
    let y1 = points.iter().map(|point| point.0).max().unwrap();
    let x0 = points.iter().map(|point| point.1).min().unwrap();
    let x1 = points.iter().map(|point| point.1).max().unwrap();
    assert_eq!((y0 + y1) % 2, 0);
    assert_eq!((x0 + x1) % 2, 0);
    let (cy, cx) = ((y0 + y1) / 2, (x0 + x1) / 2);
    let distance = (y1.abs_diff(y0).max(x1.abs_diff(x0)) + 1) / 3;
    let mut output = input.clone();
    let mut ordered_points = points;
    ordered_points.sort_by_key(|point| (point.0, point.1));
    for (y, x, value) in ordered_points {
        let ny = (y as isize + (cy as isize - y as isize).signum() * distance as isize) as usize;
        let nx = (x as isize + (cx as isize - x as isize).signum() * distance as isize) as usize;
        output[ny][nx] = Some(value);
    }
    output[cy][cx] = Some(9);
    (input, output)
}

#[test]
fn three_role_bounds_induce_canonical_adaptive_inset_geometry() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        adaptive_inset(11, 11, [(1, 1, 2), (1, 9, 3), (9, 5, 4)]),
        adaptive_inset(12, 13, [(2, 3, 5), (8, 5, 6), (4, 9, 7)]),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("aii{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("aio{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"adaptive-inset-construction").is_some());
    assert_canonical_bounding_envelope_corner_geometry(
        &program.root,
        BoundingEnvelopeCharacterization {
            provenance: b"adaptive-inset-construction",
            relation: b"envelope-center-and-maximum-span-bind-role-insets",
            source_points: b"exactly-three-distinct-query-nonbackground-roles",
            derived_geometry: b"integral-envelope-center-and-one-third-maximum-span",
            masks: b"inset-role-destinations-and-center-cell",
            composition: b"input-snapshot-row-major-role-insets-then-center",
            overwrite: b"later-role-insets-then-center-target-win",
        },
    );

    let mut malformed = vec![vec![Some(0); 11]; 10];
    for (y, x, value) in [(1, 1, 2), (2, 9, 3), (8, 5, 4)] {
        malformed[y][x] = Some(value);
    }
    let malformed = admit(&grids, "aix", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = adaptive_inset(15, 16, [(2, 2, 3), (2, 12, 6), (12, 7, 8)]);
    let query = admit(&grids, "aiq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn marker_crosses(h: usize, w: usize, points: &[(usize, usize)]) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    for &(y, x) in points {
        input[y][x] = Some(2);
    }
    let mut output = input.clone();
    for &(y, x) in points {
        for cell in &mut output[y] {
            *cell = Some(2);
        }
        for row in &mut output {
            row[x] = Some(2);
        }
    }
    for &(y, x) in points {
        output[y][x] = Some(1);
        for (dy, dx) in [(-1isize, -1isize), (-1, 1), (1, -1), (1, 1)] {
            let (ay, ax) = (y as isize + dy, x as isize + dx);
            if ay >= 0 && ax >= 0 && ay < h as isize && ax < w as isize {
                output[ay as usize][ax as usize] = Some(3);
            }
        }
    }
    (input, output)
}

#[test]
fn homogeneous_markers_induce_canonical_orthogonal_axis_pairs() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures: [&[(usize, usize)]; 2] = [&[(2, 2), (6, 7)], &[(1, 8), (7, 3)]];
    let mut lessons = Vec::new();
    for (index, points) in fixtures.into_iter().enumerate() {
        let (input, output) = marker_crosses(10, 11, points);
        let input = admit(&grids, &format!("mi{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("mo{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let candidates = machine.agree_generalized_candidates(&lessons).unwrap();
    assert_eq!(candidates.candidates().len(), 2);
    let program = candidates.primary().unwrap().program();
    let alternative = candidates.secondary().unwrap();
    assert_eq!(alternative.origin(), GeneralizedCandidateOrigin::Broad);
    assert!(program.root.child(b"marker-cross-construction").is_some());
    assert!(
        alternative
            .program()
            .root
            .child(b"complete-diagonal-neighborhoods")
            .is_some()
    );
    assert_canonical_axis_projection(
        &program.root,
        b"marker-cross-construction",
        b"homogeneous-marker-sources",
        b"horizontal-and-vertical",
        b"center-and-diagonal-neighbourhood",
        b"axis-layers-then-source-neighbourhood-wins",
    );

    let (mut malformed, _) = marker_crosses(9, 12, &[(2, 3), (6, 9)]);
    malformed[4][5] = Some(4);
    let malformed = admit(&grids, "mx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, program)
            .is_err()
    );

    let (query, expected) = marker_crosses(9, 12, &[(2, 3), (6, 9)]);
    let query = admit(&grids, "mq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);

    let (edge_query, edge_expected) = marker_crosses(14, 14, &[(2, 7), (13, 13)]);
    let edge_query = admit(&grids, "mq-edge", &edge_query, Side::QueryInput);
    let primary_output = machine
        .apply_groups(&mut grids, &edge_query, program)
        .unwrap();
    assert_eq!(primary_output.inspect().unwrap(), edge_expected);
    let secondary_output = machine
        .apply_groups(&mut grids, &edge_query, alternative.program())
        .unwrap();
    assert_ne!(
        primary_output.inspect().unwrap(),
        secondary_output.inspect().unwrap()
    );
}

fn beams(h: usize, w: usize, points: &[(usize, usize, u8)]) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    for &(y, x, value) in points {
        input[y][x] = Some(value);
    }
    let mut output = input.clone();
    let original = points.iter().map(|&(y, x, _)| (y, x)).collect::<Vec<_>>();
    for wanted in [2, 1] {
        for &(y, x, value) in points {
            let vertical = value == 2;
            if (wanted == 2) != vertical {
                continue;
            }
            if vertical {
                for (ay, row) in output.iter_mut().enumerate() {
                    if !original.contains(&(ay, x)) {
                        row[x] = Some(value);
                    }
                }
            } else {
                for (ax, cell) in output[y].iter_mut().enumerate() {
                    if !original.contains(&(y, ax)) {
                        *cell = Some(value);
                    }
                }
            }
        }
    }
    for &(y, x, value) in points {
        output[y][x] = Some(value);
    }
    (input, output)
}

#[test]
fn complete_role_evidence_induces_ordered_axis_beams() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures: [&[(usize, usize, u8)]; 2] = [
        &[(1, 1, 3), (4, 3, 3), (6, 1, 1), (7, 5, 2)],
        &[(2, 2, 2), (4, 7, 3), (6, 3, 1)],
    ];
    let mut lessons = Vec::new();
    for (index, points) in fixtures.into_iter().enumerate() {
        let (input, output) = beams(10, 9, points);
        let input = admit(&grids, &format!("bi{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("bo{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"axis-beam-construction").is_some());
    assert_canonical_mask_composition(
        &program.root,
        b"axis-beam-construction",
        b"role-to-axis-bindings",
        b"complete-axis-through-source-points",
        b"later-axis-layer-then-source-mask-wins",
    );
    assert_canonical_axis_projection(
        &program.root,
        b"axis-beam-construction",
        b"role-to-axis-bindings",
        b"role-selected-horizontal-or-vertical",
        b"source-point-restoration",
        b"later-axis-layer-then-source-mask-wins",
    );
    let (mut malformed, _) = beams(12, 11, &[(1, 2, 1), (5, 8, 3), (9, 4, 2)]);
    malformed[3][6] = Some(9);
    let malformed = admit(&grids, "bx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = beams(12, 11, &[(1, 2, 1), (5, 8, 3), (9, 4, 2)]);
    let query = admit(&grids, "bq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn move_to_touch(
    h: usize,
    w: usize,
    origin: (usize, usize),
    anchor: (usize, usize),
) -> (Pixels, Pixels) {
    let shape = [(0usize, 0usize), (0, 1), (1, 0), (1, 1), (2, 0)];
    let mut input = vec![vec![Some(0); w]; h];
    for (dy, dx) in shape {
        input[origin.0 + dy][origin.1 + dx] = Some(2);
    }
    input[anchor.0][anchor.1] = Some(8);
    let mut output = input.clone();
    for (dy, dx) in shape {
        output[origin.0 + dy][origin.1 + dx] = Some(0);
    }
    let shift = anchor.1 - (origin.1 + 1) - 1;
    for (dy, dx) in shape {
        output[origin.0 + dy][origin.1 + dx + shift] = Some(2);
    }
    (input, output)
}

#[test]
fn component_delta_induces_cardinal_move_until_touch() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [(10, 11, (1, 1), (2, 8)), (12, 14, (4, 2), (5, 11))];
    let mut lessons = Vec::new();
    for (index, (h, w, origin, anchor)) in fixtures.into_iter().enumerate() {
        let (input, output) = move_to_touch(h, w, origin, anchor);
        let input = admit(&grids, &format!("mi{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("mo{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"move-to-touch-construction").is_some());
    let (query, expected) = move_to_touch(11, 13, (3, 1), (4, 10));
    let query = admit(&grids, "mq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn picture(rows: &[&str]) -> Pixels {
    rows.iter()
        .map(|row| {
            row.bytes()
                .map(|value| Some(if value == b'.' { 0 } else { value - b'0' }))
                .collect()
        })
        .collect()
}

fn periodic_diamonds(h: usize, w: usize, markers: &[(usize, usize, u8)]) -> (Pixels, Pixels) {
    const PERIOD: usize = 3;
    let mut input = vec![vec![Some(0); w]; h];
    let mut output = input.clone();
    for &(y, x, value) in markers {
        input[y][x] = Some(value);
        output[y][x] = Some(0);
        for dy in 1..PERIOD {
            for dx in 1..PERIOD {
                if y + dy < h && x + dx < w {
                    output[y + dy][x + dx] = Some(value);
                }
            }
        }
    }
    (input, output)
}

#[test]
fn grid_origin_markers_induce_canonical_periodic_diamond_lattice() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        periodic_diamonds(7, 8, &[(0, 0, 2), (6, 6, 3)]),
        periodic_diamonds(9, 10, &[(3, 3, 4), (6, 9, 7)]),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("pdi{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("pdo{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"periodic-diamond-construction")
            .is_some()
    );
    assert_canonical_periodic_lattice_coordinate_generator(
        &program.root,
        PeriodicLatticeCharacterization {
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

    let mut malformed = vec![vec![Some(0); 8]; 8];
    malformed[1][1] = Some(6);
    let malformed = admit(&grids, "pdx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (mut query, expected) = periodic_diamonds(8, 7, &[(0, 3, 5), (3, 6, 8)]);
    query[1][4] = Some(9);
    let query = admit(&grids, "pdq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

#[test]
fn sparse_lattice_role_is_selected_by_frequency_and_noncentral_position() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        (
            picture(&[
                "..........",
                "..........",
                "..2.3.....",
                "...4.4....",
                "..3.4.3...",
                "...4.4....",
                "....3.....",
                "..........",
                "..........",
                "..........",
            ]),
            picture(&[
                "..........",
                "..........",
                "..2.3.2...",
                "...4.4....",
                "..3.4.3...",
                "...4.4....",
                "..2.3.2...",
                "..........",
                "..........",
                "..........",
            ]),
        ),
        (
            picture(&[
                "..........",
                "...8.8.8..",
                "....4.....",
                "...8.1.8..",
                "..........",
                "...8.8.8..",
                "..........",
                "..........",
                "..........",
                "..........",
            ]),
            picture(&[
                "..........",
                "...8.8.8..",
                "....4.4...",
                "...8.1.8..",
                "....4.4...",
                "...8.8.8..",
                "..........",
                "..........",
                "..........",
                "..........",
            ]),
        ),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("li{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("lo{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"sparse-lattice-construction").is_some());
    assert_canonical_periodic_lattice_coordinate_generator(
        &program.root,
        PeriodicLatticeCharacterization {
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
    assert_canonical_bounding_envelope_corner_geometry(
        &program.root,
        BoundingEnvelopeCharacterization {
            provenance: b"sparse-lattice-construction",
            relation: b"nonbackground-envelope-binds-lattice-bounds-and-center-test",
            source_points: b"all-query-nonbackground-cells",
            derived_geometry: b"inclusive-envelope-and-doubled-center-coordinate",
            masks: b"envelope-bounded-periodic-candidate-cells",
            composition: b"input-snapshot-background-only-additions",
            overwrite: b"preserve-all-nonbackground-input-cells",
        },
    );
    let query = picture(&[
        "..........",
        "..........",
        ".1.4.1....",
        "..2.2.....",
        "...1......",
        "..2.2.....",
        ".1...1....",
        "..........",
        "..........",
        "..........",
    ]);
    let expected = picture(&[
        "..........",
        "..........",
        ".1.4.1....",
        "..2.2.....",
        ".4.1.4....",
        "..2.2.....",
        ".1.4.1....",
        "..........",
        "..........",
        "..........",
    ]);
    let query = admit(&grids, "lq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn stripes(
    h: usize,
    w: usize,
    first: (usize, usize, u8),
    second: (usize, usize, u8),
) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    input[first.0][first.1] = Some(first.2);
    input[second.0][second.1] = Some(second.2);
    let row_axis = first.0.abs_diff(second.0) != 0
        && (first.1.abs_diff(second.1) == 0
            || first.0.abs_diff(second.0) < first.1.abs_diff(second.1));
    let mut roles = [first, second];
    roles.sort_by_key(|point| if row_axis { point.0 } else { point.1 });
    let step = if row_axis {
        roles[1].0 - roles[0].0
    } else {
        roles[1].1 - roles[0].1
    };
    let mut output = input.clone();
    let mut at = if row_axis { roles[0].0 } else { roles[0].1 };
    let mut index = 0;
    while at < if row_axis { h } else { w } {
        if row_axis {
            output[at].fill(Some(roles[index % 2].2));
        } else {
            for row in &mut output {
                row[at] = Some(roles[index % 2].2);
            }
        }
        at += step;
        index += 1;
    }
    (input, output)
}

#[test]
fn two_marker_evidence_induces_axis_selected_alternating_stripes() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        stripes(22, 9, (5, 0, 2), (7, 8, 3)),
        stripes(7, 23, (0, 5, 1), (6, 8, 3)),
        stripes(24, 8, (7, 0, 4), (11, 0, 1)),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("si{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("so{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"alternating-stripe-construction")
            .is_some()
    );
    let (query, expected) = stripes(11, 27, (0, 4, 8), (10, 7, 2));
    let query = admit(&grids, "sq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn boundary_rays(side: u8) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); 7]; 7];
    let mut output = input.clone();
    match side {
        b'r' => {
            for (input_row, output_row) in input.iter_mut().zip(&mut output) {
                input_row[6] = Some(5);
                output_row[6] = Some(5);
            }
            for y in 1..=2 {
                input[y][3] = Some(2);
                output[y][3..6].fill(Some(2));
            }
            for y in 4..=5 {
                input[y][1] = Some(3);
                output[y][1..6].fill(Some(3));
            }
        }
        b'b' => {
            input[6].fill(Some(5));
            output[6].fill(Some(5));
            for x in 1..=2 {
                input[2][x] = Some(2);
                for row in output.iter_mut().take(6).skip(2) {
                    row[x] = Some(2);
                }
            }
            for x in 4..=5 {
                input[4][x] = Some(3);
                for row in output.iter_mut().take(6).skip(4) {
                    row[x] = Some(3);
                }
            }
        }
        _ => {
            for (input_row, output_row) in input.iter_mut().zip(&mut output) {
                input_row[0] = Some(5);
                output_row[0] = Some(5);
            }
            for y in 1..=2 {
                input[y][4] = Some(2);
                output[y][1..=4].fill(Some(2));
            }
            for y in 4..=5 {
                input[y][6] = Some(3);
                output[y][1..=6].fill(Some(3));
            }
        }
    }
    (input, output)
}

#[test]
fn complete_boundary_role_induces_ordered_component_rays() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let mut lessons = Vec::new();
    for (index, side) in [b'r', b'b'].into_iter().enumerate() {
        let (input, output) = boundary_rays(side);
        let input = admit(&grids, &format!("ri{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("ro{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"boundary-ray-construction").is_some());
    let (query, expected) = boundary_rays(b'l');
    let query = admit(&grids, "rq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn template_rays(
    h: usize,
    w: usize,
    template_anchor: (usize, usize),
    direction: (isize, isize),
    template: u8,
    marker: u8,
) -> (Pixels, Pixels) {
    let shape = [(0isize, 0isize), (0, 1), (1, 0)];
    let mut input = vec![vec![Some(0); w]; h];
    for (dy, dx) in shape {
        input[(template_anchor.0 as isize + dy) as usize]
            [(template_anchor.1 as isize + dx) as usize] = Some(template);
    }
    let first_anchor = (
        template_anchor.0 as isize + direction.0 * 3,
        template_anchor.1 as isize + direction.1 * 3,
    );
    input[first_anchor.0 as usize][(first_anchor.1 + 1) as usize] = Some(marker);

    let mut output = input.clone();
    let mut step = 1isize;
    loop {
        let anchor = (
            template_anchor.0 as isize + direction.0 * 3 * step,
            template_anchor.1 as isize + direction.1 * 3 * step,
        );
        let mut visible = false;
        for (dy, dx) in shape {
            let (y, x) = (anchor.0 + dy, anchor.1 + dx);
            if y >= 0 && x >= 0 && y < h as isize && x < w as isize {
                output[y as usize][x as usize] = Some(marker);
                visible = true;
            }
        }
        if !visible {
            break;
        }
        step += 1;
    }
    (input, output)
}

#[test]
fn partial_components_induce_canonical_repeated_template_stencils() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        template_rays(10, 11, (5, 4), (-1, 0), 2, 7),
        template_rays(12, 13, (3, 3), (1, 1), 4, 8),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("tri{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("tro{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"template-ray-construction").is_some());
    assert_canonical_relative_stencil_placement(
        &program.root,
        b"template-ray-construction",
        b"partial-role-components-to-lattice-ray-anchors",
        b"largest-component-normalized-shape",
        b"repeated-lattice-ray-anchors",
        b"clip-border-visible-cells",
        b"later-ordered-stamp-wins",
    );

    let (mut malformed, _) = template_rays(11, 12, (5, 6), (-1, 0), 3, 9);
    malformed[2][7] = Some(3);
    let malformed = admit(&grids, "trx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = template_rays(13, 14, (6, 7), (0, -1), 3, 9);
    let query = admit(&grids, "trq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn fixture_dihedral(index: usize, y: isize, x: isize) -> (isize, isize) {
    match index {
        0 => (y, x),
        1 => (y, -x),
        2 => (-y, x),
        3 => (-y, -x),
        4 => (x, y),
        5 => (x, -y),
        6 => (-x, y),
        _ => (-x, -y),
    }
}

fn relocated_diagram(
    h: usize,
    w: usize,
    source_anchor: (usize, usize),
    target_first: (usize, usize),
    transform: usize,
) -> (Pixels, Pixels) {
    let shape = [
        (0isize, 0isize, 2u8),
        (0, 1, 4),
        (1, 1, 4),
        (1, 2, 4),
        (2, 1, 3),
    ];
    let source = shape
        .iter()
        .map(|&(dy, dx, value)| {
            (
                source_anchor.0 as isize + dy,
                source_anchor.1 as isize + dx,
                value,
            )
        })
        .collect::<Vec<_>>();
    let first = fixture_dihedral(transform, source[0].0, source[0].1);
    let translation = (
        target_first.0 as isize - first.0,
        target_first.1 as isize - first.1,
    );
    let destination = source
        .iter()
        .map(|&(y, x, value)| {
            let transformed = fixture_dihedral(transform, y, x);
            (
                (transformed.0 + translation.0) as usize,
                (transformed.1 + translation.1) as usize,
                value,
            )
        })
        .collect::<Vec<_>>();
    let mut input = vec![vec![Some(0); w]; h];
    for &(y, x, value) in &source {
        input[y as usize][x as usize] = Some(value);
    }
    for &(y, x, value) in &destination {
        if value == 2 || value == 3 {
            input[y][x] = Some(value);
        }
    }
    let mut output = input.clone();
    for &(y, x, _) in &source {
        output[y as usize][x as usize] = Some(0);
    }
    for (y, x, value) in destination {
        output[y][x] = Some(value);
    }
    (input, output)
}

fn overlapping_relocated_diagrams() -> (Pixels, Pixels) {
    let first = [
        (1usize, 1usize, 2u8),
        (1, 2, 4),
        (2, 2, 4),
        (2, 3, 4),
        (3, 2, 3),
    ];
    let second = [
        (4usize, 10usize, 5u8),
        (4, 11, 7),
        (5, 11, 7),
        (6, 11, 7),
        (6, 12, 7),
        (7, 12, 8),
    ];
    let first_destination = [
        (8usize, 4usize, 2u8),
        (8, 5, 4),
        (9, 5, 4),
        (9, 6, 4),
        (10, 5, 3),
    ];
    let second_destination = [
        (7usize, 4usize, 5u8),
        (7, 5, 7),
        (8, 5, 7),
        (9, 5, 7),
        (9, 6, 7),
        (10, 6, 8),
    ];
    let mut input = vec![vec![Some(0); 16]; 14];
    for &(y, x, value) in first.iter().chain(&second) {
        input[y][x] = Some(value);
    }
    for &(y, x, value) in first_destination.iter().chain(&second_destination) {
        if [2, 3, 5, 8].contains(&value) {
            input[y][x] = Some(value);
        }
    }
    let mut output = input.clone();
    for &(y, x, _) in first.iter().chain(&second) {
        output[y][x] = Some(0);
    }
    for &(y, x, value) in first_destination.iter().chain(&second_destination) {
        output[y][x] = Some(value);
    }
    (input, output)
}

#[test]
fn unique_role_correspondences_induce_canonical_dihedral_relocation() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        relocated_diagram(12, 14, (2, 2), (3, 10), 0),
        relocated_diagram(13, 15, (3, 3), (9, 10), 5),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("rdi{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("rdo{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"relocate-diagram-construction")
            .is_some()
    );
    assert_canonical_dihedral_translation_role_transform(
        &program.root,
        b"relocate-diagram-construction",
        b"unique-role-correspondences-bind-destination-translation",
        b"complete-mixed-role-component",
        b"unique-admissible-dihedral-and-translation",
        b"clear-all-source-components-then-ordered-destinations",
        b"ordered-destination-wins-after-input-snapshot-compatibility",
    );

    let (mut malformed, _) = relocated_diagram(14, 16, (2, 2), (3, 11), 0);
    malformed[9][2] = Some(2);
    malformed[11][3] = Some(3);
    let malformed = admit(&grids, "rdx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = overlapping_relocated_diagrams();
    let query = admit(&grids, "rdq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn framed_motif(
    h: usize,
    w: usize,
    frame_origin: (usize, usize),
    frame_size: usize,
    reflect_top_to_bottom: bool,
) -> (Pixels, Pixels) {
    let (y0, x0) = frame_origin;
    let (y1, x1) = (y0 + frame_size - 1, x0 + frame_size - 1);
    let mut input = vec![vec![Some(0); w]; h];
    input[y0][x0..=x1].fill(Some(6));
    input[y1][x0..=x1].fill(Some(6));
    for row in input.iter_mut().take(y1 + 1).skip(y0) {
        row[x0] = Some(6);
        row[x1] = Some(6);
    }
    let center = frame_size / 2;
    let motif = if reflect_top_to_bottom {
        (center - 1..=center + 1)
            .map(|dx| (y0 + 2, x0 + dx))
            .collect::<Vec<_>>()
    } else {
        (center - 1..=center + 1)
            .map(|dy| (y0 + dy, x0 + 2))
            .collect::<Vec<_>>()
    };
    for &(y, x) in &motif {
        input[y][x] = Some(2);
    }
    let mut output = input.clone();
    for (y, x) in motif {
        let target = if reflect_top_to_bottom {
            (y0 + y1 - y, x)
        } else {
            (y, x0 + x1 - x)
        };
        output[target.0][target.1] = Some(2);
    }
    (input, output)
}

#[test]
fn frame_bounds_induce_canonical_internal_motif_reflection() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        framed_motif(12, 13, (2, 2), 7, false),
        framed_motif(14, 15, (3, 4), 7, true),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("fmi{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("fmo{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"reflect-internal-motif-construction")
            .is_some()
    );
    assert_canonical_dihedral_translation_role_transform(
        &program.root,
        b"reflect-internal-motif-construction",
        b"frame-bounds-bind-reflection-translation",
        b"frame-internal-motif-cell-set",
        b"unique-admissible-axis-reflection",
        b"input-snapshot-transformed-additions-only",
        b"motif-additions-replace-admissible-background",
    );

    let (mut malformed, _) = framed_motif(13, 14, (2, 3), 7, false);
    malformed[4][5] = Some(0);
    let malformed = admit(&grids, "fmx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = framed_motif(16, 18, (4, 5), 9, false);
    let query = admit(&grids, "fmq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn cross_stencils(
    h: usize,
    w: usize,
    centers: &[(usize, usize)],
    arm: u8,
    center: u8,
) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    let mut output = input.clone();
    for &(cy, cx) in centers {
        input[cy][cx] = Some(center);
        for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
            input[(cy as isize + dy) as usize][(cx as isize + dx) as usize] = Some(arm);
        }
        for (dy, dx) in [(-2isize, 0isize), (0, -2), (0, 2), (2, 0)] {
            output[(cy as isize + dy) as usize][(cx as isize + dx) as usize] = Some(arm);
        }
        for (dy, dx) in [
            (-2isize, -2isize),
            (-2, 2),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
            (2, -2),
            (2, 2),
        ] {
            output[(cy as isize + dy) as usize][(cx as isize + dx) as usize] = Some(center);
        }
    }
    for &(cy, cx) in centers {
        output[cy][cx] = Some(center);
        for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
            output[(cy as isize + dy) as usize][(cx as isize + dx) as usize] = Some(arm);
        }
    }
    (input, output)
}

#[test]
fn normalized_component_roles_induce_palette_bound_stencil() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        cross_stencils(12, 12, &[(3, 3), (9, 8)], 8, 6),
        cross_stencils(14, 14, &[(4, 4), (9, 10)], 7, 2),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("ti{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("to{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let candidates = machine.agree_generalized_candidates(&lessons).unwrap();
    let program = candidates.primary().unwrap().program();
    let alternative = candidates
        .candidates()
        .iter()
        .find(|candidate| {
            candidate
                .program()
                .root
                .child(b"clip-at-grid-boundary")
                .is_some()
        })
        .expect("complete demonstrations retain the canonical clipped-bounds alternative");
    assert!(
        program
            .root
            .child(b"component-stencil-construction")
            .is_some()
    );
    assert_canonical_relative_stencil_placement(
        &program.root,
        b"component-stencil-construction",
        b"equal-normalized-mixed-components",
        b"component-relative-role-effect-stencil",
        b"normalized-component-top-left-anchors",
        b"reject-out-of-bounds-placement",
        b"reject-conflicting-overlap",
    );

    let mut malformed = vec![vec![Some(0); 8]; 8];
    malformed[1][1] = Some(9);
    for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
        malformed[(1isize + dy) as usize][(1isize + dx) as usize] = Some(3);
    }
    let malformed = admit(&grids, "tx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, program)
            .is_err()
    );
    let clipped = machine
        .apply_groups(&mut grids, &malformed, alternative.program())
        .expect("the independently replayed bounds policy clips only out-of-grid effect cells")
        .inspect()
        .unwrap();
    assert_eq!((clipped.len(), clipped[0].len()), (8, 8));
    assert_eq!(clipped[1][1], Some(9));
    assert_eq!(clipped[0][0], Some(9));
    assert_eq!(clipped[3][3], Some(9));

    let (query, expected) = cross_stencils(15, 15, &[(5, 4), (10, 11)], 3, 9);
    let query = admit(&grids, "tq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

#[test]
fn whole_grid_scale_is_replay_valid_and_changes_output_extent() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let scale = |input: &Pixels| -> Pixels {
        input
            .iter()
            .flat_map(|row| {
                let expanded = row
                    .iter()
                    .flat_map(|value| [*value, *value])
                    .collect::<Vec<_>>();
                [expanded.clone(), expanded]
            })
            .collect()
    };
    let examples = [
        vec![vec![Some(1), Some(0)], vec![Some(0), Some(2)]],
        vec![vec![Some(3), Some(4), Some(0)]],
    ];
    let lessons = examples
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let output = scale(input);
            (
                admit(
                    &grids,
                    &format!("extent-in-{index}"),
                    input,
                    Side::TrainingInput,
                ),
                admit(
                    &grids,
                    &format!("extent-out-{index}"),
                    &output,
                    Side::TrainingOutput,
                ),
            )
        })
        .collect::<Vec<_>>();
    let candidates = machine
        .agree_canonical_whole_grid_transforms(&lessons)
        .unwrap();
    let program = candidates
        .iter()
        .find(|candidate| candidate.root.child(b"scale").is_some())
        .unwrap();
    assert!(program.root.child(b"all-demonstrations-valid").is_some());
    let query_pixels = vec![
        vec![Some(5), Some(0)],
        vec![Some(6), Some(7)],
        vec![Some(0), Some(8)],
    ];
    let expected = scale(&query_pixels);
    let query = admit(&grids, "extent-query", &query_pixels, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!((actual.height(), actual.width()), (6, 4));
    assert_eq!(actual.inspect().unwrap(), expected);
}

#[test]
fn whole_grid_palette_mapping_is_replay_valid() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let examples = [
        (
            vec![vec![Some(1), Some(2)], vec![Some(0), Some(1)]],
            vec![vec![Some(5), Some(7)], vec![Some(0), Some(5)]],
        ),
        (
            vec![vec![Some(2), Some(0), Some(1)]],
            vec![vec![Some(7), Some(0), Some(5)]],
        ),
    ];
    let lessons = examples
        .iter()
        .enumerate()
        .map(|(index, (input, output))| {
            (
                admit(
                    &grids,
                    &format!("palette-in-{index}"),
                    input,
                    Side::TrainingInput,
                ),
                admit(
                    &grids,
                    &format!("palette-out-{index}"),
                    output,
                    Side::TrainingOutput,
                ),
            )
        })
        .collect::<Vec<_>>();
    let candidates = machine
        .agree_canonical_whole_grid_transforms(&lessons)
        .unwrap();
    let program = candidates
        .iter()
        .find(|candidate| candidate.root.child(b"palette-map").is_some())
        .unwrap();
    let query_pixels = vec![
        vec![Some(2), Some(1), Some(0)],
        vec![Some(1), Some(2), Some(2)],
    ];
    let expected = vec![
        vec![Some(7), Some(5), Some(0)],
        vec![Some(5), Some(7), Some(7)],
    ];
    let query = admit(&grids, "palette-query", &query_pixels, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

#[test]
fn unique_extreme_component_induces_bounded_object_extraction() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let input = vec![
        vec![Some(0), Some(2), Some(2), Some(0), Some(0)],
        vec![Some(0), Some(2), Some(0), Some(0), Some(3)],
        vec![Some(0), Some(0), Some(0), Some(0), Some(0)],
    ];
    let output = vec![vec![Some(2), Some(2)], vec![Some(2), Some(0)]];
    let lessons = vec![(
        admit(&grids, "object-in", &input, Side::TrainingInput),
        admit(&grids, "object-out", &output, Side::TrainingOutput),
    )];
    let candidates = machine
        .agree_canonical_whole_grid_transforms(&lessons)
        .unwrap();
    let program = candidates
        .iter()
        .find(|candidate| candidate.root.child(b"object-extraction").is_some())
        .unwrap();
    let query_pixels = vec![
        vec![Some(4), Some(0), Some(0), Some(7), Some(7), Some(7)],
        vec![Some(0), Some(0), Some(0), Some(7), Some(0), Some(7)],
        vec![Some(0), Some(0), Some(0), Some(7), Some(7), Some(7)],
    ];
    let expected = vec![
        vec![Some(7), Some(7), Some(7)],
        vec![Some(7), Some(0), Some(7)],
        vec![Some(7), Some(7), Some(7)],
    ];
    let query = admit(&grids, "object-query", &query_pixels, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

#[test]
fn repeated_blocks_induce_unique_mode_reduction() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let input = vec![
        vec![Some(1), Some(1), Some(2), Some(4), Some(4), Some(3)],
        vec![Some(1), Some(2), Some(1), Some(4), Some(3), Some(4)],
        vec![Some(2), Some(1), Some(2), Some(3), Some(4), Some(4)],
    ];
    let output = vec![vec![Some(1), Some(4)]];
    let lessons = vec![(
        admit(&grids, "block-in", &input, Side::TrainingInput),
        admit(&grids, "block-out", &output, Side::TrainingOutput),
    )];
    let candidates = machine
        .agree_canonical_whole_grid_transforms(&lessons)
        .unwrap();
    let program = candidates
        .iter()
        .find(|candidate| candidate.root.child(b"block-reduction").is_some())
        .unwrap();
    let query_pixels = vec![
        vec![Some(5), Some(5), Some(7), Some(6), Some(6), Some(8)],
        vec![Some(5), Some(7), Some(5), Some(6), Some(8), Some(6)],
        vec![Some(7), Some(5), Some(7), Some(8), Some(6), Some(6)],
    ];
    let expected = vec![vec![Some(5), Some(6)]];
    let query = admit(&grids, "block-query", &query_pixels, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

#[test]
fn complete_uniform_axes_induce_separator_compaction() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let input = vec![
        vec![Some(1), Some(9), Some(2)],
        vec![Some(9), Some(9), Some(9)],
        vec![Some(3), Some(9), Some(4)],
    ];
    let output = vec![vec![Some(1), Some(2)], vec![Some(3), Some(4)]];
    let lessons = vec![(
        admit(&grids, "compact-in", &input, Side::TrainingInput),
        admit(&grids, "compact-out", &output, Side::TrainingOutput),
    )];
    let candidates = machine
        .agree_canonical_whole_grid_transforms(&lessons)
        .unwrap();
    let program = candidates
        .iter()
        .find(|candidate| candidate.root.child(b"separator-compaction").is_some())
        .unwrap();
    let query_pixels = vec![
        vec![Some(5), Some(8), Some(6), Some(8), Some(7)],
        vec![Some(8), Some(8), Some(8), Some(8), Some(8)],
        vec![Some(1), Some(8), Some(2), Some(8), Some(3)],
    ];
    let expected = vec![
        vec![Some(5), Some(6), Some(7)],
        vec![Some(1), Some(2), Some(3)],
    ];
    let query = admit(&grids, "compact-query", &query_pixels, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn paired_lines(h: usize, w: usize, roles: &[PairedRole]) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    let mut output = input.clone();
    for &(color, first, second) in roles {
        input[first.0][first.1] = Some(color);
        input[second.0][second.1] = Some(color);
        if first.0 == second.0 {
            for cell in output[first.0]
                .iter_mut()
                .take(first.1.max(second.1) + 1)
                .skip(first.1.min(second.1))
            {
                if *cell == Some(0) {
                    *cell = Some(color);
                }
            }
        } else {
            for row in output
                .iter_mut()
                .take(first.0.max(second.0) + 1)
                .skip(first.0.min(second.0))
            {
                if row[first.1] == Some(0) {
                    row[first.1] = Some(color);
                }
            }
        }
    }
    for &(color, first, second) in roles {
        output[first.0][first.1] = Some(color);
        output[second.0][second.1] = Some(color);
    }
    (input, output)
}

#[test]
fn complete_role_pairs_induce_axis_segments() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        paired_lines(12, 12, &[(3, (2, 1), (2, 8)), (7, (4, 5), (10, 5))]),
        paired_lines(15, 10, &[(2, (1, 2), (12, 2)), (9, (8, 0), (8, 9))]),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("ui{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("uo{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let candidates = machine.agree_generalized_candidates(&lessons).unwrap();
    assert_eq!(candidates.candidates().len(), 2);
    let program = candidates.primary().unwrap().program();
    let alternative = candidates.secondary().unwrap();
    assert_eq!(alternative.origin(), GeneralizedCandidateOrigin::Broad);
    assert!(
        program
            .root
            .child(b"component-pair-fill-construction")
            .is_some()
    );
    assert!(
        alternative
            .program()
            .root
            .child(b"paired-line-construction")
            .is_some()
    );
    assert!(!program.root.same(&alternative.program().root));
    assert_canonical_mask_composition(
        &program.root,
        b"component-pair-fill-construction",
        b"same-role-equal-shape-pairs",
        b"axis-periodic-component-interpolation",
        b"earlier-background-only-layer-wins",
    );
    let addressed = candidates
        .root
        .child(b"ordered-replay-valid-candidates")
        .unwrap();
    for (index, candidate) in candidates.candidates().iter().enumerate() {
        let entry = addressed.child(&index.to_be_bytes()).unwrap();
        assert!(
            entry
                .child(b"program")
                .unwrap()
                .follow()
                .unwrap()
                .same(&candidate.program().root)
        );
        assert!(
            entry
                .child(b"all-demonstrations-replayed")
                .unwrap()
                .is_terminal()
        );
    }

    let (query, expected) = paired_lines(14, 14, &[(8, (1, 7), (11, 7)), (4, (3, 2), (3, 12))]);
    let query = admit(&grids, "uq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);

    let (divergent, _) = paired_lines(11, 11, &[(8, (1, 3), (9, 3)), (4, (5, 1), (5, 9))]);
    let divergent = admit(&grids, "uq-divergent", &divergent, Side::QueryInput);
    assert!(
        machine
            .source_pair_applicable(&divergent, alternative.program())
            .unwrap()
    );
    let primary_output = machine
        .apply_groups(&mut grids, &divergent, program)
        .unwrap();
    let secondary_output = machine
        .apply_groups(&mut grids, &divergent, alternative.program())
        .unwrap();
    assert_ne!(
        primary_output.inspect().unwrap(),
        secondary_output.inspect().unwrap()
    );
}

#[test]
fn same_role_point_pairs_induce_canonical_cardinal_segments() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        paired_lines(9, 11, &[(3, (2, 1), (2, 8))]),
        paired_lines(12, 8, &[(7, (1, 5), (10, 5))]),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("pli{index}"), &input, Side::TrainingInput);
        let output = admit(
            &grids,
            &format!("plo{index}"),
            &output,
            Side::TrainingOutput,
        );
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"paired-line-construction").is_some());
    assert_canonical_cardinal_segment_placement(
        &program.root,
        b"paired-line-construction",
        b"same-role-aligned-point-pairs",
        b"input-snapshot-canonical-mask-layers",
        b"earlier-background-only-segment-layer-wins",
    );

    let mut malformed = vec![vec![Some(0); 13]; 10];
    malformed[2][3] = Some(4);
    malformed[7][9] = Some(4);
    let malformed = admit(&grids, "plx", &malformed, Side::QueryInput);
    assert!(
        machine
            .apply_groups(&mut grids, &malformed, &program)
            .is_err()
    );

    let (query, expected) = paired_lines(10, 13, &[(4, (6, 1), (6, 11))]);
    let query = admit(&grids, "plq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}

fn ordinal_columns(h: usize, w: usize, columns: &[(usize, Vec<(usize, u8)>)]) -> (Pixels, Pixels) {
    let mut input = vec![vec![Some(0); w]; h];
    let mut output = input.clone();
    for (x, markers) in columns {
        let mut previous = None;
        for &(y, color) in markers {
            input[y][*x] = Some(color);
            for row in output
                .iter_mut()
                .take(y + 1)
                .skip(previous.map_or(0, |at| at + 1))
            {
                row[*x] = Some(color);
            }
            previous = Some(y);
        }
    }
    (input, output)
}

#[test]
fn bottom_bound_columns_induce_ordinal_interval_fill() {
    let mut grids = Grids::new();
    let mut machine = SparseRewrite::new(&grids);
    let fixtures = [
        ordinal_columns(
            12,
            12,
            &[
                (2, vec![(1, 3), (3, 3), (7, 4), (11, 5)]),
                (8, vec![(4, 7), (5, 6), (8, 8), (11, 5)]),
            ],
        ),
        ordinal_columns(
            9,
            10,
            &[
                (3, vec![(2, 1), (5, 2), (8, 5)]),
                (7, vec![(0, 9), (4, 6), (8, 5)]),
            ],
        ),
    ];
    let mut lessons = Vec::new();
    for (index, (input, output)) in fixtures.into_iter().enumerate() {
        let input = admit(&grids, &format!("vi{index}"), &input, Side::TrainingInput);
        let output = admit(&grids, &format!("vo{index}"), &output, Side::TrainingOutput);
        lessons.push(
            machine
                .derive_groups(&index.to_be_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let program = machine
        .agree_generalized_actions(&lessons)
        .unwrap()
        .unwrap();
    assert!(
        program
            .root
            .child(b"ordinal-column-fill-construction")
            .is_some()
    );
    let (query, expected) = ordinal_columns(
        11,
        13,
        &[
            (4, vec![(3, 2), (7, 8), (10, 5)]),
            (9, vec![(1, 6), (10, 5)]),
        ],
    );
    let query = admit(&grids, "vq", &query, Side::QueryInput);
    let actual = machine.apply_groups(&mut grids, &query, &program).unwrap();
    assert_eq!(actual.inspect().unwrap(), expected);
}
