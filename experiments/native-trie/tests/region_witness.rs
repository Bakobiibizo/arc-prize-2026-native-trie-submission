use native_trie_seed::{
    Trie,
    arc_grids::{Grid, Grids, Side},
    region_witness::{RegionError, RegionWitness},
};
fn grid(s: &Grids, id: &[u8], data: &[Vec<Option<u8>>]) -> Grid {
    let src = s
        .source(
            id,
            b"supplied fixture; values retained verbatim in native source cells",
        )
        .unwrap();
    s.admit(id, Side::QueryInput, data, &src).unwrap()
}
fn outline(h: usize, w: usize) -> Vec<Vec<Option<u8>>> {
    (0..h)
        .map(|y| {
            (0..w)
                .map(|x| {
                    Some(if y == 0 || x == 0 || y + 1 == h || x + 1 == w {
                        2
                    } else {
                        0
                    })
                })
                .collect()
        })
        .collect()
}
#[test]
fn closed_open_and_unknown_frontiers_are_native_results() {
    let s = Grids::new();
    for (i, kind) in ["bounded", "escape", "unresolved"].into_iter().enumerate() {
        let mut a = outline(5, 5);
        if i == 1 {
            a[0][2] = Some(0)
        }
        if i == 2 {
            a[0][2] = None
        }
        let g = grid(&s, &i.to_be_bytes(), &a);
        let store = Trie::default();
        let root = store.root();
        let cache = root.provision(b"cache");
        let mut w = RegionWitness::prepare(&s, &g, &g.cell(2, 2).unwrap(), &root, b"proof", &cache)
            .unwrap();
        let limit = w.budget;
        w.execute(limit).unwrap();
        let (n, e, done) = w.inspect().unwrap();
        assert_eq!(e, 4 * n);
        assert_eq!(done, n);
        let p = w.record().unwrap();
        assert_eq!(p.value.token().unwrap(), kind.as_bytes());
        assert!(
            p.proof
                .child(b"frontier-exhausted")
                .unwrap()
                .follow()
                .is_ok()
        );
    }
}
#[test]
fn distant_opening_and_escape_despite_unknown_frontier() {
    let s = Grids::new();
    let mut a = outline(7, 9);
    a[0][7] = Some(0);
    a[6][2] = None;
    let g = grid(&s, b"a", &a);
    let store = Trie::default();
    let root = store.root();
    let mut w = RegionWitness::prepare(
        &s,
        &g,
        &g.cell(3, 3).unwrap(),
        &root,
        b"proof",
        &root.provision(b"cache"),
    )
    .unwrap();
    w.execute(w.budget).unwrap();
    assert_eq!(
        w.record().unwrap().value.token(),
        Some(b"escape".as_slice())
    );
}
#[test]
fn unknown_seed_is_not_an_equal_unknown_region_or_closed_proof() {
    let s = Grids::new();
    let g = grid(&s, b"a", &[vec![None, None]]);
    let store = Trie::default();
    let root = store.root();
    let mut w = RegionWitness::prepare(
        &s,
        &g,
        &g.cell(0, 0).unwrap(),
        &root,
        b"proof",
        &root.provision(b"cache"),
    )
    .unwrap();
    assert_eq!(w.execute(w.budget).unwrap(), 3);
    assert_eq!(w.inspect().unwrap(), (0, 0, 0));
    let p = w.record().unwrap();
    assert_eq!(p.value.token(), Some(b"unresolved".as_slice()));
    assert!(
        p.proof
            .child(b"frontier-exhausted")
            .unwrap()
            .follow()
            .is_err()
    );
}
#[test]
fn partial_budget_never_means_exhausted_and_cached_proof_is_not_completed() {
    let s = Grids::new();
    let g = grid(&s, b"a", &outline(5, 5));
    let store = Trie::default();
    let root = store.root();
    let cache = root.provision(b"cache");
    let mut w =
        RegionWitness::prepare(&s, &g, &g.cell(2, 2).unwrap(), &root, b"proof", &cache).unwrap();
    assert!(matches!(w.execute(15), Err(RegionError::Limit)));
    assert!(w.result.follow().is_err());
    assert!(
        w.root
            .child(b"frontier-exhausted")
            .unwrap()
            .follow()
            .is_err()
    );
    assert!(matches!(w.record(), Err(RegionError::Incomplete)));
}
#[test]
fn repeated_membership_is_cacheable_but_barriers_are_not_members() {
    let s = Grids::new();
    let g = grid(&s, b"a", &outline(5, 5));
    let store = Trie::default();
    let root = store.root();
    let cache = root.provision(b"cache");
    let mut w =
        RegionWitness::prepare(&s, &g, &g.cell(2, 2).unwrap(), &root, b"proof", &cache).unwrap();
    let before = g.inspect().unwrap();
    w.execute(w.budget).unwrap();
    let p = w.record().unwrap();
    assert_eq!(g.inspect().unwrap(), before);
    assert!(
        p.proof
            .child(b"input")
            .unwrap()
            .follow()
            .unwrap()
            .same(&g.anchor)
    );
    // Source-view proof slots are native writes, not a host membership classifier.
    for y in 1..4 {
        for x in 1..4 {
            let (proof, value) = RegionWitness::cached(&cache, &g.cell(y, x).unwrap())
                .unwrap()
                .unwrap();
            assert!(proof.same(&p.proof));
            assert!(value.same(&p.value));
        }
    }
    assert!(
        RegionWitness::cached(&cache, &g.cell(0, 2).unwrap())
            .unwrap()
            .is_none()
    );
}
#[test]
fn work_scales_with_native_visited_source_not_the_number_of_walk_paths() {
    let s = Grids::new();
    for size in [5usize, 9, 15] {
        let g = grid(&s, &size.to_be_bytes(), &outline(size, size));
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
        let copies = w.execute(w.budget).unwrap();
        let (m, edges, done) = w.inspect().unwrap();
        assert_eq!(m, (size - 2) * (size - 2));
        assert_eq!(edges, 4 * m);
        assert_eq!(done, m);
        assert!(copies < (40 * size * size + 32) as u64);
        assert_eq!(
            w.record().unwrap().value.token(),
            Some(b"bounded".as_slice())
        );
    }
}
