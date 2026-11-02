use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
const POS: [(usize, usize); 4] = [(2, 2), (2, 4), (3, 2), (3, 4)];
fn fixture(
    palette: [u8; 4],
    query: bool,
    extra_unchanged: bool,
    duplicate_target: bool,
) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(0); 12]; 12];
    let mut b = a.clone();
    for (j, (y, x)) in POS.iter().enumerate() {
        a[*y][*x] = Some(palette[j]);
        b[*y][*x] = Some(palette[j]);
    }
    for j in 0..4 {
        let y = 6 + j;
        let n = if query { j + 1 } else { 1 };
        for dx in 0..n {
            a[y][2 + dx] = Some(palette[j]);
            b[y][2 + dx] = Some(palette[(j + 1) % 4]);
        }
    }
    if extra_unchanged {
        a[6][9] = Some(palette[0]);
        b[6][9] = Some(palette[0]);
    }
    if duplicate_target {
        a[4][4] = Some(palette[1]);
        b[4][4] = Some(palette[1]);
    }
    (a, b)
}
fn grid(s: &Grids, id: &str, a: &Pixels, side: Side) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"declared non-corner directed input-role cycle fixture",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn learn(s: &Grids, m: &mut SparseRewrite, extra: bool, dup: bool) -> Option<GroupProgram> {
    let mut lessons = Vec::new();
    for (i, palette) in [[1, 3, 2, 8], [4, 2, 3, 7], [9, 4, 7, 6]]
        .iter()
        .enumerate()
    {
        let (a, b) = fixture(*palette, false, extra, dup);
        let input = grid(s, &format!("i{i}"), &a, Side::TrainingInput);
        let output = grid(s, &format!("o{i}"), &b, Side::TrainingOutput);
        lessons.push(
            m.derive_groups(format!("d{i}").as_bytes(), &input, &output)
                .unwrap(),
        );
    }
    m.agree_source_pair_actions(&lessons).unwrap()
}
#[test]
fn learns_non_corner_general_directed_cycle_and_reads_new_input_action_slots() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = learn(&s, &mut m, false, false).unwrap();
    assert!(p.root.child(b"source-pair-program").is_some());
    assert_eq!(p.groups(), 4);
    let (a, b) = fixture([8, 9, 2, 4], true, false, false);
    let q = grid(&s, "q", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap(),
        b
    );
}
#[test]
fn unchanged_data_outside_own_two_roles_is_mandatory() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = learn(&s, &mut m, true, false);
    assert!(p.is_none_or(|p| p.root.child(b"source-pair-program").is_none()));
}
#[test]
fn equal_payload_distinct_input_target_slots_are_ambiguous_not_substituted() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = learn(&s, &mut m, false, true);
    assert!(p.is_none_or(|p| p.root.child(b"source-pair-program").is_none()));
}
#[test]
fn two_independently_valid_equal_payload_targets_do_not_choose_a_programme_subset() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut a = vec![vec![Some(0); 12]; 12];
    a[2][2] = Some(1);
    a[2][4] = Some(3);
    a[4][4] = Some(3);
    a[6][2] = Some(1);
    let mut b = a.clone();
    b[6][2] = Some(3);
    let i = grid(&s, "i", &a, Side::TrainingInput);
    let o = grid(&s, "o", &b, Side::TrainingOutput);
    let d = m.derive_groups(b"d", &i, &o).unwrap();
    assert!(m.agree_source_pair_actions(&[d]).unwrap().is_none());
}
#[test]
fn actual_stored_target_role_mutation_and_missing_unknown_roles() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = learn(&s, &mut m, false, false).unwrap();
    let (a, want) = fixture([8, 9, 2, 4], true, false, false);
    let q = grid(&s, "old", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &p).unwrap();
    let rule = p
        .root
        .child(b"groups")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .follow()
        .unwrap();
    let h = rule.child(b"head").unwrap();
    let t = rule.child(b"target").unwrap();
    t.child(b"column-key")
        .unwrap()
        .point_to(&h.child(b"column-key").unwrap().follow().unwrap());
    let q2 = grid(&s, "new", &a, Side::QueryInput);
    assert_ne!(
        m.apply_groups(&mut s, &q2, &p).unwrap().inspect().unwrap(),
        want
    );
    assert_eq!(old.inspect().unwrap(), want);
    let small = grid(&s, "small", &vec![vec![Some(0)]], Side::QueryInput);
    assert!(!m.source_pair_applicable(&small, &p).unwrap());
    let mut unknown = a;
    unknown[2][2] = None;
    let q3 = grid(&s, "unknown", &unknown, Side::QueryInput);
    assert!(!m.source_pair_applicable(&q3, &p).unwrap());
}
