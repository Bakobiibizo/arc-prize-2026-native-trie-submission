use native_trie_seed::{
    arc_colors::{ColorError, ColorModel},
    arc_grids::{Grid, Grids, Side},
    records::{self, Selection},
    source_field,
};
fn admit(space: &Grids, id: &[u8], side: Side, rows: &[&[u8]]) -> Grid {
    let source = space
        .source(
            id,
            b"literal supplied test values are retained in the admitted grid",
        )
        .unwrap();
    let values: Vec<Vec<_>> = rows
        .iter()
        .map(|r| r.iter().map(|v| Some(*v)).collect())
        .collect();
    space.admit(id, side, &values, &source).unwrap()
}
fn predict(model: &mut ColorModel, space: &mut Grids, id: &[u8], input: &Grid) -> Grid {
    let mut run = model.begin(id, input).unwrap();
    model
        .execute(&mut run, input.width() * input.height())
        .unwrap();
    assert_eq!(run.cycles, (input.width() * input.height()) as u64);
    model.record(space, run).unwrap()
}
#[test]
fn learns_actual_pairs_at_new_shape_and_location_without_a_supplied_mapping() {
    let mut s = Grids::new();
    let mut m = ColorModel::new(&s, b"task").unwrap();
    let a = admit(&s, b"a", Side::TrainingInput, &[&[1, 2], &[2, 1]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[&[8, 4], &[4, 8]]);
    let q = admit(&s, b"q", Side::QueryInput, &[&[2, 1, 2, 2, 1]]);
    assert_eq!(
        predict(&mut m, &mut s, b"untaught", &q).inspect().unwrap(),
        vec![vec![None; 5]]
    );
    m.teach(b"lesson", &a, &b).unwrap();
    let out = predict(&mut m, &mut s, b"taught", &q);
    assert_eq!(
        out.inspect().unwrap(),
        vec![vec![Some(4), Some(8), Some(4), Some(4), Some(8)]]
    );
    assert!(out.observed_source().unwrap().is_none());
    assert_eq!(
        q.inspect().unwrap(),
        vec![vec![Some(2), Some(1), Some(2), Some(2), Some(1)]]
    );
}
#[test]
fn contradictory_and_unseen_traits_abstain_without_identity_or_last_write_wins() {
    let mut s = Grids::new();
    let mut m = ColorModel::new(&s, b"task").unwrap();
    let a = admit(&s, b"a", Side::TrainingInput, &[&[1, 1, 2]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[&[8, 9, 4]]);
    m.teach(b"lesson", &a, &b).unwrap();
    let q = admit(&s, b"q", Side::QueryInput, &[&[1, 2, 3]]);
    let mut run = m.begin(b"query", &q).unwrap();
    m.execute(&mut run, 3).unwrap();
    assert_eq!((run.ambiguous, run.unknown), (1, 1));
    let out = m.record(&mut s, run).unwrap();
    assert_eq!(out.inspect().unwrap(), vec![vec![None, Some(4), None]]);
    let value = a
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    assert!(matches!(
        m.lookup(&value).unwrap(),
        Selection::Ambiguous { alternatives: 2 }
    ));
}
#[test]
fn evidence_retains_both_observed_sides_and_exact_replay_is_idempotent() {
    let s = Grids::new();
    let m = ColorModel::new(&s, b"task").unwrap();
    let a = admit(&s, b"a", Side::TrainingInput, &[&[1]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[&[7]]);
    m.teach(b"lesson", &a, &b).unwrap();
    m.teach(b"lesson", &a, &b).unwrap();
    let v = a
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    if let Selection::Unique { candidate, .. } = m.lookup(&v).unwrap() {
        let e = records::inspect_evidence(&candidate).unwrap();
        assert_eq!(e.len(), 1);
        assert!(
            e[0].follow()
                .unwrap()
                .child(b"input-cell")
                .unwrap()
                .follow()
                .unwrap()
                .same(&a.cell(0, 0).unwrap())
        );
        assert!(
            e[0].follow()
                .unwrap()
                .child(b"output-cell")
                .unwrap()
                .follow()
                .unwrap()
                .same(&b.cell(0, 0).unwrap())
        );
    } else {
        panic!("not unique")
    }
}
#[test]
fn stored_instruction_is_causal_and_no_host_recolor_fallback_exists() {
    let mut s = Grids::new();
    let mut m = ColorModel::new(&s, b"task").unwrap();
    let a = admit(&s, b"a", Side::TrainingInput, &[&[1]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[&[7]]);
    m.teach(b"lesson", &a, &b).unwrap();
    source_field(&m.construction)
        .unwrap()
        .point_to(&a.cell(0, 0).unwrap().child(b"value").unwrap());
    let out = predict(&mut m, &mut s, b"query", &a);
    assert_eq!(out.inspect().unwrap(), vec![vec![Some(1)]]);
}
#[test]
fn incomplete_run_has_no_valid_grid_and_new_scope_has_no_task_knowledge() {
    let mut s = Grids::new();
    let mut m = ColorModel::new(&s, b"task").unwrap();
    let a = admit(&s, b"a", Side::TrainingInput, &[&[1, 1]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[&[7, 7]]);
    m.teach(b"lesson", &a, &b).unwrap();
    let mut run = m.begin(b"query", &a).unwrap();
    assert!(matches!(m.execute(&mut run, 1), Err(ColorError::Limit)));
    assert!(matches!(
        m.record(&mut s, run),
        Err(ColorError::NotComplete)
    ));
    let mut fresh = ColorModel::new(&s, b"other-task").unwrap();
    assert_eq!(
        predict(&mut fresh, &mut s, b"other-query", &a)
            .inspect()
            .unwrap(),
        vec![vec![None, None]]
    );
}
#[test]
fn operator_gap_example_exposes_missing_structural_context_not_a_missing_primitive() {
    let mut s = Grids::new();
    let mut m = ColorModel::new(&s, b"gap").unwrap();
    let a = admit(
        &s,
        b"a",
        Side::TrainingInput,
        &[&[1, 0, 1], &[0, 1, 0], &[1, 0, 1]],
    );
    let b = admit(
        &s,
        b"b",
        Side::TrainingOutput,
        &[&[1, 6, 1], &[0, 1, 0], &[1, 6, 1]],
    );
    m.teach(b"example", &a, &b).unwrap();
    let out = predict(&mut m, &mut s, b"query", &a);
    assert_eq!(
        out.inspect().unwrap(),
        vec![
            vec![Some(1), None, Some(1)],
            vec![None, Some(1), None],
            vec![Some(1), None, Some(1)]
        ]
    );
    let gap = a
        .cell(0, 1)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    assert!(matches!(
        m.lookup(&gap).unwrap(),
        Selection::Ambiguous { alternatives: 2 }
    ));
}
#[test]
fn shapes_are_family_constraints_not_silently_truncated_pairs() {
    let s = Grids::new();
    let m = ColorModel::new(&s, b"task").unwrap();
    let a = admit(&s, b"a", Side::TrainingInput, &[&[1, 2]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[&[7]]);
    assert!(matches!(m.teach(b"bad", &a, &b), Err(ColorError::Grid(_))));
    assert!(m.root.child(b"examples").is_none());
}
