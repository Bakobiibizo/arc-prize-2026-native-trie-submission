use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    positional::Positional,
};
#[path = "../support/heap.rs"]
mod heap;
fn admit(s: &Grids, id: &[u8], side: Side, v: &[Vec<u8>]) -> Grid {
    let source = s.source(id, b"positional test fixture").unwrap();
    s.admit(
        id,
        side,
        &v.iter()
            .map(|r| r.iter().copied().map(Some).collect())
            .collect::<Vec<_>>(),
        &source,
    )
    .unwrap()
}
#[test]
fn independently_matching_palette_independent_encoding_executes_new_input() {
    let mut s = Grids::new();
    let mut m = Positional::default();
    let a = admit(
        &s,
        b"a",
        Side::TrainingInput,
        &[vec![1, 2, 3], vec![4, 5, 6]],
    );
    let b = admit(
        &s,
        b"b",
        Side::TrainingOutput,
        &[vec![6, 5, 4], vec![3, 2, 1]],
    );
    let c = admit(
        &s,
        b"c",
        Side::TrainingInput,
        &[vec![8, 0, 7], vec![2, 9, 4]],
    );
    let d = admit(
        &s,
        b"d",
        Side::TrainingOutput,
        &[vec![4, 9, 2], vec![7, 0, 8]],
    );
    let x = m.derive(b"first", &a, &b).unwrap().encoding.unwrap();
    let y = m.derive(b"second", &c, &d).unwrap().encoding.unwrap();
    assert!(x.root.same(&y.root));
    let q = admit(&s, b"q", Side::QueryInput, &[vec![9, 9, 0], vec![2, 2, 1]]);
    m.begin(&q, &x).unwrap();
    heap::start();
    let copies = m.execute(18).unwrap();
    assert_eq!(heap::finish().requests, 0);
    assert_eq!(copies, 18);
    let out = m.record(&mut s).unwrap();
    assert_eq!(
        out.inspect().unwrap(),
        vec![
            vec![Some(1), Some(2), Some(2)],
            vec![Some(0), Some(9), Some(9)]
        ]
    );
    assert!(m.derive(b"late", &a, &b).is_err());
}
#[test]
fn agreement_requires_two_complete_encodings_and_rejects_foreign_execution() {
    let s = Grids::new();
    let a = admit(&s, b"a", Side::TrainingInput, &[vec![1, 2]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[vec![2, 1]]);
    let mut m = Positional::default();
    let d = m.derive(b"d", &a, &b).unwrap();
    assert!(Positional::agree(&[]).is_none());
    assert!(Positional::agree(std::slice::from_ref(&d)).is_none());
    let e = m.derive(b"e", &a, &b).unwrap();
    let agreed = Positional::agree(&[d, e]).unwrap();
    let mut other = Positional::default();
    assert!(other.begin(&a, &agreed).is_err());
}
#[test]
fn differing_encodings_do_not_match() {
    let s = Grids::new();
    let mut m = Positional::default();
    let a = admit(&s, b"a", Side::TrainingInput, &[vec![1, 2, 3]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[vec![3, 2, 1]]);
    let c = admit(&s, b"c", Side::TrainingOutput, &[vec![2, 3, 1]]);
    let x = m.derive(b"x", &a, &b).unwrap().encoding.unwrap();
    let y = m.derive(b"y", &a, &c).unwrap().encoding.unwrap();
    assert!(!x.root.same(&y.root));
}
#[test]
fn repeated_sources_and_new_colors_remain_explicitly_unresolved() {
    let s = Grids::new();
    let mut m = Positional::default();
    let a = admit(&s, b"a", Side::TrainingInput, &[vec![1, 1, 2]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[vec![1, 2, 4]]);
    let d = m.derive(b"d", &a, &b).unwrap();
    assert_eq!(d.unresolved, 2);
    assert!(d.encoding.is_none());
    assert!(d.evidence.child(b"by-value").is_some());
}
#[test]
fn partial_execution_and_different_shape_do_not_publish() {
    let mut s = Grids::new();
    let mut m = Positional::default();
    let a = admit(&s, b"a", Side::TrainingInput, &[vec![1, 2]]);
    let b = admit(&s, b"b", Side::TrainingOutput, &[vec![2, 1]]);
    let e = m.derive(b"d", &a, &b).unwrap().encoding.unwrap();
    let q = admit(&s, b"q", Side::QueryInput, &[vec![3], vec![4]]);
    assert!(m.begin(&q, &e).is_err());
    m.begin(&a, &e).unwrap();
    assert!(m.execute(2).is_err());
    assert!(m.record(&mut s).is_err());
    assert_eq!(m.execute(4).unwrap(), 6);
    assert_eq!(
        m.record(&mut s).unwrap().inspect().unwrap(),
        vec![vec![Some(2), Some(1)]]
    );
}
