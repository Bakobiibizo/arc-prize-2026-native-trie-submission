use native_trie_seed::{
    Address, Trie,
    arc_grids::{Grids, Side},
    complete_count_witness::CompleteCountWitness,
    region_witness::{RegionProjection, RegionWitness},
    source_field,
};
type Pixels = Vec<Vec<Option<u8>>>;
fn region(
    s: &Grids,
    parent: &Address,
    id: &[u8],
    a: &Pixels,
    seed: (usize, usize),
) -> RegionProjection {
    let src = s
        .source(id, b"declared complete-cardinality witness fixture")
        .unwrap();
    let g = s.admit(id, Side::TrainingInput, a, &src).unwrap();
    let mut w = RegionWitness::prepare_counted(
        s,
        &g,
        &g.cell(seed.0, seed.1).unwrap(),
        parent,
        id,
        &parent.provision(b"cache").provision(id),
    )
    .unwrap();
    w.execute(w.budget).unwrap();
    w.record().unwrap()
}
fn unary(parent: &Address, n: usize) -> Address {
    let mut at = parent
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
fn known_single_multi_and_escape_publish_actual_unary_reference() {
    let s = Grids::new();
    let parent = Trie::default().root();
    let cases = [
        (
            vec![
                vec![Some(1); 3],
                vec![Some(1), Some(0), Some(1)],
                vec![Some(1); 3],
            ],
            (1, 1),
            1,
        ),
        (
            vec![
                vec![Some(1); 4],
                vec![Some(1), Some(0), Some(0), Some(1)],
                vec![Some(1), Some(0), Some(0), Some(1)],
                vec![Some(1); 4],
            ],
            (1, 1),
            4,
        ),
        (vec![vec![Some(0); 2]; 2], (0, 0), 4),
    ];
    for (i, (a, seed, n)) in cases.iter().enumerate() {
        let id = i.to_be_bytes();
        let p = region(&s, &parent, &id, a, *seed);
        let mut w = CompleteCountWitness::prepare(p, &parent, &id).unwrap();
        let budget = w.budget;
        assert_eq!(w.execute(budget).unwrap(), budget as u64);
        let p = w.record().unwrap();
        assert!(p.value.same(&unary(&parent, *n)));
        assert_eq!(p.status.token(), Some(b"complete".as_slice()));
        assert!(p.proof.child(b"committed").unwrap().is_terminal());
    }
}
#[test]
fn unknown_frontier_for_multi_count_escape_and_seed_are_unavailable() {
    let s = Grids::new();
    let parent = Trie::default().root();
    let cases = [
        (
            vec![
                vec![Some(1); 4],
                vec![Some(1), Some(0), None, Some(1)],
                vec![Some(1), Some(0), Some(0), Some(1)],
                vec![Some(1); 4],
            ],
            (1, 1),
        ),
        (vec![vec![Some(0), None], vec![Some(0), Some(0)]], (0, 0)),
        (vec![vec![None]], (0, 0)),
    ];
    for (i, (a, seed)) in cases.iter().enumerate() {
        let id = i.to_be_bytes();
        let p = region(&s, &parent, &id, a, *seed);
        if i == 1 {
            assert_eq!(p.value.token(), Some(b"escape".as_slice()));
        }
        let mut w = CompleteCountWitness::prepare(p, &parent, &id).unwrap();
        w.execute(w.budget).unwrap();
        let p = w.record().unwrap();
        assert_eq!(p.value.token(), Some(b"unavailable".as_slice()));
        assert_eq!(p.status.token(), Some(b"unknown-frontier".as_slice()));
    }
}
#[test]
fn invalidated_region_commit_marker_is_not_trusted() {
    let s = Grids::new();
    let parent = Trie::default().root();
    let p = region(&s, &parent, b"r", &vec![vec![Some(0)]], (0, 0));
    p.proof
        .child(b"committed")
        .unwrap()
        .point_to(&parent.provision(b"not-committed"));
    assert!(CompleteCountWitness::prepare(p, &parent, b"c").is_err());
}
#[test]
fn interrupted_and_faulted_native_attempts_retain_control_without_count() {
    for fault in [false, true] {
        let s = Grids::new();
        let parent = Trie::default().root();
        let p = region(&s, &parent, b"r", &vec![vec![Some(0); 2]; 2], (0, 0));
        let mut w = CompleteCountWitness::prepare(p, &parent, b"c").unwrap();
        let root = w.root.clone();
        if fault {
            let current = w.machine.current().unwrap();
            source_field(&current)
                .unwrap()
                .point_to(&root.provision(b"unset-test-source"));
            assert!(w.execute(w.budget).is_err());
        } else {
            assert!(w.execute(0).is_err());
        }
        let actual = w.machine.cycles;
        assert!(w.record().is_err());
        assert!(root.child(b"committed").is_none());
        assert!(root.child(b"value").unwrap().follow().is_err());
        assert!(
            root.child(b"attempt-copies")
                .unwrap()
                .child(&actual.to_be_bytes())
                .is_some()
        );
        assert!(
            root.child(b"execution")
                .unwrap()
                .child(b"machine")
                .is_some()
        );
        assert_eq!(RegionWitness::dangling_links(&parent), 0);
    }
}
#[test]
fn projection_keeps_native_facts_alive_and_constructor_charge_fits_reserve() {
    let s = Grids::new();
    let parent = Trie::default().root();
    let p = region(&s, &parent, b"r", &vec![vec![Some(0); 30]; 30], (0, 0));
    let region_proof = p.proof.clone();
    let reserve = CompleteCountWitness::reserve_bytes(&p).unwrap();
    let before = RegionWitness::storage_charge(&parent);
    let mut w = CompleteCountWitness::prepare(p, &parent, b"c").unwrap();
    assert!(RegionWitness::storage_charge(&parent) - before < reserve);
    w.execute(w.budget).unwrap();
    let p = w.record().unwrap();
    let expected = unary(&parent, 900);
    drop(s);
    drop(parent);
    assert!(p.value.same(&expected));
    assert!(region_proof.child(b"views").is_some());
    assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
    let mut raw = Vec::new();
    native_trie_seed::graph_export::write(std::slice::from_ref(&p.proof), &mut raw).unwrap();
    assert!(!String::from_utf8(raw).unwrap().contains("\"fault\""));
}
