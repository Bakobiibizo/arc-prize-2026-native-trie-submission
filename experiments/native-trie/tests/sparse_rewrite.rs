use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::SparseRewrite,
};
fn grid(s: &Grids, id: &[u8], side: Side, a: &[Vec<u8>]) -> Grid {
    let src = s.source(id, b"sparse relation fixture").unwrap();
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
fn given() -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    (
        vec![
            vec![1, 0, 1, 0],
            vec![0, 1, 0, 1],
            vec![0, 0, 1, 1],
            vec![1, 1, 0, 1],
        ],
        vec![
            vec![1, 1, 1, 0],
            vec![0, 1, 1, 1],
            vec![0, 0, 1, 1],
            vec![1, 1, 1, 1],
        ],
    )
}
fn second() -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    (
        vec![
            vec![2, 0, 2, 0, 0, 0],
            vec![0; 6],
            vec![2, 0, 2, 2, 0, 2],
            vec![0; 6],
            vec![0, 2, 0, 2, 0, 0],
        ],
        vec![
            vec![2, 2, 2, 0, 0, 0],
            vec![0; 6],
            vec![2, 2, 2, 2, 2, 2],
            vec![0; 6],
            vec![0, 2, 2, 2, 0, 0],
        ],
    )
}
fn transpose(a: &[Vec<u8>]) -> Vec<Vec<u8>> {
    (0..a[0].len())
        .map(|x| a.iter().map(|r| r[x]).collect())
        .collect()
}
fn inspect(a: &[Vec<u8>]) -> Vec<Vec<Option<u8>>> {
    a.iter()
        .map(|r| r.iter().copied().map(Some).collect())
        .collect()
}
#[test]
fn sparse_relationships_transfer_across_shape_positions_and_target_colour() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, b) = given();
    let (c, d) = second();
    let i = grid(&s, b"i", Side::TrainingInput, &a);
    let o = grid(&s, b"o", Side::TrainingOutput, &b);
    let j = grid(&s, b"j", Side::TrainingInput, &c);
    let p = grid(&s, b"p", Side::TrainingOutput, &d);
    let first = m.derive(b"first", &i, &o).unwrap();
    let second = m.derive(b"second", &j, &p).unwrap();
    assert_eq!((first.changed, first.positive_tests), (3, 12));
    assert_eq!((second.changed, second.positive_tests), (4, 16));
    let rule = SparseRewrite::agree(&[first, second]).unwrap();
    assert_eq!(rule.clauses(), 2);
    assert_eq!(
        m.apply(&mut s, &i, &rule).unwrap().inspect().unwrap(),
        inspect(&b)
    );
    let q = grid(
        &s,
        b"q",
        Side::QueryInput,
        &[
            vec![3, 0, 3, 0, 3, 0, 0],
            vec![0, 0, 3, 0, 3, 0, 3],
            vec![3, 0, 0, 3, 0, 3, 0],
        ],
    );
    let want = vec![
        vec![3, 3, 3, 3, 3, 0, 0],
        vec![0, 0, 3, 3, 3, 3, 3],
        vec![3, 0, 0, 3, 3, 3, 0],
    ];
    assert_eq!(
        m.apply(&mut s, &q, &rule).unwrap().inspect().unwrap(),
        inspect(&want)
    );
    assert!(m.derive(b"late", &i, &o).is_err());
}
#[test]
fn vertical_relations_are_learned_and_do_not_match_horizontal() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, b) = given();
    let (c, d) = second();
    let i = grid(&s, b"i", Side::TrainingInput, &a);
    let o = grid(&s, b"o", Side::TrainingOutput, &b);
    let vi = grid(&s, b"vi", Side::TrainingInput, &transpose(&a));
    let vo = grid(&s, b"vo", Side::TrainingOutput, &transpose(&b));
    let vj = grid(&s, b"vj", Side::TrainingInput, &transpose(&c));
    let vp = grid(&s, b"vp", Side::TrainingOutput, &transpose(&d));
    let h = m.derive(b"h", &i, &o).unwrap();
    let v = m.derive(b"v", &vi, &vo).unwrap();
    assert!(
        !h.rule
            .as_ref()
            .unwrap()
            .root
            .same(&v.rule.as_ref().unwrap().root)
    );
    assert!(SparseRewrite::agree(&[h, v]).is_none());
    let v = m.derive(b"v2", &vi, &vo).unwrap();
    let w = m.derive(b"w", &vj, &vp).unwrap();
    let rule = SparseRewrite::agree(&[v, w]).unwrap();
    assert_eq!(
        m.apply(&mut s, &vi, &rule).unwrap().inspect().unwrap(),
        inspect(&transpose(&b))
    );
}
#[test]
fn unchanged_qualifying_cell_is_a_native_counterexample() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, mut b) = given();
    b[1][2] = 0;
    let i = grid(&s, b"i", Side::TrainingInput, &a);
    let o = grid(&s, b"o", Side::TrainingOutput, &b);
    let d = m.derive(b"d", &i, &o).unwrap();
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
fn empty_changes_and_multiple_changing_source_colours_do_not_invent_rule() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let i = grid(&s, b"i", Side::TrainingInput, &[vec![0, 1, 0]]);
    let o = grid(&s, b"o", Side::TrainingOutput, &[vec![2, 3, 2]]);
    assert!(m.derive(b"same", &i, &i).unwrap().rule.is_none());
    assert!(m.derive(b"multi", &i, &o).unwrap().rule.is_none());
}
#[test]
fn positive_relation_storage_tracks_changes_not_grid_area() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut rules = Vec::new();
    for n in [5usize, 12] {
        let mut a = vec![vec![0; n]; n];
        let mut b = a.clone();
        for y in [0, n / 2, n - 1] {
            a[y][0] = 1;
            a[y][2] = 1;
            b[y][0] = 1;
            b[y][1] = 1;
            b[y][2] = 1;
        }
        let i = grid(&s, format!("i{n}").as_bytes(), Side::TrainingInput, &a);
        let o = grid(&s, format!("o{n}").as_bytes(), Side::TrainingOutput, &b);
        let d = m.derive(format!("d{n}").as_bytes(), &i, &o).unwrap();
        assert_eq!(
            (
                d.changed,
                d.positive_tests,
                d.rule.as_ref().unwrap().clauses()
            ),
            (3, 12, 2)
        );
        assert_eq!(d.negative_checks, n * n - 9);
        rules.push(d);
    }
    assert!(SparseRewrite::agree(&rules).is_some());
}
