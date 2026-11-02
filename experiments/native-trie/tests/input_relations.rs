use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::SparseRewrite,
};
fn g(s: &Grids, id: &[u8], side: Side, a: &[Vec<u8>]) -> Grid {
    let src = s
        .source(id, b"independent input selection fixture")
        .unwrap();
    s.admit(
        id,
        side,
        &a.iter()
            .map(|r| r.iter().copied().map(Some).collect())
            .collect::<Vec<_>>(),
        &src,
    )
    .unwrap()
}
fn a() -> Vec<Vec<u8>> {
    vec![
        vec![1, 0, 1, 0],
        vec![0, 1, 0, 1],
        vec![0, 0, 1, 1],
        vec![1, 1, 0, 1],
    ]
}
fn b() -> Vec<Vec<u8>> {
    vec![
        vec![2, 0, 2, 0, 0, 0],
        vec![0; 6],
        vec![2, 0, 2, 2, 0, 2],
        vec![0; 6],
        vec![0, 2, 0, 2, 0, 0],
    ]
}
fn changed(mut a: Vec<Vec<u8>>, positions: &[(usize, usize)], target: u8) -> Vec<Vec<u8>> {
    for &(y, x) in positions {
        a[y][x] = target;
    }
    a
}
fn opt(a: &[Vec<u8>]) -> Vec<Vec<Option<u8>>> {
    a.iter()
        .map(|r| r.iter().copied().map(Some).collect())
        .collect()
}
#[test]
fn selection_does_not_require_the_target_value_in_any_input_neighbour() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let i = g(&s, b"i", Side::TrainingInput, &a());
    let o = g(
        &s,
        b"o",
        Side::TrainingOutput,
        &changed(a(), &[(0, 1), (1, 2), (3, 2)], 4),
    );
    assert!(m.derive(b"old", &i, &o).unwrap().rule.is_none());
    let j = g(&s, b"j", Side::TrainingInput, &b());
    let p = g(
        &s,
        b"p",
        Side::TrainingOutput,
        &changed(b(), &[(0, 1), (2, 1), (2, 4), (4, 2)], 4),
    );
    let d = m.derive_input_relations(b"d", &i, &o).unwrap();
    let e = m.derive_input_relations(b"e", &j, &p).unwrap();
    assert_eq!(
        (
            d.changed,
            d.positive_tests,
            d.rule.as_ref().unwrap().clauses()
        ),
        (3, 30, 3)
    );
    assert_eq!(e.positive_tests, 40);
    let rule = SparseRewrite::agree(&[d, e]).unwrap();
    let raw = vec![
        vec![3, 0, 3, 0, 3, 0, 0],
        vec![0, 0, 3, 0, 3, 0, 3],
        vec![3, 0, 0, 3, 0, 3, 0],
    ];
    let want = changed(raw.clone(), &[(0, 1), (0, 3), (1, 3), (1, 5), (2, 4)], 4);
    let q = g(&s, b"q", Side::QueryInput, &raw);
    assert_eq!(
        m.apply(&mut s, &q, &rule).unwrap().inspect().unwrap(),
        opt(&want)
    );
}
#[test]
fn unchanged_qualifying_cell_still_rejects_selection_independent_of_action() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let i = g(&s, b"i", Side::TrainingInput, &a());
    let o = g(
        &s,
        b"o",
        Side::TrainingOutput,
        &changed(a(), &[(0, 1), (3, 2)], 4),
    );
    let d = m.derive_input_relations(b"d", &i, &o).unwrap();
    assert!(d.rule.is_none());
    assert!(
        d.evidence
            .child(b"counterexamples")
            .unwrap()
            .child(b"holds")
            .is_some()
    );
}
#[test]
fn different_literal_actions_do_not_masquerade_as_same_construction() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let i = g(&s, b"i", Side::TrainingInput, &a());
    let o = g(
        &s,
        b"o",
        Side::TrainingOutput,
        &changed(a(), &[(0, 1), (1, 2), (3, 2)], 4),
    );
    let p = g(
        &s,
        b"p",
        Side::TrainingOutput,
        &changed(a(), &[(0, 1), (1, 2), (3, 2)], 5),
    );
    let d = m.derive_input_relations(b"d", &i, &o).unwrap();
    let e = m.derive_input_relations(b"e", &i, &p).unwrap();
    assert!(d.rule.is_some() && e.rule.is_some());
    assert!(SparseRewrite::agree(&[d, e]).is_none());
}
