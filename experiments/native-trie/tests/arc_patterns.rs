use native_trie_seed::{
    arc_colors::{ColorError, ColorModel},
    arc_grids::{Grid, Grids, Side},
    arc_patterns::{Axis, PatternModel},
    records::{self, Selection},
    source_field,
};
fn admit(s: &Grids, id: &[u8], side: Side, rows: &[&[u8]]) -> Grid {
    let source = s
        .source(
            id,
            b"literal test fixture; all values retained in source cells",
        )
        .unwrap();
    let cells: Vec<Vec<_>> = rows
        .iter()
        .map(|r| r.iter().map(|v| Some(*v)).collect())
        .collect();
    s.admit(id, side, &cells, &source).unwrap()
}
fn fixture(s: &Grids) -> (Grid, Grid) {
    (
        admit(
            s,
            b"before",
            Side::TrainingInput,
            &[&[1, 0, 1], &[0, 1, 0], &[1, 0, 1]],
        ),
        admit(
            s,
            b"after",
            Side::TrainingOutput,
            &[&[1, 6, 1], &[0, 1, 0], &[1, 6, 1]],
        ),
    )
}
fn predict(m: &mut PatternModel, s: &mut Grids, id: &[u8], g: &Grid) -> (Grid, u64) {
    let mut r = m.begin(id, g).unwrap();
    m.execute(&mut r, g.width() * g.height()).unwrap();
    let n = r.cycles;
    (m.record(s, r).unwrap(), n)
}
#[test]
fn actual_pair_learns_shared_horizontal_rule_and_vertical_preservation() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    let (out, n) = predict(&mut m, &mut s, b"query", &a);
    assert_eq!(n, 9);
    assert_eq!(out.inspect().unwrap(), b.inspect().unwrap());
    let vals = m
        .observe(&a.cell(0, 1).unwrap(), Axis::Horizontal)
        .unwrap()
        .unwrap();
    assert!(matches!(
        m.base.lookup(&vals[1]).unwrap(),
        Selection::Ambiguous { alternatives: 2 }
    ));
    let h = m.anchor(Axis::Horizontal, &vals).unwrap();
    let v = m.anchor(Axis::Vertical, &vals).unwrap();
    assert!(!h.same(&v));
    if let Selection::Unique { target, .. } = records::resolve(&h, b"output").unwrap() {
        assert!(
            target.same(
                &b.cell(0, 1)
                    .unwrap()
                    .child(b"value")
                    .unwrap()
                    .follow()
                    .unwrap()
            )
        )
    } else {
        panic!("horizontal not unique")
    }
    if let Selection::Unique { target, .. } = records::resolve(&v, b"output").unwrap() {
        assert!(
            target.same(
                &b.cell(1, 0)
                    .unwrap()
                    .child(b"value")
                    .unwrap()
                    .follow()
                    .unwrap()
            )
        )
    } else {
        panic!("vertical not unique")
    }
}
#[test]
fn new_sizes_and_positions_use_structure_not_coordinates() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    let q = admit(
        &s,
        b"horizontal",
        Side::QueryInput,
        &[&[1, 0, 1, 0, 1, 0, 1]],
    );
    assert_eq!(
        predict(&mut m, &mut s, b"qh", &q).0.inspect().unwrap(),
        vec![vec![
            Some(1),
            Some(6),
            Some(1),
            Some(6),
            Some(1),
            Some(6),
            Some(1)
        ]]
    );
    let q = admit(
        &s,
        b"vertical",
        Side::QueryInput,
        &[&[1], &[0], &[1], &[0], &[1], &[0], &[1]],
    );
    assert_eq!(
        predict(&mut m, &mut s, b"qv", &q).0.inspect().unwrap(),
        q.inspect().unwrap()
    );
}
#[test]
fn reached_axes_that_disagree_abstain_and_retain_both_proofs() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    let q = admit(
        &s,
        b"crossing",
        Side::QueryInput,
        &[&[0, 1, 0], &[1, 0, 1], &[0, 1, 0]],
    );
    let (out, _) = predict(&mut m, &mut s, b"query", &q);
    assert_eq!(out.inspect().unwrap()[1][1], None);
    let work = out
        .cell(1, 1)
        .unwrap()
        .child(b"derived-from")
        .unwrap()
        .follow()
        .unwrap();
    let views = work.child(b"contexts").unwrap();
    for key in [b"horizontal".as_slice(), b"vertical"] {
        assert!(
            views
                .child(key)
                .unwrap()
                .child(b"candidate")
                .unwrap()
                .follow()
                .is_ok()
        );
    }
}
#[test]
fn unknown_complete_context_does_not_fall_back_to_known_center_color() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    for (i, line) in [[1, 1, 0], [1, 0, 0], [2, 0, 2]].iter().enumerate() {
        let id = i.to_be_bytes();
        let q = admit(&s, &id, Side::QueryInput, &[line]);
        let out = predict(&mut m, &mut s, &id, &q).0;
        assert_eq!(out.inspect().unwrap()[0][1], None);
    }
}
#[test]
fn same_structural_address_accumulates_different_occurrences_without_duplication() {
    let s = Grids::new();
    let m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    m.teach(b"lesson", &a, &b).unwrap();
    let one = m
        .anchor(
            Axis::Horizontal,
            &m.observe(&a.cell(0, 1).unwrap(), Axis::Horizontal)
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    let two = m
        .anchor(
            Axis::Horizontal,
            &m.observe(&a.cell(2, 1).unwrap(), Axis::Horizontal)
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    assert!(one.same(&two));
    if let Selection::Unique { candidate, .. } = records::resolve(&one, b"output").unwrap() {
        let es = records::inspect_evidence(&candidate).unwrap();
        assert_eq!(es.len(), 2);
        for e in es {
            let source = e.follow().unwrap();
            assert!(source.child(b"input-cell").unwrap().follow().is_ok());
            assert!(source.child(b"output-cell").unwrap().follow().is_ok());
            assert!(
                source
                    .child(b"construction")
                    .unwrap()
                    .follow()
                    .unwrap()
                    .same(&m.base.construction)
            );
        }
    } else {
        panic!("ambiguous")
    }
}
#[test]
fn contradictory_specific_evidence_cannot_be_overridden_by_other_axis() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    let c = admit(&s, b"c", Side::TrainingInput, &[&[0, 1, 0]]);
    let d = admit(&s, b"d", Side::TrainingOutput, &[&[0, 9, 0]]);
    m.teach(b"conflict", &c, &d).unwrap();
    assert_eq!(
        predict(&mut m, &mut s, b"query", &a).0.inspect().unwrap()[1][1],
        None
    );
}
#[test]
fn snapshot_reads_do_not_apply_a_newly_created_context_in_same_pass() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    let c = admit(&s, b"c", Side::TrainingInput, &[&[6, 1, 0]]);
    let d = admit(&s, b"d", Side::TrainingOutput, &[&[6, 9, 0]]);
    m.teach(b"stage", &c, &d).unwrap();
    assert_eq!(
        predict(&mut m, &mut s, b"installed", &c)
            .0
            .inspect()
            .unwrap()[0][1],
        Some(9)
    );
    let q = admit(&s, b"q", Side::QueryInput, &[&[1, 0, 1, 0, 2]]);
    let old = q.inspect().unwrap();
    let out = predict(&mut m, &mut s, b"query", &q).0;
    let values = out.inspect().unwrap();
    assert_eq!(values[0][1], Some(6));
    assert_eq!(values[0][2], Some(1));
    assert_eq!(q.inspect().unwrap(), old);
}
#[test]
fn unrelated_model_growth_leaves_reached_addresses_outputs_and_copies_unchanged() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    let (before, n) = predict(&mut m, &mut s, b"first", &a);
    for i in 0..100usize {
        let other = ColorModel::new(&s, &i.to_be_bytes()).unwrap();
        other.teach(b"unrelated", &a, &b).unwrap();
    }
    let (after, k) = predict(&mut m, &mut s, b"second", &a);
    assert_eq!(n, k);
    assert_eq!(before.inspect().unwrap(), after.inspect().unwrap());
    for (y, x) in [(0, 1), (1, 1), (1, 0)] {
        for axis in [b"horizontal".as_slice(), b"vertical"] {
            let at = |g: &Grid| {
                g.cell(y, x)
                    .unwrap()
                    .child(b"derived-from")
                    .unwrap()
                    .follow()
                    .unwrap()
                    .child(b"contexts")
                    .unwrap()
                    .child(axis)
                    .unwrap()
                    .child(b"lookup")
                    .unwrap()
                    .follow()
                    .ok()
            };
            match (at(&before), at(&after)) {
                (Some(a), Some(b)) => assert!(a.same(&b)),
                (None, None) => (),
                _ => panic!("changed reached lookup"),
            }
        }
    }
}
#[test]
fn no_rules_partial_runs_and_bad_neighbor_facts_do_not_invent_outputs() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    assert!(
        predict(&mut m, &mut s, b"fresh", &a)
            .0
            .inspect()
            .unwrap()
            .iter()
            .flatten()
            .all(Option::is_none)
    );
    m.teach(b"lesson", &a, &b).unwrap();
    let mut run = m.begin(b"partial", &a).unwrap();
    assert!(matches!(m.execute(&mut run, 1), Err(ColorError::Limit)));
    assert!(matches!(
        m.record(&mut s, run),
        Err(ColorError::NotComplete)
    ));
    a.cell(0, 0).unwrap().child(b"east").unwrap().terminate();
    let mut bad = m.begin(b"bad", &a).unwrap();
    assert!(matches!(
        m.execute(&mut bad, 9),
        Err(ColorError::Machine(_))
    ));
}
#[test]
fn stored_constructor_controls_actual_emission_and_derived_lineage_is_not_observed() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let (a, b) = fixture(&s);
    m.teach(b"lesson", &a, &b).unwrap();
    source_field(&m.base.construction)
        .unwrap()
        .point_to(&a.cell(0, 0).unwrap().child(b"value").unwrap());
    let out = predict(&mut m, &mut s, b"query", &a).0;
    assert_eq!(out.inspect().unwrap(), vec![vec![Some(1); 3]; 3]);
    assert!(out.observed_source().unwrap().is_none());
    let work = out
        .cell(0, 1)
        .unwrap()
        .child(b"derived-from")
        .unwrap()
        .follow()
        .unwrap();
    assert!(
        work.child(b"input-cell")
            .unwrap()
            .follow()
            .unwrap()
            .same(&a.cell(0, 1).unwrap())
    );
}
#[test]
fn asymmetry_is_not_silently_reversed_or_shared_with_new_task() {
    let mut s = Grids::new();
    let mut m = PatternModel::new(&s, b"task").unwrap();
    let a = admit(&s, b"a", Side::TrainingInput, &[&[1, 0, 2]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[&[1, 6, 2]]);
    m.teach(b"lesson", &a, &b).unwrap();
    let q = admit(&s, b"q", Side::QueryInput, &[&[2, 0, 1]]);
    assert_eq!(
        predict(&mut m, &mut s, b"reverse", &q).0.inspect().unwrap()[0][1],
        None
    );
    let mut fresh = PatternModel::new(&s, b"other").unwrap();
    assert!(
        predict(&mut fresh, &mut s, b"query", &a)
            .0
            .inspect()
            .unwrap()
            .iter()
            .flatten()
            .all(Option::is_none)
    );
}
