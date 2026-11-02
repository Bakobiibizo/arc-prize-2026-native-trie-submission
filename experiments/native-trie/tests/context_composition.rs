use native_trie_seed::{
    Address,
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn fixture(
    size: usize,
    markers: usize,
    query: bool,
    palette: [u8; 4],
    two_sources: bool,
) -> (Pixels, Pixels) {
    let [bg, subject, marker, target] = palette;
    let mut a = vec![vec![Some(bg); 14]; 10];
    let mut b = a.clone();
    let (y, x) = if query { (4, 6) } else { (2, 2) };
    for i in 0..size {
        a[y][x + i] = Some(subject);
        b[y][x + i] = Some(if size == 2 && markers == 2 {
            target
        } else {
            subject
        });
    }
    if two_sources {
        for i in 0..size {
            a[y + 2][x + i] = Some(8);
            b[y + 2][x + i] = Some(if size == 2 && markers == 2 { target } else { 8 });
        }
    }
    for k in 0..markers {
        let len = if query { 3 } else { 1 };
        for i in 0..len {
            a[8][1 + k * 8 + i] = Some(marker);
            b[8][1 + k * 8 + i] = Some(marker);
        }
    }
    if query {
        for dy in 0..3 {
            for dx in 0..3 {
                if dy == 0 || dy == 2 || dx == 0 || dx == 2 {
                    a[1 + dy][10 + dx] = Some(3);
                    b[1 + dy][10 + dx] = Some(3);
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
            b"Declared independent factorial native input-count conjunction fixture",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, a, &src).unwrap()
}
fn learn(s: &Grids, m: &mut SparseRewrite, palette: [u8; 4], two: bool) -> GroupProgram {
    let mut lessons = Vec::new();
    for size in 1..=2 {
        for markers in 1..=2 {
            let (a, b) = fixture(size, markers, false, palette, two);
            let id = format!("s{size}m{markers}");
            let i = grid(s, &format!("{id}i"), &a, Side::TrainingInput);
            let o = grid(s, &format!("{id}o"), &b, Side::TrainingOutput);
            lessons.push(m.derive_groups(id.as_bytes(), &i, &o).unwrap());
        }
    }
    assert!(m.agree_context_actions(&lessons).unwrap().is_none());
    m.agree_context_composition(&lessons).unwrap().unwrap()
}
fn leaf(p: &GroupProgram) -> Address {
    p.root
        .child(b"groups")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .follow()
        .unwrap()
}
#[test]
fn independent_factorial_requires_both_actual_native_fields() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let palette = [6, 9, 4, 7];
    let p = learn(&s, &mut m, palette, false);
    assert_eq!(p.groups(), 1);
    for (size, markers) in [(1, 1), (1, 2), (2, 1), (2, 2), (3, 2)] {
        let (a, b) = fixture(size, markers, true, palette, false);
        let q = grid(&s, &format!("q{size}{markers}"), &a, Side::QueryInput);
        assert_eq!(
            m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap(),
            b
        );
    }
    let (a, want) = fixture(2, 2, true, palette, false);
    let q = grid(&s, "old", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &p).unwrap();
    let g = leaf(&p);
    let role = g.child(b"context-role").unwrap().follow().unwrap();
    role.child(b"context-colour").unwrap().point_to(
        &q.cell(4, 6)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    let q2 = grid(&s, "changed-role", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q2, &p).unwrap().inspect().unwrap()[4][6],
        Some(9)
    );
    assert_eq!(old.inspect().unwrap(), want);
}
#[test]
fn base_count_is_not_a_shadow_or_ignored_by_context() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = learn(&s, &mut m, [6, 9, 4, 7], false);
    let (a, want) = fixture(2, 2, true, [6, 9, 4, 7], false);
    let q = grid(&s, "old", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &p).unwrap();
    let one = m
        .root
        .child(b"native-components")
        .unwrap()
        .child(b"cardinalities")
        .unwrap()
        .child(b"zero")
        .unwrap()
        .child(b"successor")
        .unwrap();
    leaf(&p).child(b"expected-count").unwrap().point_to(&one);
    let q = grid(&s, "new", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap()[4][6],
        Some(9)
    );
    assert_eq!(old.inspect().unwrap(), want);
}
#[test]
fn palette_relearning_and_context_guards_survive_binding_generalization() {
    for (palette, two) in [([0, 5, 2, 7], false), ([6, 9, 4, 7], true)] {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let p = learn(&s, &mut m, palette, two);
        assert_eq!(p.groups(), if two { 2 } else { 1 });
        for markers in 1..=2 {
            let (a, b) = fixture(2, markers, true, palette, two);
            let q = grid(&s, &format!("q{markers}"), &a, Side::QueryInput);
            assert_eq!(
                m.apply_groups(&mut s, &q, &p).unwrap().inspect().unwrap(),
                b
            );
        }
    }
}
