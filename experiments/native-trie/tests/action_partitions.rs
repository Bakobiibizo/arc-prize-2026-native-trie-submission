use native_trie_seed::{
    Address,
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupLesson, GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn fixture(h: usize, w: usize, c: u8, shift: bool, coarse: bool) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(0); w]; h];
    let mut b = a.clone();
    let places = if shift {
        [(2, 2, 2, 4, 8), (6, 11, 4, 6, 4)]
    } else {
        [(1, 1, 2, 4, 8), (1, 9, 4, 6, 4)]
    };
    for (y, x, ih, iw, target) in places {
        for dy in 0..ih + 2 {
            for dx in 0..iw + 2 {
                if dy == 0 || dx == 0 || dy == ih + 1 || dx == iw + 1 {
                    a[y + dy][x + dx] = Some(c);
                    b[y + dy][x + dx] = Some(c);
                } else {
                    b[y + dy][x + dx] = Some(if coarse { 4 } else { target });
                }
            }
        }
    }
    (a, b)
}
fn transpose(a: &Pixels) -> Pixels {
    (0..a[0].len())
        .map(|x| (0..a.len()).map(|y| a[y][x]).collect())
        .collect()
}
fn grid(s: &Grids, id: &str, a: &Pixels, side: Side) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"declared independent action partition fixture",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn teach(s: &Grids, m: &mut SparseRewrite, id: &str, a: &Pixels, b: &Pixels) -> GroupLesson {
    let i = grid(s, &format!("{id}i"), a, Side::TrainingInput);
    let o = grid(s, &format!("{id}o"), b, Side::TrainingOutput);
    m.derive_groups(id.as_bytes(), &i, &o).unwrap()
}
fn learned(s: &Grids, m: &mut SparseRewrite, coarse: bool) -> GroupProgram {
    let (a, b) = fixture(9, 19, 1, false, coarse);
    let first = teach(s, m, "a", &a, &b);
    let (a, b) = fixture(9, 19, 2, false, coarse);
    let second = teach(s, m, "b", &transpose(&a), &transpose(&b));
    m.agree_partitions(&[first, second]).unwrap().unwrap()
}
#[test]
fn same_colour_same_kind_two_actions_from_independent_complete_counts() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let program = learned(&s, &mut m, false);
    assert_eq!(program.groups(), 2);
    let (a, want) = fixture(13, 21, 3, true, false);
    let q = grid(&s, "q", &a, Side::QueryInput);
    let out = m.apply_groups(&mut s, &q, &program).unwrap();
    assert_eq!(out.inspect().unwrap(), want);
    assert_eq!(m.count_proofs, 6);
    assert!(m.count_copies > 0);
    assert!(m.workspace_bytes() < 286720000);
}
#[test]
fn coarser_consistent_kind_rule_is_not_overconditioned_by_size() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let program = learned(&s, &mut m, true);
    assert_eq!(program.groups(), 1);
    let leaf = program
        .root
        .child(b"groups")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .follow()
        .unwrap();
    assert!(leaf.child(b"expected-kind").is_some());
    assert!(leaf.child(b"expected-count").is_none());
    let (a, want) = fixture(13, 21, 3, true, true);
    let q = grid(&s, "q", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        want
    );
    assert_eq!(m.count_proofs, 0);
}
#[test]
fn identical_input_tuple_different_actions_rejects() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, b) = fixture(9, 19, 1, false, false);
    let first = teach(&s, &mut m, "a", &a, &b);
    let (a, b) = fixture(9, 19, 2, false, true);
    let second = teach(&s, &mut m, "b", &a, &b);
    assert!(m.agree_partitions(&[first, second]).unwrap().is_none());
}
#[test]
fn unchanged_same_source_cells_are_constraints_not_ignored() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, mut b) = fixture(9, 19, 1, false, false);
    for row in b.iter_mut().take(4).skip(2) {
        for v in row.iter_mut().take(6).skip(2) {
            *v = Some(0);
        }
    }
    let d = teach(&s, &mut m, "a", &a, &b);
    let program = m.agree_partitions(&[d]).unwrap().unwrap();
    assert_eq!(program.groups(), 1);
    let q = grid(&s, "q", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        b
    );
}
#[test]
fn unknown_positive_count_or_kind_cannot_bind() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (mut a, mut b) = fixture(9, 19, 1, false, false);
    a[2][2] = None;
    b[2][2] = None;
    let d = teach(&s, &mut m, "a", &a, &b);
    assert!(m.agree_partitions(&[d]).unwrap().is_none());
}
fn unary(m: &SparseRewrite, n: usize) -> Address {
    let mut at = m
        .root
        .child(b"native-components")
        .unwrap()
        .child(b"cardinalities")
        .unwrap()
        .child(b"zero")
        .unwrap();
    for _ in 0..n {
        at = at.child(b"successor").unwrap();
    }
    at
}
#[test]
fn stored_count_condition_controls_execution_preserving_old_output() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let program = learned(&s, &mut m, false);
    let (a, want) = fixture(13, 21, 3, true, false);
    let q = grid(&s, "q1", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &program).unwrap();
    let eight = unary(&m, 8);
    let leaf = (0..program.groups())
        .map(|i| {
            program
                .root
                .child(b"groups")
                .unwrap()
                .child(&i.to_be_bytes())
                .unwrap()
                .follow()
                .unwrap()
        })
        .find(|g| {
            g.child(b"expected-count")
                .unwrap()
                .follow()
                .unwrap()
                .same(&eight)
        })
        .unwrap();
    leaf.child(b"expected-count")
        .unwrap()
        .point_to(&unary(&m, 1));
    let q = grid(&s, "q2", &a, Side::QueryInput);
    let output = m
        .apply_groups(&mut s, &q, &program)
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(output[3][3], Some(0));
    assert_eq!(output[7][12], Some(4));
    assert_eq!(old.inspect().unwrap(), want);
}
