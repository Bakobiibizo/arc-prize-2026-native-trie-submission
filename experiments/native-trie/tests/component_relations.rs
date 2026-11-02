use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{Derived, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn fixture(h: usize, w: usize, c: u8, inverse: bool) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(0); w]; h];
    let mut b = a.clone();
    for y in 0..h {
        for x in 0..w {
            let ring = (y == 1 || y == h - 2) && (1..w - 1).contains(&x)
                || (x == 1 || x == w - 2) && (1..h - 1).contains(&y);
            let inside = (2..h - 2).contains(&y) && (2..w - 2).contains(&x);
            if ring {
                a[y][x] = Some(c);
                b[y][x] = Some(c);
            } else if inside != inverse {
                b[y][x] = Some(4);
            }
        }
    }
    (a, b)
}
fn admit(s: &Grids, id: &str, a: &[Vec<Option<u8>>], side: Side) -> Grid {
    let src = s
        .source(id.as_bytes(), b"declared ring geometry control")
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn teach(
    s: &Grids,
    m: &mut SparseRewrite,
    id: &str,
    h: usize,
    w: usize,
    c: u8,
    inverse: bool,
) -> Derived {
    let (a, b) = fixture(h, w, c, inverse);
    let i = admit(s, &format!("{id}i"), &a, Side::TrainingInput);
    let o = admit(s, &format!("{id}o"), &b, Side::TrainingOutput);
    m.derive_component_relations(id.as_bytes(), &i, &o).unwrap()
}
#[test]
fn independently_learns_both_kind_values_and_literal_separately() {
    for inverse in [false, true] {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let a = teach(&s, &mut m, "a", 5, 7, 1, inverse);
        let b = teach(&s, &mut m, "b", 7, 9, 2, inverse);
        let rule = SparseRewrite::agree(&[a, b]).unwrap();
        assert_eq!(rule.clauses(), 1);
        let (i, want) = fixture(9, 11, 3, inverse);
        let q = admit(&s, "q", &i, Side::QueryInput);
        let out = m.apply(&mut s, &q, &rule).unwrap();
        assert_eq!(out.inspect().unwrap(), want);
        assert!(m.region_copies > 0 && m.region_cache_hits > 0);
        assert_eq!(m.region_proofs, 6);
        assert!(m.workspace_bytes() < 286720000);
    }
}
#[test]
fn unchanged_same_kind_and_unknown_frontier_reject() {
    let s = Grids::new();
    for unknown in [false, true] {
        let mut m = SparseRewrite::new(&s);
        let (mut i, mut o) = fixture(7, 9, 1, false);
        if unknown {
            i[3][3] = None;
            o[3][3] = None;
        } else {
            o[3][3] = Some(0);
        }
        let i = admit(
            &s,
            if unknown { "ui" } else { "ci" },
            &i,
            Side::TrainingInput,
        );
        let o = admit(
            &s,
            if unknown { "uo" } else { "co" },
            &o,
            Side::TrainingOutput,
        );
        let d = m.derive_component_relations(b"d", &i, &o).unwrap();
        assert!(d.rule.is_none());
        if unknown {
            assert!(
                d.evidence
                    .child(b"unavailable-positive-kinds")
                    .unwrap()
                    .child(b"holds")
                    .is_some()
            );
        } else {
            assert!(d.negative_checks > 0);
        }
    }
}
#[test]
fn stored_expected_kind_controls_execution_preserving_prior_query() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let a = teach(&s, &mut m, "a", 5, 7, 1, false);
    let b = teach(&s, &mut m, "b", 7, 9, 2, false);
    let rule = SparseRewrite::agree(&[a, b]).unwrap();
    let (i, inside) = fixture(9, 11, 3, false);
    let q = admit(&s, "q1", &i, Side::QueryInput);
    let out = m.apply(&mut s, &q, &rule).unwrap();
    assert_eq!(out.inspect().unwrap(), inside);
    let escape = m
        .root
        .child(b"native-components")
        .unwrap()
        .child(b"relation-tags")
        .unwrap()
        .child(b"escape")
        .unwrap();
    rule.root.child(b"expected-kind").unwrap().point_to(&escape);
    let q = admit(&s, "q2", &i, Side::QueryInput);
    let out2 = m.apply(&mut s, &q, &rule).unwrap();
    assert_eq!(out2.inspect().unwrap(), fixture(9, 11, 3, true).1);
    assert_eq!(out.inspect().unwrap(), inside);
}
