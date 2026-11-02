use super::*;
use crate::arc_grids::Side;
fn grid(s: &Grids, id: &[u8], a: &[Vec<Option<u8>>]) -> Grid {
    let src = s
        .source(id, b"declared native component-context fixture")
        .unwrap();
    s.admit(id, Side::TrainingInput, a, &src).unwrap()
}
fn ordinal(m: &SparseRewrite, n: usize) -> Address {
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
fn grid_context_counts_components_not_pixels_and_reads_current_colour_binding() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut a = vec![vec![Some(6); 5]; 3];
    a[0][0] = Some(4);
    a[2][3] = Some(4);
    a[2][4] = Some(4);
    let input = grid(&s, b"a", &a);
    let role = m.root.provision(b"test-role");
    role.provision(b"context-colour").point_to(
        &input
            .cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    let (p, two) = m
        .context_value(&input, &input.cell(0, 0).unwrap(), &role)
        .unwrap();
    assert!(two.same(&ordinal(&m, 2)));
    let copies = m.set_copies;
    let (again, _) = m
        .context_value(&input, &input.cell(1, 1).unwrap(), &role)
        .unwrap();
    assert!(again.same(&p));
    assert_eq!(m.set_copies, copies);
    role.child(b"context-colour").unwrap().point_to(
        &input
            .cell(1, 1)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    let (_, one) = m
        .context_value(&input, &input.cell(0, 0).unwrap(), &role)
        .unwrap();
    assert!(one.same(&ordinal(&m, 1)));
    assert!(p.child(b"value").unwrap().follow().unwrap().same(&two));
}
#[test]
fn unknown_input_prevents_complete_global_component_count() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let input = grid(&s, b"a", &[vec![Some(4), None, Some(4)]]);
    let role = m.root.provision(b"role");
    role.provision(b"context-colour").point_to(
        &input
            .cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    let (_, value) = m
        .context_value(&input, &input.cell(0, 0).unwrap(), &role)
        .unwrap();
    assert_eq!(value.token(), Some(b"unavailable".as_slice()));
}
#[test]
fn equal_pixel_size_different_adjacent_component_degree_and_kind_param_authority() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut a = vec![vec![Some(6); 9]; 8];
    for dy in 0..4 {
        for dx in 0..6 {
            if dy == 0 || dy == 3 || dx == 0 || dx == 5 {
                a[dy + 2][dx + 1] = Some(4);
            }
        }
    }
    let mut b = vec![vec![Some(6); 9]; 8];
    for (y, x) in [(1, 1), (3, 3)] {
        for dy in 0..3 {
            for dx in 0..3 {
                if dy == 0 || dy == 2 || dx == 0 || dx == 2 {
                    b[y + dy][x + dx] = Some(4);
                }
            }
        }
    }
    b[0][1] = Some(4);
    let ga = grid(&s, b"a", &a);
    let gb = grid(&s, b"b", &b);
    let role = m.root.provision(b"role");
    role.provision(b"context-colour").point_to(
        &ga.cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    let tags = m
        .root
        .provision(b"native-components")
        .provision(b"relation-tags");
    role.provision(b"neighbour-kind")
        .point_to(&tags.provision(b"bounded"));
    let (pa, _) = m.component_value(&ga, &ga.cell(2, 1).unwrap()).unwrap();
    let (pb, _) = m.component_value(&gb, &gb.cell(0, 1).unwrap()).unwrap();
    assert!(
        pa.child(b"observed-cardinality")
            .unwrap()
            .follow()
            .unwrap()
            .same(&ordinal(&m, 16))
    );
    assert!(
        pb.child(b"observed-cardinality")
            .unwrap()
            .follow()
            .unwrap()
            .same(&ordinal(&m, 16))
    );
    let (_, one) = m
        .context_value(&ga, &ga.cell(2, 1).unwrap(), &role)
        .unwrap();
    let (old, two) = m
        .context_value(&gb, &gb.cell(0, 1).unwrap(), &role)
        .unwrap();
    assert!(one.same(&ordinal(&m, 1)));
    assert!(two.same(&ordinal(&m, 2)));
    role.child(b"neighbour-kind")
        .unwrap()
        .point_to(&tags.provision(b"escape"));
    let (_, outside) = m
        .context_value(&gb, &gb.cell(0, 1).unwrap(), &role)
        .unwrap();
    assert!(outside.same(&ordinal(&m, 1)));
    assert!(old.child(b"value").unwrap().follow().unwrap().same(&two));
}
#[test]
fn escape_with_unknown_frontier_is_not_a_complete_local_degree() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let input = grid(&s, b"a", &[vec![Some(4), None], vec![Some(4), Some(6)]]);
    let role = m.root.provision(b"role");
    role.provision(b"context-colour").point_to(
        &input
            .cell(1, 1)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap(),
    );
    role.provision(b"neighbour-kind").point_to(
        &m.root
            .provision(b"native-components")
            .provision(b"relation-tags")
            .provision(b"bounded"),
    );
    let (_, kind) = m
        .component_value(&input, &input.cell(0, 0).unwrap())
        .unwrap();
    assert_eq!(kind.token(), Some(b"escape".as_slice()));
    let (_, value) = m
        .context_value(&input, &input.cell(0, 0).unwrap(), &role)
        .unwrap();
    assert_eq!(value.token(), Some(b"unavailable".as_slice()));
}
