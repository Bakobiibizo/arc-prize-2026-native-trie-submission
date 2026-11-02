#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Address, Trie,
    arc_grids::{Grid, Grids, Side},
    arc_markers::MarkerModel,
    records::{self, Selection},
};
fn add(a: &Address, t: &Address, e: &Address) {
    records::record(a, b"source", t, e).unwrap();
}
#[test]
fn candidate_reference_overlap_is_not_strict_resolve_relaxation() {
    let r = Trie::default().root();
    let a = r.provision(b"a");
    let b = r.provision(b"b");
    let e = r.provision(b"e");
    let x = r.provision(b"x");
    let y = r.provision(b"y");
    let z = r.provision(b"z");
    for t in [&x, &y] {
        add(&a, t, &e);
    }
    for t in [&y, &z] {
        add(&b, t, &e);
    }
    assert!(matches!(
        records::resolve(&a, b"source").unwrap(),
        Selection::Ambiguous { alternatives: 2 }
    ));
    match records::intersect(&[Some(&a), Some(&b)], b"source").unwrap() {
        Selection::Unique { target, .. } => assert!(target.same(&y)),
        _ => panic!("no unique reference"),
    };
}
#[test]
fn absent_heads_and_disjoint_support_are_distinct() {
    let r = Trie::default().root();
    let a = r.provision(b"a");
    let b = r.provision(b"b");
    let e = r.provision(b"e");
    assert!(matches!(
        records::intersect(&[Some(&a), None], b"source").unwrap(),
        Selection::Unknown
    ));
    add(&a, &r.provision(b"x"), &e);
    assert!(matches!(
        records::intersect(&[Some(&a), Some(&b)], b"source").unwrap(),
        Selection::Unique { .. }
    ));
    add(&b, &r.provision(b"y"), &e);
    assert!(matches!(
        records::intersect(&[Some(&a), Some(&b)], b"source").unwrap(),
        Selection::Ambiguous { alternatives: 0 }
    ));
}
#[test]
fn equal_slot_contents_do_not_make_source_recipes_identical() {
    let r = Trie::default().root();
    let a = r.provision(b"a");
    let b = r.provision(b"b");
    let e = r.provision(b"e");
    let x = r.provision(b"x");
    let y = r.provision(b"y");
    let value = r.provision(b"value");
    x.point_to(&value);
    y.point_to(&value);
    add(&a, &x, &e);
    add(&b, &y, &e);
    assert!(matches!(
        records::intersect(&[Some(&a), Some(&b)], b"source").unwrap(),
        Selection::Ambiguous { alternatives: 0 }
    ));
}
#[test]
fn physical_candidate_constraint_lookup_allocates_nothing() {
    let r = Trie::default().root();
    let a = r.provision(b"a");
    let b = r.provision(b"b");
    let e = r.provision(b"e");
    let x = r.provision(b"x");
    let y = r.provision(b"y");
    add(&a, &x, &e);
    add(&a, &y, &e);
    add(&b, &x, &e);
    heap::start();
    for _ in 0..20 {
        assert!(matches!(
            records::intersect(&[Some(&a), Some(&b)], b"source").unwrap(),
            Selection::Unique { .. }
        ));
    }
    assert_eq!(heap::finish().requests, 0);
}
fn admit(s: &Grids, id: &[u8], side: Side, a: &[Vec<Option<u8>>]) -> Grid {
    let raw = format!("{a:?}");
    let src = s.source(id, raw.as_bytes()).unwrap();
    s.admit(id, side, a, &src).unwrap()
}
fn teach(m: &mut MarkerModel, s: &Grids, id: &[u8], a: &[Vec<Option<u8>>], b: &[Vec<Option<u8>>]) {
    let i = admit(s, &[id, b"i"].concat(), Side::TrainingInput, a);
    let o = admit(s, &[id, b"o"].concat(), Side::TrainingOutput, b);
    m.teach(id, &i, &o).unwrap();
}
fn predict(m: &mut MarkerModel, s: &mut Grids, a: &[Vec<Option<u8>>]) -> Vec<Vec<Option<u8>>> {
    let q = admit(s, b"query", Side::QueryInput, a);
    let mut run = m.begin(b"query", &q).unwrap();
    heap::start();
    m.execute(&mut run, a.len() * a[0].len()).unwrap();
    assert_eq!(heap::finish().requests, 0);
    assert_eq!(run.cycles, 2 * (a.len() * a[0].len()) as u64);
    m.record(s, run).unwrap().inspect().unwrap()
}
#[test]
fn color_refines_coarse_region_source_ambiguity() {
    let mut s = Grids::new();
    let mut m = MarkerModel::new_constraints(&s, b"task").unwrap();
    teach(
        &mut m,
        &s,
        b"a",
        &[vec![Some(1), Some(1), Some(2), Some(2)]],
        &[vec![Some(4), Some(4), Some(5), Some(5)]],
    );
    assert_eq!(
        predict(&mut m, &mut s, &[vec![Some(2), Some(2), Some(1), Some(1)]]),
        vec![vec![Some(5), Some(5), Some(4), Some(4)]]
    );
}
#[test]
fn compatible_broad_color_narrows_multiple_when_exact_kind_color_missing() {
    let mut s = Grids::new();
    let mut m = MarkerModel::new_constraints(&s, b"task").unwrap();
    let mut a = vec![vec![Some(0); 5]; 5];
    let mut b = a.clone();
    for (y, x) in [(2, 2), (2, 3)] {
        a[y][x] = Some(1);
        b[y][x] = Some(4);
    }
    for (y, x) in [(0, 0), (0, 1)] {
        a[y][x] = Some(3);
        b[y][x] = Some(4);
    }
    for (y, x) in [(4, 3), (4, 4)] {
        a[y][x] = Some(2);
        b[y][x] = Some(5);
    }
    teach(&mut m, &s, b"a", &a, &b);
    assert_eq!(
        predict(&mut m, &mut s, &[vec![Some(1), Some(1), Some(2), Some(2)]]),
        vec![vec![Some(4), Some(4), Some(5), Some(5)]]
    );
}
#[test]
fn disjoint_structured_support_never_falls_back_to_unique_color() {
    let mut s = Grids::new();
    let mut m = MarkerModel::new_constraints(&s, b"task").unwrap();
    teach(
        &mut m,
        &s,
        b"a",
        &[vec![Some(1), Some(1), Some(2)]],
        &[vec![Some(4), Some(4), Some(5)]],
    );
    assert_eq!(predict(&mut m, &mut s, &[vec![Some(1)]]), vec![vec![None]]);
}
#[test]
fn exact_refined_conflicts_remain_unresolved() {
    let mut s = Grids::new();
    let mut m = MarkerModel::new_constraints(&s, b"task").unwrap();
    teach(
        &mut m,
        &s,
        b"a",
        &[vec![Some(1), Some(1)]],
        &[vec![Some(4), Some(4)]],
    );
    teach(
        &mut m,
        &s,
        b"b",
        &[vec![Some(1), Some(1)]],
        &[vec![Some(5), Some(5)]],
    );
    assert_eq!(
        predict(&mut m, &mut s, &[vec![Some(1), Some(1)]]),
        vec![vec![None, None]]
    );
}
#[test]
fn native_unbound_applicability_keeps_inactive_observations_but_not_query_constraint() {
    let mut s = Grids::new();
    let mut m = MarkerModel::new_constraints(&s, b"task").unwrap();
    teach(
        &mut m,
        &s,
        b"a",
        &[vec![Some(1), Some(1), Some(2), Some(2)]],
        &[vec![Some(4), Some(4), Some(5), Some(5)]],
    );
    assert_eq!(
        predict(&mut m, &mut s, &[vec![Some(2), Some(2), Some(1), Some(1)]]),
        vec![vec![Some(5), Some(5), Some(4), Some(4)]]
    );
    let run = m.base.root.child(b"runs").unwrap().child(b"query").unwrap();
    let census = run.child(b"marker-census").unwrap().follow().unwrap();
    let scope = census.child(b"head-scope").unwrap().follow().unwrap();
    assert_eq!(scope.token(), Some(b"role-unbound".as_slice()));
    assert!(
        !scope
            .child(b"record-prefix")
            .unwrap()
            .follow()
            .unwrap()
            .same(&scope.child(b"query-prefix").unwrap().follow().unwrap())
    );
    assert!(
        native_trie_seed::region_witness::RegionWitness::storage_charge(
            &m.base.root.child(b"inactive-marker-observations").unwrap()
        ) > 1024
    );
    assert!(
        census
            .child(b"applicability-proof")
            .unwrap()
            .follow()
            .unwrap()
            .child(b"copies")
            .unwrap()
            .child(&6u64.to_be_bytes())
            .is_some()
    );
}
#[test]
fn bound_applicability_is_a_native_shared_record_query_prefix() {
    let mut s = Grids::new();
    let mut m = MarkerModel::new_constraints(&s, b"task").unwrap();
    teach(
        &mut m,
        &s,
        b"a",
        &marker(5, 3, 6, false),
        &marker(5, 3, 6, true),
    );
    teach(
        &mut m,
        &s,
        b"b",
        &marker(7, 2, 4, false),
        &marker(7, 2, 4, true),
    );
    assert_eq!(
        predict(&mut m, &mut s, &marker(5, 8, 2, false)),
        marker(5, 8, 2, true)
    );
    let run = m.base.root.child(b"runs").unwrap().child(b"query").unwrap();
    let scope = run
        .child(b"marker-census")
        .unwrap()
        .follow()
        .unwrap()
        .child(b"head-scope")
        .unwrap()
        .follow()
        .unwrap();
    assert_eq!(scope.token(), Some(b"role-bound".as_slice()));
    assert!(
        scope
            .child(b"record-prefix")
            .unwrap()
            .follow()
            .unwrap()
            .same(&scope.child(b"query-prefix").unwrap().follow().unwrap())
    );
}
fn marker(n: usize, color: u8, role: u8, out: bool) -> Vec<Vec<Option<u8>>> {
    let mut a = vec![vec![Some(0); n]; n];
    for (y, x) in [(1, 1), (1, 2), (2, 1), (2, 2)] {
        a[y][x] = Some(if out { role } else { color });
    }
    if !out {
        a[n - 1][0] = Some(role);
    }
    a
}
#[test]
fn structured_variable_and_singleton_deletion_refine_old_color_mapping() {
    let mut s = Grids::new();
    let mut m = MarkerModel::new_constraints(&s, b"task").unwrap();
    teach(
        &mut m,
        &s,
        b"a",
        &marker(5, 3, 6, false),
        &marker(5, 3, 6, true),
    );
    teach(
        &mut m,
        &s,
        b"b",
        &marker(7, 2, 4, false),
        &marker(7, 2, 4, true),
    );
    assert_eq!(
        predict(&mut m, &mut s, &marker(5, 8, 2, false)),
        marker(5, 8, 2, true)
    );
}
