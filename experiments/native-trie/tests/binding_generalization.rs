use native_trie_seed::{
    Address,
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupLesson, GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn pixels(rows: &[&str]) -> Pixels {
    rows.iter()
        .map(|r| {
            r.bytes()
                .map(|b| {
                    Some(match b {
                        b'0' => 6,
                        b'3' => 9,
                        _ => b - b'0',
                    })
                })
                .collect()
        })
        .collect()
}
fn grid(s: &Grids, id: &str, a: &Pixels, side: Side) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"palette6/literal9 counterfactual derived from archived small-object controls",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn lesson(s: &Grids, m: &mut SparseRewrite, id: &str, a: Pixels, b: Pixels) -> GroupLesson {
    let i = grid(s, &format!("{id}i"), &a, Side::TrainingInput);
    let o = grid(s, &format!("{id}o"), &b, Side::TrainingOutput);
    m.derive_groups(id.as_bytes(), &i, &o).unwrap()
}
fn lessons(s: &Grids, m: &mut SparseRewrite) -> Vec<GroupLesson> {
    vec![
        lesson(
            s,
            m,
            "a",
            pixels(&["17710805", "17711010", "88007777", "01000011", "50801011"]),
            pixels(&["37710303", "37711030", "33007777", "03000011", "30303011"]),
        ),
        lesson(
            s,
            m,
            "b",
            pixels(&[
                "00181110", "15171100", "08077788", "08800080", "07008550", "10000001", "10877800",
                "00877088", "08808088",
            ]),
            pixels(&[
                "00331110", "33371100", "08077788", "08800080", "03003330", "30000003", "30877300",
                "00877088", "08803088",
            ]),
        ),
        lesson(
            s,
            m,
            "c",
            pixels(&["105", "100", "777"]),
            pixels(&["303", "300", "777"]),
        ),
        lesson(
            s,
            m,
            "d",
            pixels(&[
                "00107770", "88005500", "08800550", "01108001", "07018000", "80001070", "08881000",
            ]),
            pixels(&[
                "00307770", "88005500", "08800550", "03303003", "03033000", "30003030", "08883000",
            ]),
        ),
    ]
}
fn query() -> (Pixels, Pixels) {
    (
        pixels(&[
            "05015505", "11000110", "07700005", "11050100", "01055501", "07007007", "10100017",
            "00110107",
        ]),
        pixels(&[
            "03033303", "33000330", "03300003", "11050300", "01055503", "03003007", "30100037",
            "00110307",
        ]),
    )
}
fn leafs(p: &GroupProgram) -> Vec<Address> {
    (0..p.groups())
        .map(|i| {
            p.root
                .child(b"groups")
                .unwrap()
                .child(&i.to_be_bytes())
                .unwrap()
                .follow()
                .unwrap()
        })
        .collect()
}
#[test]
fn merge_learns_excluded_colour6_and_new_colour7_count2_action9() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let ds = lessons(&s, &mut m);
    let old = m.agree_partitions(&ds).unwrap().unwrap();
    let p = m.generalize_source_bindings(&ds, old).unwrap();
    let (a, want) = query();
    let q = grid(&s, "q", &a, Side::QueryInput);
    let output = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_eq!(output.inspect().unwrap(), want);
    let variable: Vec<_> = leafs(&p)
        .into_iter()
        .filter(|g| g.child(b"source-colour").is_none())
        .collect();
    assert!(!variable.is_empty());
    let six = q
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    for g in variable {
        assert!(
            g.child(b"excluded-source-colours")
                .unwrap()
                .child(&0usize.to_be_bytes())
                .unwrap()
                .follow()
                .unwrap()
                .same(&six)
        );
    }
}
#[test]
fn stored_exclusion_is_runtime_authority_and_history_is_immutable() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let ds = lessons(&s, &mut m);
    let old = m.agree_partitions(&ds).unwrap().unwrap();
    let p = m.generalize_source_bindings(&ds, old).unwrap();
    let (a, want) = query();
    let q = grid(&s, "q1", &a, Side::QueryInput);
    let previous = m.apply_groups(&mut s, &q, &p).unwrap();
    let seven = q
        .cell(2, 1)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    for g in leafs(&p) {
        if let Some(ex) = g.child(b"excluded-source-colours") {
            ex.child(&0usize.to_be_bytes()).unwrap().point_to(&seven);
        }
    }
    let q = grid(&s, "q2", &a, Side::QueryInput);
    let output = m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap();
    assert_eq!(output[2][1], Some(7));
    assert_eq!(output[2][2], Some(7));
    assert_eq!(previous.inspect().unwrap(), want);
}
#[test]
fn unsafe_kind_dropping_is_rejected_instead_of_ignoring_same_colour_negative() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut a = vec![vec![Some(6); 11]; 7];
    for (y, x, c) in [
        (0, 0, 1),
        (0, 1, 1),
        (6, 0, 1),
        (6, 1, 1),
        (6, 2, 1),
        (0, 5, 8),
        (0, 6, 8),
        (6, 7, 8),
        (6, 8, 8),
        (6, 9, 8),
        (3, 5, 8),
        (3, 6, 8),
    ] {
        a[y][x] = Some(c);
    }
    let mut b = a.clone();
    for (y, x) in [(0, 0), (0, 1), (0, 5), (0, 6)] {
        b[y][x] = Some(9);
    }
    let d = lesson(&s, &mut m, "a", a, b);
    let ds = [d];
    let old = m.agree_partitions(&ds).unwrap().unwrap();
    let identity = old.root.clone();
    let p = m.generalize_source_bindings(&ds, old).unwrap();
    assert!(p.root.same(&identity));
    assert!(
        leafs(&p)
            .iter()
            .all(|g| g.child(b"source-colour").is_some())
    );
}
#[test]
fn one_source_binding_is_not_invented_into_a_merge() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let a = vec![
        vec![Some(6), Some(1), Some(1), Some(6)],
        vec![Some(6); 4],
        vec![Some(1), Some(1), Some(1), Some(6)],
    ];
    let mut b = a.clone();
    b[0][1] = Some(9);
    b[0][2] = Some(9);
    let ds = [lesson(&s, &mut m, "a", a, b)];
    let old = m.agree_partitions(&ds).unwrap().unwrap();
    let identity = old.root.clone();
    let p = m.generalize_source_bindings(&ds, old).unwrap();
    assert!(p.root.same(&identity));
}
