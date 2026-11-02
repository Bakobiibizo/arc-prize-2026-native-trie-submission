use native_trie_seed::{
    Address,
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupLesson, GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn p(a: &[&[u8]]) -> Pixels {
    a.iter()
        .map(|r| r.iter().map(|v| Some(*v)).collect())
        .collect()
}
fn grid(s: &Grids, id: &str, a: &Pixels, side: Side) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"source group control declared training/query data",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn teach(s: &Grids, m: &mut SparseRewrite, id: &str, a: &Pixels, b: &Pixels) -> GroupLesson {
    let i = grid(s, &format!("{id}i"), a, Side::TrainingInput);
    let o = grid(s, &format!("{id}o"), b, Side::TrainingOutput);
    m.derive_groups(id.as_bytes(), &i, &o).unwrap()
}
fn swap_setup(s: &Grids, m: &mut SparseRewrite) -> GroupProgram {
    let a = teach(
        s,
        m,
        "a",
        &p(&[&[2, 0, 5, 8], &[5, 2, 0, 8]]),
        &p(&[&[5, 0, 2, 8], &[2, 5, 0, 8]]),
    );
    let b = teach(
        s,
        m,
        "b",
        &p(&[&[0, 2, 8], &[5, 8, 2], &[8, 5, 0]]),
        &p(&[&[0, 5, 8], &[2, 8, 5], &[8, 2, 0]]),
    );
    m.agree_groups(&[a, b]).unwrap().unwrap()
}
fn member(program: &GroupProgram, source: &Address) -> Address {
    (0..program.groups())
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
            g.child(b"source-colour")
                .unwrap()
                .follow()
                .unwrap()
                .same(source)
        })
        .unwrap()
}
#[test]
fn independently_taught_two_action_swap_reads_original_input() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let program = swap_setup(&s, &mut m);
    assert_eq!(program.groups(), 2);
    let q = grid(
        &s,
        "q",
        &p(&[&[5, 2, 5, 0, 8], &[2, 5, 2, 8, 0]]),
        Side::QueryInput,
    );
    assert_eq!(
        m.apply_groups(&mut s, &q, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        p(&[&[2, 5, 2, 0, 8], &[5, 2, 5, 8, 0]])
    );
    assert_eq!(m.region_proofs, 0);
    assert!(m.workspace_bytes() < 286720000);
}
fn ring(h: usize, w: usize, c: u8) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(0); w]; h];
    let mut b = a.clone();
    for y in 0..h {
        for x in 0..w {
            let rim = (y == 1 || y == h - 2) && (1..w - 1).contains(&x)
                || (x == 1 || x == w - 2) && (1..h - 1).contains(&y);
            if rim {
                a[y][x] = Some(c);
                b[y][x] = Some(c);
            } else if (2..h - 2).contains(&y) && (2..w - 2).contains(&x) {
                b[y][x] = Some(4);
            }
        }
    }
    a[0][0] = Some(2);
    b[0][0] = Some(5);
    (a, b)
}
#[test]
fn mixed_source_identity_and_native_kind_condition() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (i, o) = ring(5, 7, 8);
    let a = teach(&s, &mut m, "a", &i, &o);
    let (i, o) = ring(7, 9, 9);
    let b = teach(&s, &mut m, "b", &i, &o);
    let program = m.agree_groups(&[a, b]).unwrap().unwrap();
    assert_eq!(program.groups(), 2);
    let (i, want) = ring(9, 11, 7);
    let q = grid(&s, "q", &i, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        want
    );
    assert!(m.region_copies > 0 && m.region_cache_hits > 0);
}
#[test]
fn absent_source_is_not_present_unchanged() {
    for present in [false, true] {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let a = teach(&s, &mut m, "a", &p(&[&[2, 0]]), &p(&[&[5, 0]]));
        let other = p(&[&[if present { 2 } else { 3 }, 0]]);
        let b = teach(&s, &mut m, "b", &other, &other);
        let result = m.agree_groups(&[a, b]).unwrap();
        if present {
            assert!(result.is_none());
        } else {
            let q = grid(&s, "q", &p(&[&[2, 3, 0]]), Side::QueryInput);
            assert_eq!(
                m.apply_groups(&mut s, &q, &result.unwrap())
                    .unwrap()
                    .inspect()
                    .unwrap(),
                p(&[&[5, 3, 0]])
            );
        }
    }
}
#[test]
fn target_conflicts_within_and_across_demos_reject() {
    for within in [false, true] {
        let s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let a = teach(
            &s,
            &mut m,
            "a",
            &p(&[&[2, 2]]),
            &p(&[&[5, if within { 4 } else { 5 }]]),
        );
        let b = teach(&s, &mut m, "b", &p(&[&[2]]), &p(&[&[4]]));
        assert!(m.agree_groups(&[a, b]).unwrap().is_none());
    }
}
#[test]
fn stored_source_and_action_fields_control_execution_and_preserve_history() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let program = swap_setup(&s, &mut m);
    let input = p(&[&[2, 5, 8, 0]]);
    let q = grid(&s, "q1", &input, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &program).unwrap();
    let two = q
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let g = member(&program, &two);
    g.child(b"source-colour").unwrap().point_to(
        &q.cell(0, 2)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    g.child(b"literal-action-slot")
        .unwrap()
        .follow()
        .unwrap()
        .point_to(
            &q.cell(0, 3)
                .unwrap()
                .child(b"value")
                .unwrap()
                .follow()
                .unwrap(),
        );
    let q2 = grid(&s, "q2", &input, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q2, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        p(&[&[2, 2, 0, 0]])
    );
    assert_eq!(old.inspect().unwrap(), p(&[&[5, 2, 8, 0]]));
}
#[test]
fn identity_has_evidence_but_no_lessons_does_not() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    assert!(m.agree_groups(&[]).is_err());
    let a = p(&[&[2, 0, 5]]);
    let d = teach(&s, &mut m, "a", &a, &a);
    let program = m.agree_groups(&[d]).unwrap().unwrap();
    assert_eq!(program.groups(), 0);
    let q = grid(&s, "q", &p(&[&[8, 9]]), Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        p(&[&[8, 9]])
    );
}
#[test]
fn unavailable_positive_kind_does_not_count_as_known() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (mut i, mut o) = ring(7, 9, 8);
    i[3][3] = None;
    o[3][3] = None;
    let d = teach(&s, &mut m, "a", &i, &o);
    assert!(m.agree_groups(&[d]).unwrap().is_none());
}
#[test]
fn simultaneously_selected_distinct_action_slots_do_not_fallback() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let program = swap_setup(&s, &mut m);
    let q = grid(&s, "q", &p(&[&[2, 5]]), Side::QueryInput);
    let two = q
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let five = q
        .cell(0, 1)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    member(&program, &five)
        .child(b"source-colour")
        .unwrap()
        .point_to(&two);
    assert!(m.apply_groups(&mut s, &q, &program).is_err());
}
