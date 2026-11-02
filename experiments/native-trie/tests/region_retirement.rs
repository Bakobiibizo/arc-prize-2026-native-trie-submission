use native_trie_seed::{
    Trie,
    arc_grids::{Grid, Grids, Side},
    region_witness::{RegionError, RegionWitness},
};
fn no_dangling(root: &native_trie_seed::Address) {
    assert_eq!(RegionWitness::dangling_links(root), 0);
}
fn fixture() -> (Grids, Grid) {
    let s = Grids::new();
    let source = s.source(b"data", b"closed5x5 source").unwrap();
    let data = (0..5)
        .map(|y| {
            (0..5)
                .map(|x| {
                    Some(if y == 0 || x == 0 || y == 4 || x == 4 {
                        2
                    } else {
                        0
                    })
                })
                .collect()
        })
        .collect::<Vec<_>>();
    let g = s.admit(b"input", Side::QueryInput, &data, &source).unwrap();
    (s, g)
}
#[test]
fn native_fact_nodes_publish_unchanged_and_survive_execution_retirement() {
    let (s, g) = fixture();
    let store = Trie::default();
    let root = store.root();
    let cache = root.provision(b"cache");
    let seed = g.cell(2, 2).unwrap();
    let mut w = RegionWitness::prepare(&s, &g, &seed, &root, b"proof", &cache).unwrap();
    w.execute(w.budget).unwrap();
    assert!(RegionWitness::cached(&cache, &seed).unwrap().is_none());
    let original = w.view(&seed).unwrap();
    let before = RegionWitness::owned_nodes(&w.activation_root());
    let p = w.record().unwrap();
    no_dangling(&p.proof);
    let published = RegionWitness::recorded_view(&p.proof, &seed).unwrap();
    assert!(published.same(&original));
    assert!(p.proof.child(b"execution").is_none());
    assert!(p.proof.child(b"activation-retired").unwrap().is_terminal());
    assert!(RegionWitness::owned_nodes(&p.proof) < before / 4);
    assert!(
        RegionWitness::cached(&cache, &seed)
            .unwrap()
            .unwrap()
            .0
            .same(&p.proof)
    );
    for fact in RegionWitness::inspect_facts(&p.proof).unwrap() {
        assert!(fact.source.child(b"value").is_some());
        for (n, _) in fact.edges {
            assert!(n.child(b"value").is_some() || n.token() == Some(b"outside".as_slice()));
        }
    }
}
#[test]
fn partial_activation_and_written_facts_remain_failure_evidence() {
    let (s, g) = fixture();
    let store = Trie::default();
    let root = store.root();
    let cache = root.provision(b"cache");
    let seed = g.cell(2, 2).unwrap();
    let mut w = RegionWitness::prepare(&s, &g, &seed, &root, b"proof", &cache).unwrap();
    let proof = w.root.clone();
    assert!(matches!(w.execute(15), Err(RegionError::Limit)));
    assert!(matches!(w.record(), Err(RegionError::Incomplete)));
    no_dangling(&proof);
    assert!(
        proof
            .child(b"execution")
            .unwrap()
            .child(b"machine")
            .unwrap()
            .child(b"instruction-pointer")
            .unwrap()
            .follow()
            .is_ok()
    );
    assert!(
        proof
            .child(b"execution")
            .unwrap()
            .child(b"native-facts")
            .is_some()
    );
    assert!(proof.child(b"committed").is_none());
    assert!(RegionWitness::recorded_view(&proof, &seed).is_some());
    assert!(RegionWitness::cached(&cache, &seed).unwrap().is_none());
}
