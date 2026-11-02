#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Address, Trie,
    arc_grids::{Grid, Grids, Side},
    region_witness::{RegionError, RegionProjection, RegionWitness},
    singleton_witness::SingletonWitness,
    source_field,
};
fn setup(data: &[Vec<Option<u8>>]) -> (Grids, Grid, Address) {
    let s = Grids::new();
    let source = s.source(b"source", b"supplied singleton controls").unwrap();
    let g = s.admit(b"input", Side::QueryInput, data, &source).unwrap();
    (s, g, Trie::default().root())
}
fn region(s: &Grids, g: &Grid, root: &Address, seed: (usize, usize)) -> RegionProjection {
    let mut w = RegionWitness::prepare_counted(
        s,
        g,
        &g.cell(seed.0, seed.1).unwrap(),
        root,
        b"region",
        &root.provision(b"cache"),
    )
    .unwrap();
    w.execute(w.budget).unwrap();
    w.record().unwrap()
}
fn prove(data: &[Vec<Option<u8>>], seed: (usize, usize), status: &[u8], candidate: bool) {
    let (s, g, root) = setup(data);
    let p = region(&s, &g, &root, seed);
    let mut w = SingletonWitness::prepare(p, &root, b"role").unwrap();
    assert!(w.root.child(b"candidate").unwrap().follow().is_err());
    let budget = w.budget;
    heap::start();
    let copies = w.execute(budget).unwrap();
    let alloc = heap::finish();
    assert_eq!(alloc.requests, 0);
    assert!(copies <= budget as u64);
    let p = w.record().unwrap();
    assert_eq!(p.status.token(), Some(status));
    assert_eq!(p.candidate.is_some(), candidate);
    if let Some(value) = p.candidate {
        assert!(
            value.same(
                &g.cell(seed.0, seed.1)
                    .unwrap()
                    .child(b"value")
                    .unwrap()
                    .follow()
                    .unwrap()
            )
        );
    }
    assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
}
#[test]
fn provided_palette_and_position_transfer() {
    for c in 0..10 {
        prove(&[vec![Some(c)]], (0, 0), b"singleton", true);
        let other = (c + 1) % 10;
        prove(
            &[
                vec![Some(other); 3],
                vec![Some(other), Some(c), Some(other)],
                vec![Some(other); 3],
            ],
            (1, 1),
            b"singleton",
            true,
        );
    }
}
#[test]
fn plural_is_not_a_singleton() {
    prove(
        &[vec![Some(6), Some(6)]],
        (0, 1),
        b"not-observed-one",
        false,
    )
}
#[test]
fn unknown_seed_is_not_empty_or_singleton() {
    prove(&[vec![None]], (0, 0), b"not-observed-one", false)
}
#[test]
fn escape_does_not_override_unknown_cardinality() {
    prove(&[vec![Some(8), None]], (0, 0), b"unknown-frontier", false)
}
#[test]
fn bounded_unknown_frontier_does_not_publish_marker() {
    prove(
        &[
            vec![Some(0), None, Some(0)],
            vec![Some(0), Some(4), Some(0)],
            vec![Some(0); 3],
        ],
        (1, 1),
        b"unknown-frontier",
        false,
    )
}
#[test]
fn actual_native_fault_is_not_repaired() {
    let (s, g, root) = setup(&[vec![Some(8)]]);
    let p = region(&s, &g, &root, (0, 0));
    let mut w = SingletonWitness::prepare(p, &root, b"role").unwrap();
    let proof = w.root.clone();
    source_field(&w.construction.child(b"emit-source-color").unwrap())
        .unwrap()
        .point_to(&root.provision(b"unset"));
    assert!(matches!(w.execute(w.budget), Err(RegionError::Machine(_))));
    assert!(matches!(w.record(), Err(RegionError::Incomplete)));
    assert!(proof.child(b"committed").is_none());
    assert!(proof.child(b"candidate").unwrap().follow().is_err());
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
}
#[test]
fn partial_native_run_never_commits_role() {
    let (s, g, root) = setup(&[vec![Some(8)]]);
    let p = region(&s, &g, &root, (0, 0));
    let mut w = SingletonWitness::prepare(p, &root, b"role").unwrap();
    let proof = w.root.clone();
    assert!(matches!(w.execute(1), Err(RegionError::Limit)));
    assert!(matches!(w.record(), Err(RegionError::Incomplete)));
    assert!(proof.child(b"committed").is_none());
}
#[test]
fn uncounted_projection_is_not_silently_counted_in_host() {
    let (s, g, root) = setup(&[vec![Some(8)]]);
    let mut w = RegionWitness::prepare(
        &s,
        &g,
        &g.cell(0, 0).unwrap(),
        &root,
        b"region",
        &root.provision(b"cache"),
    )
    .unwrap();
    w.execute(w.budget).unwrap();
    assert!(SingletonWitness::prepare(w.record().unwrap(), &root, b"role").is_err());
}
