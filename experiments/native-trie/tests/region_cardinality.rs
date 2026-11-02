#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Address, Trie,
    arc_grids::{Grids, Side},
    region_witness::RegionWitness,
};
fn run(data: Vec<Vec<Option<u8>>>, seed: (usize, usize), expected: usize) {
    let s = Grids::new();
    let source = s
        .source(b"source", b"independent cardinality control")
        .unwrap();
    let g = s.admit(b"input", Side::QueryInput, &data, &source).unwrap();
    let owner = Trie::default();
    let root = owner.root();
    let cell = g.cell(seed.0, seed.1).unwrap();
    let mut baseline = RegionWitness::prepare(
        &s,
        &g,
        &cell,
        &root,
        b"base",
        &root.provision(b"base-cache"),
    )
    .unwrap();
    baseline.execute(baseline.budget).unwrap();
    let base = baseline.record().unwrap();
    assert!(base.proof.child(b"observed-cardinality").is_none());
    let mut w = RegionWitness::prepare_counted(
        &s,
        &g,
        &cell,
        &root,
        b"counted",
        &root.provision(b"count-cache"),
    )
    .unwrap();
    heap::start();
    let copies = w.execute(w.budget).unwrap();
    let sample = heap::finish();
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    let p = w.record().unwrap();
    assert!(p.value.same(&base.value));
    let old_facts = RegionWitness::inspect_facts(&base.proof).unwrap();
    let new_facts = RegionWitness::inspect_facts(&p.proof).unwrap();
    assert_eq!(old_facts.len(), new_facts.len());
    for (a, b) in old_facts.iter().zip(&new_facts) {
        assert!(a.source.same(&b.source));
        assert_eq!(a.expanded, b.expanded);
        assert_eq!(a.class.is_some(), b.class.is_some());
        if let (Some(x), Some(y)) = (&a.class, &b.class) {
            assert!(x.same(y));
        }
        assert_eq!(a.edges.len(), b.edges.len());
        for ((x, c), (y, d)) in a.edges.iter().zip(&b.edges) {
            assert!(x.same(y));
            assert_eq!(c, d);
        }
    }
    assert_eq!(copies, base.copies + 2 * expected as u64);
    let mut want = root
        .child(b"cardinalities")
        .unwrap()
        .child(b"zero")
        .unwrap();
    for _ in 0..expected {
        want = want.child(b"successor").unwrap();
    }
    let actual = p
        .proof
        .child(b"observed-cardinality")
        .unwrap()
        .follow()
        .unwrap();
    assert!(actual.same(&want));
    assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
    assert!(p.proof.child(b"execution").is_none());
}
#[test]
fn singleton() {
    run(vec![vec![Some(8)]], (0, 0), 1)
}
#[test]
fn uniform_nine_counts_cycles_once() {
    run(vec![vec![Some(1); 3]; 3], (1, 1), 9)
}
#[test]
fn maximum900() {
    run(vec![vec![Some(0); 30]; 30], (15, 15), 900)
}
#[test]
fn closed_frame_inner_nine() {
    let mut a = vec![vec![Some(2); 5]; 5];
    for row in &mut a[1..4] {
        for v in &mut row[1..4] {
            *v = Some(0);
        }
    }
    run(a, (2, 2), 9)
}
#[test]
fn moved_recolored_disconnected_component() {
    run(
        vec![
            vec![Some(0), Some(4), Some(0), Some(0)],
            vec![Some(0), Some(4), Some(0), Some(7)],
            vec![Some(0), Some(0), Some(0), Some(7)],
        ],
        (1, 3),
        2,
    )
}
#[test]
fn unknown_seed_zero_is_observed_not_total() {
    run(vec![vec![None]], (0, 0), 0)
}
#[test]
fn unknown_frontier_observed_only() {
    run(vec![vec![Some(0), None, Some(0)]], (0, 0), 1)
}
#[test]
fn number_identity_is_shared_across_proofs_not_source_colors() {
    let s = Grids::new();
    let source = s
        .source(b"source", b"same two-cell topology colors differ")
        .unwrap();
    let root = Trie::default().root();
    let mut counts: Vec<Address> = vec![];
    for (id, color) in [(b"one".as_slice(), 2), (b"two".as_slice(), 6)] {
        let g = s
            .admit(id, Side::QueryInput, &[vec![Some(color); 2]], &source)
            .unwrap();
        let mut w = RegionWitness::prepare_counted(
            &s,
            &g,
            &g.cell(0, 0).unwrap(),
            &root,
            id,
            &root.provision(id),
        )
        .unwrap();
        w.execute(w.budget).unwrap();
        counts.push(
            w.record()
                .unwrap()
                .proof
                .child(b"observed-cardinality")
                .unwrap()
                .follow()
                .unwrap(),
        );
    }
    assert!(counts[0].same(&counts[1]));
}
