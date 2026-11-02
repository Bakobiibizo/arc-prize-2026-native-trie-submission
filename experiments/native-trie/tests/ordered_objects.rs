use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn grid(s: &Grids, id: &str, a: &Pixels, side: Side) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"declared bottom panels/ordered-corner copy teaching fixture",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn fixture(items: &[(u8, usize, usize)], h: usize, w: usize, reverse: bool) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(7); w]; h];
    let mut x = 1;
    for &(v, hh, ww) in items {
        for row in a.iter_mut().skip(h - hh) {
            for cell in row.iter_mut().skip(x).take(ww) {
                *cell = Some(v);
            }
        }
        x += ww + 1;
    }
    let mut b = vec![vec![Some(7); w]; h];
    let mut order = items.to_vec();
    if reverse {
        order.reverse()
    }
    let (mut y, mut x) = (0, 0);
    for (v, hh, ww) in order {
        for row in b.iter_mut().skip(y).take(hh) {
            for cell in row.iter_mut().skip(x).take(ww) {
                *cell = Some(v);
            }
        }
        y += hh - 1;
        x += ww - 1;
    }
    (a, b)
}
fn teach(s: &Grids, m: &mut SparseRewrite, reverse: bool, bad: bool) -> Option<GroupProgram> {
    let mut lessons = Vec::new();
    for (i, items) in [
        vec![(5, 2, 2), (4, 3, 2)],
        vec![(9, 3, 3), (6, 2, 2), (2, 2, 1)],
    ]
    .iter()
    .enumerate()
    {
        let (a, mut b) = fixture(items, 9, 12, reverse);
        if bad && i == 1 {
            b[8][1] = a[8][1];
        }
        let input = grid(s, &format!("i{i}"), &a, Side::TrainingInput);
        let output = grid(s, &format!("o{i}"), &b, Side::TrainingOutput);
        lessons.push(m.derive_groups(&i.to_be_bytes(), &input, &output).unwrap());
    }
    m.agree_ordered_objects(&lessons).unwrap()
}
#[test]
fn learns_native_universe_order_and_corner_copy_from_all_labels() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = teach(&s, &mut m, false, false).unwrap();
    assert!(p.root.child(b"ordered-object-program").is_some());
    assert_eq!(p.groups(), 2);
    let (a, b) = fixture(&[(1, 4, 3), (8, 2, 2), (1, 3, 2)], 13, 15, false);
    let q = grid(&s, "q", &a, Side::QueryInput);
    assert!(m.source_pair_applicable(&q, &p).unwrap());
    let old = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_eq!(old.inspect().unwrap(), b);
    let nw = p.root.child(b"initial-corner").unwrap().follow().unwrap();
    p.root.child(b"next-corner").unwrap().point_to(&nw);
    let altered = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_ne!(altered.inspect().unwrap(), b);
    assert_eq!(old.inspect().unwrap(), b);
    assert!(m.workspace_bytes() < 286720000);
}
#[test]
fn source_order_is_learned_not_always_left_to_right() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = teach(&s, &mut m, true, false).unwrap();
    assert!(p.root.child(b"ordered-object-program").is_some());
    assert_eq!(
        p.root
            .child(b"source-order")
            .unwrap()
            .follow()
            .unwrap()
            .token(),
        Some(b"west".as_slice())
    );
    let (a, b) = fixture(&[(1, 4, 3), (8, 2, 2), (3, 3, 2)], 13, 15, true);
    let q = grid(&s, "q", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap(),
        b
    );
}
#[test]
fn unavailable_off_bottom_holes_and_foreign_owners_are_not_empty_answers() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = teach(&s, &mut m, false, false).unwrap();
    let mut foreign = SparseRewrite::new(&s);
    let (a, _) = fixture(&[(1, 3, 3)], 9, 12, false);
    let valid = grid(&s, "valid", &a, Side::QueryInput);
    assert!(foreign.source_pair_applicable(&valid, &p).is_err());
    for case in 0..3 {
        let mut bad = a.clone();
        match case {
            0 => bad[0][0] = None,
            1 => {
                bad.rotate_left(1);
                bad[8].fill(Some(7));
            }
            _ => bad[7][2] = Some(7),
        }
        let q = grid(&s, &format!("bad{case}"), &bad, Side::QueryInput);
        assert!(!m.source_pair_applicable(&q, &p).unwrap());
    }
    let q = grid(&s, "known", &a, Side::QueryInput);
    let out = m.apply_groups(&mut s, &q, &p).unwrap();
    assert!(
        out.inspect()
            .unwrap()
            .iter()
            .flatten()
            .any(|v| *v == Some(1))
    );
}
#[test]
fn unchanged_cell_contradiction_cannot_be_ignored() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    assert!(
        teach(&s, &mut m, false, true)
            .is_none_or(|p| p.root.child(b"ordered-object-program").is_none())
    );
}
