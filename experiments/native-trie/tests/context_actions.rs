use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupLesson, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn grid(s: &Grids, id: &str, p: &Pixels, side: Side) -> Grid {
    let src = s
        .source(
            id.as_bytes(),
            b"declared native context-relation counterfactual fixture",
        )
        .unwrap();
    s.admit(id.as_bytes(), side, p, &src).unwrap()
}
fn teach(s: &Grids, m: &mut SparseRewrite, id: &str, a: &Pixels, b: &Pixels) -> GroupLesson {
    let i = grid(s, &format!("{id}i"), a, Side::TrainingInput);
    let o = grid(s, &format!("{id}o"), b, Side::TrainingOutput);
    m.derive_groups(id.as_bytes(), &i, &o).unwrap()
}
fn global_case(two: bool, query: bool) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(6); 14]; 10];
    let mut b = a.clone();
    let (y, x) = if query { (3, 6) } else { (2, 2) };
    for dy in 0..2 {
        for dx in 0..2 {
            a[y + dy][x + dx] = Some(9);
            b[y + dy][x + dx] = Some(if two { 7 } else { 3 });
        }
    }
    let marks = if query {
        vec![(7, 1), (7, 2), (7, 3), (8, 3), (7, 10), (8, 10), (8, 11)]
    } else if two {
        vec![(6, 1), (6, 2), (6, 10)]
    } else {
        vec![(6, 1), (6, 2), (6, 3)]
    };
    for (y, x) in marks {
        a[y][x] = Some(4);
        b[y][x] = Some(4);
    }
    (a, b)
}
#[test]
fn action_depends_on_distinct_components_of_another_colour_not_painted_pixels() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, b) = global_case(false, false);
    let d = teach(&s, &mut m, "a", &a, &b);
    let (a, b) = global_case(true, false);
    let e = teach(&s, &mut m, "b", &a, &b);
    let program = m.agree_context_actions(&[d, e]).unwrap().unwrap();
    assert_eq!(program.groups(), 2);
    let (a, want) = global_case(true, true);
    let q = grid(&s, "q", &a, Side::QueryInput);
    assert_eq!(
        m.apply_groups(&mut s, &q, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        want
    );
    let leaf = program
        .root
        .child(b"groups")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .follow()
        .unwrap();
    let role = leaf.child(b"context-role").unwrap().follow().unwrap();
    assert!(role.child(b"neighbour-kind").is_none());
    assert!(m.set_copies > 0);
}
fn local_case(two: bool, query: bool) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(6); 20]; 14];
    let mut b = a.clone();
    let dy = usize::from(query);
    let dx = usize::from(query);
    let mut paint = Vec::new();
    if two {
        for (y, x) in [(2 + dy, 1 + dx), (4 + dy, 3 + dx)] {
            for yy in 0..3 {
                for xx in 0..3 {
                    if yy == 0 || xx == 0 || yy == 2 || xx == 2 {
                        paint.push((y + yy, x + xx));
                    }
                }
            }
        }
        paint.push((1 + dy, 1 + dx));
        if query {
            paint.push((dy, 1 + dx));
        }
    } else {
        for yy in 0..4 {
            for xx in 0..6 {
                if yy == 0 || xx == 0 || yy == 3 || xx == 5 {
                    paint.push((2 + yy, 1 + xx));
                }
            }
        }
    }
    for (y, x) in paint {
        a[y][x] = Some(9);
        b[y][x] = Some(if two { 7 } else { 3 });
    }
    for yy in 0..3 {
        for xx in 0..3 {
            let draw = if !two || query {
                yy == 0 || xx == 0 || yy == 2 || xx == 2
            } else {
                !(yy == 2 && xx == 2)
            };
            if draw {
                a[9 + yy][13 + xx] = Some(4);
                b[9 + yy][13 + xx] = Some(4);
            }
        }
    }
    (a, b)
}
#[test]
fn learns_local_relation_degree_not_equal_pixel_size_or_global_hole_count() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (a, b) = local_case(false, false);
    let d = teach(&s, &mut m, "a", &a, &b);
    let (a, b) = local_case(true, false);
    let e = teach(&s, &mut m, "b", &a, &b);
    let program = m.agree_context_actions(&[d, e]).unwrap().unwrap();
    let (a, want) = local_case(true, true);
    let q = grid(&s, "q1", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &program).unwrap();
    assert_eq!(old.inspect().unwrap(), want);
    let leaf = program
        .root
        .child(b"groups")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .follow()
        .unwrap();
    let role = leaf.child(b"context-role").unwrap().follow().unwrap();
    assert!(role.child(b"neighbour-kind").is_some());
    let other = q
        .cell(9, 13)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    role.child(b"context-colour").unwrap().point_to(&other);
    let q2 = grid(&s, "q2", &a, Side::QueryInput);
    let changed = m
        .apply_groups(&mut s, &q2, &program)
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(changed[3][2], Some(9));
    assert_eq!(old.inspect().unwrap(), want);
}
