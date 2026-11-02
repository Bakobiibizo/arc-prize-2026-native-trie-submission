use native_trie_seed::{
    Trie,
    arc_grids::{Grids, Side},
    region_witness::{RegionError, RegionWitness},
    source_field,
};
fn fixture() -> (Grids, native_trie_seed::arc_grids::Grid) {
    let s = Grids::new();
    let src = s.source(b"source", b"closed5x5 supplied data").unwrap();
    let a = (0..5)
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
    let g = s.admit(b"input", Side::QueryInput, &a, &src).unwrap();
    (s, g)
}
#[test]
fn native_emit_mutation_changes_relation_not_a_host_floodfill_classifier() {
    let (s, g) = fixture();
    let store = Trie::default();
    let root = store.root();
    let mut w = RegionWitness::prepare(
        &s,
        &g,
        &g.cell(2, 2).unwrap(),
        &root,
        b"proof",
        &root.provision(b"cache"),
    )
    .unwrap();
    let emit = w.construction.child(b"emit-bounded").unwrap();
    source_field(&emit)
        .unwrap()
        .point_to(&w.activation_root().child(b"escape-tag").unwrap());
    w.execute(w.budget).unwrap();
    assert_eq!(
        w.record().unwrap().value.token(),
        Some(b"escape".as_slice())
    );
}
#[test]
fn broken_native_data_faults_without_host_repair() {
    let (s, g) = fixture();
    let store = Trie::default();
    let root = store.root();
    let mut w = RegionWitness::prepare(
        &s,
        &g,
        &g.cell(2, 2).unwrap(),
        &root,
        b"proof",
        &root.provision(b"cache"),
    )
    .unwrap();
    let emit = w.construction.child(b"emit-bounded").unwrap();
    source_field(&emit)
        .unwrap()
        .point_to(&root.provision(b"unset"));
    assert!(matches!(w.execute(w.budget), Err(RegionError::Machine(_))));
    assert!(matches!(w.record(), Err(RegionError::Incomplete)));
}
