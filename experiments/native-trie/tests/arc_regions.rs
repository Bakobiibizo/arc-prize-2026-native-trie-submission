use native_trie_seed::{
    arc_colors::ColorError,
    arc_grids::{Grid, Grids, Side},
    arc_regions::RegionModel,
};
fn frame(n: usize, fill: u8, open: bool) -> Vec<Vec<Option<u8>>> {
    (0..n)
        .map(|y| {
            (0..n)
                .map(|x| {
                    Some(
                        if (y == 1 || y == n - 2) && x >= 1 && x <= n - 2
                            || (x == 1 || x == n - 2) && y >= 1 && y <= n - 2
                        {
                            if open && y == 1 && x == 2 { 0 } else { 2 }
                        } else if y > 1 && y < n - 2 && x > 1 && x < n - 2 {
                            fill
                        } else {
                            0
                        },
                    )
                })
                .collect()
        })
        .collect()
}
fn admit(s: &Grids, id: &[u8], side: Side, data: &[Vec<Option<u8>>]) -> Grid {
    let src = s
        .source(
            id,
            b"literal controlled paired data; original values retained in source cells",
        )
        .unwrap();
    s.admit(id, side, data, &src).unwrap()
}
fn teach(s: &Grids, m: &mut RegionModel, n: usize, id: &[u8], color: u8) {
    let mut outid = id.to_vec();
    outid.push(b'o');
    let a = admit(s, id, Side::TrainingInput, &frame(n, 0, false));
    let b = admit(s, &outid, Side::TrainingOutput, &frame(n, color, false));
    m.teach(id, &a, &b).unwrap();
}
#[test]
fn learns_on_native_relations_and_transfers_to_larger_closed_and_open_grids() {
    let mut s = Grids::new();
    let mut m = RegionModel::new(&s, b"task").unwrap();
    teach(&s, &mut m, 5, b"a", 4);
    teach(&s, &mut m, 7, b"b", 4);
    for (id, open) in [(b"closed".as_slice(), false), (b"opened".as_slice(), true)] {
        let a = admit(&s, id, Side::QueryInput, &frame(9, 0, open));
        let mut r = m.begin(id, &a).unwrap();
        assert!(r.relation_proofs <= 3);
        assert!(r.relation_cache_hits > 50);
        assert!(r.relation_copies > 0);
        m.execute(&mut r, 81).unwrap();
        let out = m.record(&mut s, r).unwrap();
        assert_eq!(
            out.inspect().unwrap(),
            frame(9, if open { 0 } else { 4 }, open)
        );
        assert!(out.observed_source().unwrap().is_none());
    }
}
#[test]
fn conflicting_relation_targets_remain_unresolved_and_never_fall_back() {
    let mut s = Grids::new();
    let mut m = RegionModel::new(&s, b"task").unwrap();
    teach(&s, &mut m, 5, b"a", 4);
    teach(&s, &mut m, 7, b"b", 8);
    let a = admit(&s, b"query", Side::QueryInput, &frame(9, 0, false));
    let mut r = m.begin(b"q", &a).unwrap();
    m.execute(&mut r, 81).unwrap();
    assert!(r.ambiguous > 0);
    let out = m.record(&mut s, r).unwrap();
    assert_eq!(out.inspect().unwrap()[4][4], None);
}
#[test]
fn durable_fact_budget_remains_an_explicit_error() {
    let s = Grids::new();
    let mut m = RegionModel::new(&s, b"task-facts").unwrap();
    m.max_fact_views = 0;
    let a = admit(&s, b"facts-a", Side::TrainingInput, &frame(5, 0, false));
    let b = admit(&s, b"facts-b", Side::TrainingOutput, &frame(5, 4, false));
    assert!(matches!(
        m.teach(b"facts", &a, &b),
        Err(ColorError::WorkspaceLimit)
    ));
}
#[test]
fn explicit_workspace_limit_is_an_error_not_a_fake_witness_or_color_prediction() {
    let s = Grids::new();
    let mut m = RegionModel::new(&s, b"task").unwrap();
    m.max_views = 0;
    let a = admit(&s, b"a", Side::TrainingInput, &frame(5, 0, false));
    let b = admit(&s, b"b", Side::TrainingOutput, &frame(5, 4, false));
    assert!(matches!(
        m.teach(b"lesson", &a, &b),
        Err(ColorError::WorkspaceLimit)
    ));
}
