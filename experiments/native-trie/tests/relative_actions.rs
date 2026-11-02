use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn fixture(sizes: &[usize], source: u8, bg: u8, actions: &[u8], query: bool) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(bg); 24]; 24];
    let mut b = a.clone();
    for (j, n) in sizes.iter().enumerate() {
        let y = 2 + 4 * j + usize::from(query);
        for x in 0..*n {
            a[y][2 + x] = Some(source);
            b[y][2 + x] = Some(actions.get(j).copied().unwrap_or(source));
        }
    }
    if query {
        for dy in 0..3 {
            for dx in 0..3 {
                if dy == 0 || dy == 2 || dx == 0 || dx == 2 {
                    a[20 + dy][19 + dx] = Some(3);
                    b[20 + dy][19 + dx] = Some(3);
                }
            }
        }
    }
    (a, b)
}
fn grid(s: &Grids, id: &str, a: &Pixels, side: Side) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"declared native relative-count learning/counterfactual fixture",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn learn(s: &Grids, m: &mut SparseRewrite, source: u8, bg: u8, actions: &[u8]) -> GroupProgram {
    let mut lessons = Vec::new();
    for (i, sizes) in [[8usize, 5, 4, 2], [9, 8, 6, 3]].iter().enumerate() {
        let (a, b) = fixture(sizes, source, bg, actions, false);
        let input = grid(s, &format!("i{i}"), &a, Side::TrainingInput);
        let output = grid(s, &format!("o{i}"), &b, Side::TrainingOutput);
        lessons.push(
            m.derive_groups(format!("d{i}").as_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let p = m.agree_relative_actions(&lessons).unwrap().unwrap();
    assert_eq!(p.groups(), 4);
    for i in 0..p.groups() {
        assert!(
            p.root
                .child(b"groups")
                .unwrap()
                .child(&i.to_be_bytes())
                .unwrap()
                .follow()
                .unwrap()
                .child(b"relative-role")
                .is_some()
        );
    }
    p
}
#[test]
fn learns_opaque_actions_under_new_absolute_sizes_palette_topology_and_object_counts() {
    for (source, bg, actions) in [(9, 6, [7, 2, 8, 1]), (4, 0, [9, 7, 2, 6])] {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let p = learn(&s, &mut m, source, bg, &actions);
        for (i, sizes) in [vec![14, 10, 9, 6], vec![12, 8, 2], vec![14, 10, 9, 6, 3]]
            .iter()
            .enumerate()
        {
            let (a, b) = fixture(sizes, source, bg, &actions, true);
            let q = grid(&s, &format!("q{i}"), &a, Side::QueryInput);
            assert_eq!(
                m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap(),
                b
            );
        }
        assert!(m.relative_statistics().0 > 0);
    }
}
#[test]
fn actual_role_expected_rank_and_literal_binding_control_execution_without_mutating_prior_output() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = learn(&s, &mut m, 9, 6, &[7, 2, 8, 1]);
    let (a, want) = fixture(&[14, 10, 9, 6], 9, 6, &[7, 2, 8, 1], true);
    let q = grid(&s, "old", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_eq!(old.inspect().unwrap(), want);
    let leaf = p
        .root
        .child(b"groups")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .follow()
        .unwrap();
    let role = leaf.child(b"relative-role").unwrap().follow().unwrap();
    let original = role.child(b"context-colour").unwrap().follow().unwrap();
    role.child(b"context-colour").unwrap().point_to(
        &q.cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    let q2 = grid(&s, "role", &a, Side::QueryInput);
    assert_ne!(
        m.apply_groups(&mut s, &q2, &p).unwrap().inspect().unwrap(),
        want
    );
    role.child(b"context-colour").unwrap().point_to(&original);
    let slot = leaf
        .child(b"literal-action-slot")
        .unwrap()
        .follow()
        .unwrap();
    slot.point_to(&original);
    let q3 = grid(&s, "action", &a, Side::QueryInput);
    assert_ne!(
        m.apply_groups(&mut s, &q3, &p).unwrap().inspect().unwrap(),
        want
    );
    let mut impossible = m
        .root
        .child(b"native-components")
        .unwrap()
        .child(b"cardinalities")
        .unwrap()
        .child(b"zero")
        .unwrap();
    for _ in 0..900 {
        impossible = impossible.child(b"successor").unwrap();
    }
    for i in 0..p.groups() {
        let leaf = p
            .root
            .child(b"groups")
            .unwrap()
            .child(&i.to_be_bytes())
            .unwrap()
            .follow()
            .unwrap();
        leaf.child(b"expected-relative-count")
            .unwrap()
            .point_to(&impossible);
    }
    let q4 = grid(&s, "expected", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q4, &p).unwrap().inspect().unwrap(),
        a
    );
    assert_eq!(old.inspect().unwrap(), want);
}
#[test]
fn unknown_universe_or_frontier_prevents_relative_actions() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = learn(&s, &mut m, 9, 6, &[7, 2, 8, 1]);
    let (mut a, _) = fixture(&[14, 10, 9, 6], 9, 6, &[7, 2, 8, 1], true);
    a[3][1] = None;
    let q = grid(&s, "unknown", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap(),
        a
    );
}
#[test]
fn equal_size_components_keep_equal_relative_keys_when_labels_agree() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut lessons = Vec::new();
    for (i, sizes) in [[8usize, 8, 4, 2], [9, 9, 8, 3]].iter().enumerate() {
        let (a, b) = fixture(sizes, 9, 6, &[7, 7, 8, 1], false);
        let input = grid(&s, &format!("i{i}"), &a, Side::TrainingInput);
        let output = grid(&s, &format!("o{i}"), &b, Side::TrainingOutput);
        lessons.push(
            m.derive_groups(format!("d{i}").as_bytes(), &input, &output)
                .unwrap(),
        );
    }
    let p = m.agree_relative_actions(&lessons).unwrap().unwrap();
    assert_eq!(p.groups(), 3);
    let (a, b) = fixture(&[14, 14, 9, 6], 9, 6, &[7, 7, 8, 1], true);
    let q = grid(&s, "q", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap(),
        b
    );
}
#[test]
fn equal_relative_key_with_conflicting_labels_is_rejected() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, b) = fixture(&[4, 4], 9, 6, &[7, 2], false);
    let i = grid(&s, "i", &a, Side::TrainingInput);
    let o = grid(&s, "o", &b, Side::TrainingOutput);
    let d = m.derive_groups(b"d", &i, &o).unwrap();
    assert!(m.agree_relative_actions(&[d]).unwrap().is_none());
}
