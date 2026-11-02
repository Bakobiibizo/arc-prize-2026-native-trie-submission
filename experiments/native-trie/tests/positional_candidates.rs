use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    positional::Positional,
};
fn g(s: &Grids, id: &[u8], side: Side, v: &[u8]) -> Grid {
    let src = s.source(id, b"candidate positional fixture").unwrap();
    s.admit(id, side, &[v.iter().copied().map(Some).collect()], &src)
        .unwrap()
}
#[test]
fn independently_ambiguous_encodings_share_one_construction_without_joint_enumeration() {
    let mut s = Grids::new();
    let mut m = Positional::default();
    let a = g(&s, b"a", Side::TrainingInput, &[1, 1, 2]);
    let b = g(&s, b"b", Side::TrainingOutput, &[2, 1, 1]);
    let c = g(&s, b"c", Side::TrainingInput, &[3, 4, 4]);
    let d = g(&s, b"d", Side::TrainingOutput, &[4, 4, 3]);
    let first = m.derive(b"first", &a, &b).unwrap();
    let second = m.derive(b"second", &c, &d).unwrap();
    assert!(first.encoding.is_none() && second.encoding.is_none());
    assert!(Positional::agree(&[]).is_none());
    let agreed = m.agree_candidates(&[first, second]).unwrap().unwrap();
    let u = g(&s, b"u", Side::TrainingInput, &[7, 8, 9]);
    let v = g(&s, b"v", Side::TrainingOutput, &[9, 8, 7]);
    let unique = m.derive(b"unique", &u, &v).unwrap().encoding.unwrap();
    assert!(unique.root.same(&agreed.root));
    let q = g(&s, b"q", Side::QueryInput, &[5, 6, 7]);
    m.begin(&q, &agreed).unwrap();
    assert_eq!(m.execute(9).unwrap(), 9);
    assert_eq!(
        m.record(&mut s).unwrap().inspect().unwrap(),
        vec![vec![Some(7), Some(6), Some(5)]]
    );
}
#[test]
fn missing_source_is_mandatory_not_ignored() {
    let s = Grids::new();
    let mut m = Positional::default();
    let a = g(&s, b"a", Side::TrainingInput, &[1, 2]);
    let b = g(&s, b"b", Side::TrainingOutput, &[2, 1]);
    let c = g(&s, b"c", Side::TrainingOutput, &[9, 1]);
    let x = m.derive(b"x", &a, &b).unwrap();
    let y = m.derive(b"y", &a, &c).unwrap();
    assert!(m.agree_candidates(&[x, y]).unwrap().is_none());
}
#[test]
fn contradictory_candidate_positions_do_not_fallback() {
    let s = Grids::new();
    let mut m = Positional::default();
    let a = g(&s, b"a", Side::TrainingInput, &[1, 2]);
    let b = g(&s, b"b", Side::TrainingOutput, &[2, 1]);
    let x = m.derive(b"x", &a, &b).unwrap();
    let y = m.derive(b"y", &a, &a).unwrap();
    assert!(m.agree_candidates(&[x, y]).unwrap().is_none());
}
