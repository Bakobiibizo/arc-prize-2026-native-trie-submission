use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::SparseRewrite,
};
type Pixels = Vec<Vec<Option<u8>>>;
fn grid(s: &Grids, id: &str, p: &Pixels) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"cyclic prototype declared fixture, not solver input",
        )
        .unwrap();
    s.admit(id.as_bytes(), Side::TrainingInput, p, &src)
        .unwrap()
}
fn text(rows: &[&str]) -> Pixels {
    rows.iter()
        .map(|r| r.bytes().map(|b| Some(b - b'0')).collect())
        .collect()
}
#[test]
fn supplied_cycle_native_output_fits_all_052_training_cells() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let cases = [
        (
            [
                "0000000", "0000000", "0000001", "0000012", "0000124", "0001240", "0012400",
            ],
            [
                "2412412", "4124124", "1241241", "2412412", "4124124", "1241241", "2412412",
            ],
        ),
        (
            [
                "2830000", "8300000", "3000000", "0000000", "0000000", "0000000", "0000000",
            ],
            [
                "2832832", "8328328", "3283283", "2832832", "8328328", "3283283", "2832832",
            ],
        ),
        (
            [
                "0000830", "0008300", "0083000", "0830004", "8300040", "3000400", "0004000",
            ],
            [
                "4834834", "8348348", "3483483", "4834834", "8348348", "3483483", "4834834",
            ],
        ),
    ];
    for (i, (a, b)) in cases.iter().enumerate() {
        let input = grid(&s, &format!("train{i}"), &text(a));
        let bg = input
            .cell(6, 6)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap();
        let p = m.cyclic_program(&bg, 3).unwrap();
        let output = m.cyclic_lattice(&mut s, &input, &p).unwrap();
        assert_eq!(output.inspect().unwrap(), text(b));
    }
    assert!(m.workspace_bytes() < 286720000);
}
#[test]
fn different_supplied_lengths_palette_geometry_and_lifetime() {
    for n in [2, 3, 4] {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let palette = [8, 1, 3, 6];
        let mut a = vec![vec![Some(7); 9]; 8];
        for (i, v) in palette.iter().take(n).enumerate() {
            a[0][i] = Some(*v);
        }
        let input = grid(&s, "source", &a);
        let bg = input
            .cell(7, 8)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap();
        let p = m.cyclic_program(&bg, n).unwrap();
        let phases = p.root.child(b"phases").unwrap();
        let shared = p.root.provision(b"shared-payload");
        for i in 0..n {
            phases.child(&i.to_be_bytes()).unwrap().point_to(&shared);
        }
        let out = m.cyclic_lattice(&mut s, &input, &p).unwrap();
        let expected: Pixels = (0..8)
            .map(|y| (0..9).map(|x| Some(palette[(x + y) % n])).collect())
            .collect();
        assert_eq!(out.inspect().unwrap(), expected);
        drop(input);
        drop(p);
        drop(m);
        drop(s);
        assert_eq!(out.inspect().unwrap(), expected);
    }
}
#[test]
fn unknown_missing_conflicting_all_background_and_foreign_cannot_commit() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut base = vec![vec![Some(0); 7]; 7];
    base[0][0] = Some(2);
    base[0][1] = Some(8);
    base[0][2] = Some(3);
    let input = grid(&s, "known", &base);
    let bg = input
        .cell(6, 6)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let p = m.cyclic_program(&bg, 3).unwrap();
    let mut foreign = SparseRewrite::new(&s);
    assert!(foreign.cyclic_lattice(&mut s, &input, &p).is_err());
    for case in 0..4 {
        let mut a = base.clone();
        match case {
            0 => a[6][6] = None,
            1 => {
                a[0][1] = Some(0);
                a[0][2] = Some(0);
            }
            2 => a[1][0] = Some(3),
            _ => a.iter_mut().for_each(|r| r.fill(Some(0))),
        }
        let bad = grid(&s, &format!("bad{case}"), &a);
        assert!(m.cyclic_lattice(&mut s, &bad, &p).is_err());
    }
}
#[test]
fn actual_successor_fields_control_phase_output_and_parent_contradictions() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let a = text(&[
        "2830000", "0000000", "0000000", "0000000", "0000000", "0000000", "0000000",
    ]);
    let input = grid(&s, "source", &a);
    let bg = input
        .cell(6, 6)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let p = m.cyclic_program(&bg, 3).unwrap();
    let old = m.cyclic_lattice(&mut s, &input, &p).unwrap();
    let expected = old.inspect().unwrap();
    let phases = p.root.child(b"phases").unwrap();
    let zero = phases.child(&0usize.to_be_bytes()).unwrap();
    zero.child(b"south-next").unwrap().point_to(&zero);
    assert!(m.cyclic_lattice(&mut s, &input, &p).is_err());
    assert_eq!(old.inspect().unwrap(), expected);
}
