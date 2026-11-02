#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{
    arc_colors::ColorError,
    arc_grids::{Grid, Grids, Side},
    arc_markers::MarkerModel,
    records::Selection,
};
fn state(
    n: usize,
    object: u8,
    marker: Option<u8>,
    offset: (usize, usize),
    output: bool,
) -> Vec<Vec<Option<u8>>> {
    let mut a = vec![vec![Some(0); n]; n];
    for (y, x) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        a[offset.0 + y][offset.1 + x] = Some(if output { marker.unwrap() } else { object });
    }
    if !output {
        a[n - 1][0] = marker.map_or(Some(0), Some);
    }
    a
}
fn admit(s: &Grids, id: &[u8], side: Side, a: &[Vec<Option<u8>>]) -> Grid {
    let raw = format!("{a:?}");
    let src = s.source(id, raw.as_bytes()).unwrap();
    s.admit(id, side, a, &src).unwrap()
}
fn lesson(m: &mut MarkerModel, s: &Grids, id: &[u8], n: usize, object: u8, marker: u8) {
    let a = state(n, object, Some(marker), (1, 1), false);
    let b = state(n, object, Some(marker), (1, 1), true);
    let i = admit(s, &[id, b"i"].concat(), Side::TrainingInput, &a);
    let o = admit(s, &[id, b"o"].concat(), Side::TrainingOutput, &b);
    m.teach(id, &i, &o).unwrap();
}
fn trained(s: &Grids) -> MarkerModel {
    let mut m = MarkerModel::new(s, b"task").unwrap();
    lesson(&mut m, s, b"a", 5, 3, 6);
    lesson(&mut m, s, b"b", 7, 2, 4);
    m
}
#[test]
fn variable_marker_and_object_traits_positions_sizes_transfer_with_native_emission() {
    let mut s = Grids::new();
    let mut m = trained(&s);
    let a = state(9, 8, Some(2), (3, 4), false);
    let q = admit(&s, b"q", Side::QueryInput, &a);
    let mut run = m.begin(b"query", &q).unwrap();
    assert!(m.last.region_copies > 0 && m.last.role_copies > 0);
    heap::start();
    m.execute(&mut run, 81).unwrap();
    let sample = heap::finish();
    assert_eq!(sample.requests, 0);
    assert_eq!(run.cycles, 162);
    let out = m.record(&mut s, run).unwrap();
    assert_eq!(out.inspect().unwrap(), state(9, 8, Some(2), (3, 4), true));
    assert!(
        out.anchor
            .child(b"construction")
            .unwrap()
            .follow()
            .unwrap()
            .same(&m.base.source_construction)
    );
    let mut shape = vec![vec![Some(0); 9]; 9];
    let mut want = shape.clone();
    for (y, x) in [(2, 3), (3, 3), (4, 3), (4, 4), (4, 5)] {
        shape[y][x] = Some(5);
        want[y][x] = Some(7);
    }
    shape[8][0] = Some(7);
    let q2 = admit(&s, b"q-shape", Side::QueryInput, &shape);
    let mut next = m.begin(b"query-shape", &q2).unwrap();
    m.execute(&mut next, 81).unwrap();
    assert_eq!(m.record(&mut s, next).unwrap().inspect().unwrap(), want);
}
#[test]
fn missing_marker_is_unresolved_data_not_literal_training_answer() {
    let mut s = Grids::new();
    let mut m = trained(&s);
    let q = admit(
        &s,
        b"q",
        Side::QueryInput,
        &state(5, 3, None, (1, 1), false),
    );
    let mut run = m.begin(b"query", &q).unwrap();
    m.execute(&mut run, 25).unwrap();
    let a = m.record(&mut s, run).unwrap().inspect().unwrap();
    assert_eq!(a[1][1], None);
    assert_eq!(a[0][0], Some(0));
}
#[test]
fn conflicting_singleton_colors_remain_ambiguous() {
    let mut s = Grids::new();
    let mut m = trained(&s);
    let mut a = state(7, 8, Some(2), (1, 1), false);
    a[6][6] = Some(7);
    let q = admit(&s, b"q", Side::QueryInput, &a);
    let mut run = m.begin(b"query", &q).unwrap();
    m.execute(&mut run, 49).unwrap();
    let out = m.record(&mut s, run).unwrap().inspect().unwrap();
    assert_eq!(out[1][1], None);
    assert_eq!(out[6][0], Some(0));
    assert_eq!(out[6][6], Some(0));
}
#[test]
fn source_expression_conflict_never_bypasses_to_color_evidence() {
    let mut s = Grids::new();
    let mut m = trained(&s);
    let i = admit(
        &s,
        b"ci",
        Side::TrainingInput,
        &state(7, 2, Some(7), (1, 1), false),
    );
    let mut a = state(7, 2, Some(7), (1, 1), true);
    for row in &mut a[1..3] {
        for cell in &mut row[1..3] {
            *cell = Some(9);
        }
    }
    let o = admit(&s, b"co", Side::TrainingOutput, &a);
    m.teach(b"conflict", &i, &o).unwrap();
    let q = admit(
        &s,
        b"q",
        Side::QueryInput,
        &state(5, 3, Some(2), (1, 1), false),
    );
    let value = q
        .cell(1, 1)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    assert!(matches!(
        m.base.lookup(&value).unwrap(),
        Selection::Unique { .. }
    ));
    let mut run = m.begin(b"query", &q).unwrap();
    m.execute(&mut run, 25).unwrap();
    assert!(run.ambiguous > 0);
    assert_eq!(
        m.record(&mut s, run).unwrap().inspect().unwrap()[1][1],
        None
    );
}
#[test]
fn budget_error_remains_error_not_unknown_output() {
    let s = Grids::new();
    let mut m = MarkerModel::new(&s, b"task").unwrap();
    m.max_workspace_bytes = 1;
    let i = admit(
        &s,
        b"i",
        Side::TrainingInput,
        &state(5, 3, Some(6), (1, 1), false),
    );
    let o = admit(
        &s,
        b"o",
        Side::TrainingOutput,
        &state(5, 3, Some(6), (1, 1), true),
    );
    assert!(matches!(
        m.teach(b"e", &i, &o),
        Err(ColorError::WorkspaceLimit)
    ));
}
#[test]
fn active_query_and_teaching_freeze_are_explicit() {
    let mut s = Grids::new();
    let mut m = trained(&s);
    let q = admit(
        &s,
        b"q",
        Side::QueryInput,
        &state(5, 8, Some(2), (1, 1), false),
    );
    let mut run = m.begin(b"query", &q).unwrap();
    assert!(matches!(
        m.begin(b"other", &q),
        Err(ColorError::NotComplete)
    ));
    assert!(matches!(
        m.teach(b"late", &q, &q),
        Err(ColorError::NotComplete)
    ));
    m.execute(&mut run, 25).unwrap();
    assert_eq!(
        m.record(&mut s, run).unwrap().inspect().unwrap()[1][1],
        Some(2)
    );
    let mut next = m.begin(b"next", &q).unwrap();
    m.execute(&mut next, 25).unwrap();
}
#[test]
fn stored_source_slot_is_actual_native_authority_not_host_expression_decoder() {
    let mut s = Grids::new();
    let mut m = trained(&s);
    let q = admit(
        &s,
        b"q",
        Side::QueryInput,
        &state(5, 8, Some(2), (1, 1), false),
    );
    let donor = admit(&s, b"donor", Side::QueryInput, &[vec![Some(7)]]);
    let mut run = m.begin(b"query", &q).unwrap();
    m.marker_source.point_to(
        &donor
            .cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    m.execute(&mut run, 25).unwrap();
    assert_eq!(
        m.record(&mut s, run).unwrap().inspect().unwrap()[1][1],
        Some(7)
    );
}
#[test]
fn unknown_frontier_does_not_bind_singleton_colour() {
    let mut s = Grids::new();
    let mut m = trained(&s);
    let mut a = state(5, 8, Some(2), (1, 1), false);
    a[4][1] = None;
    let q = admit(&s, b"q", Side::QueryInput, &a);
    let mut run = m.begin(b"query", &q).unwrap();
    m.execute(&mut run, 25).unwrap();
    assert_eq!(
        m.record(&mut s, run).unwrap().inspect().unwrap()[1][1],
        None
    );
}
