use native_trie_seed::{
    arc_grids::{Grids, Side},
    sparse_rewrite::SparseRewrite,
};
#[test]
fn empty_extra_relations_still_selects_source_colour_and_requires_negative_validation() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let src = s.source(b"s", b"empty conjunction fixture").unwrap();
    let a = s
        .admit(b"a", Side::TrainingInput, &[vec![Some(0)]], &src)
        .unwrap();
    let b = s
        .admit(b"b", Side::TrainingOutput, &[vec![Some(4)]], &src)
        .unwrap();
    let c = s
        .admit(b"c", Side::TrainingInput, &[vec![Some(0), Some(0)]], &src)
        .unwrap();
    let d = s
        .admit(b"d", Side::TrainingOutput, &[vec![Some(4), Some(4)]], &src)
        .unwrap();
    let x = m.derive_input_relations(b"x", &a, &b).unwrap();
    let y = m.derive_input_relations(b"y", &c, &d).unwrap();
    let rule = SparseRewrite::agree(&[x, y]).unwrap();
    assert_eq!(rule.clauses(), 0);
    let q = s
        .admit(
            b"q",
            Side::QueryInput,
            &[vec![Some(0), Some(1), Some(0)]],
            &src,
        )
        .unwrap();
    assert_eq!(
        m.apply(&mut s, &q, &rule).unwrap().inspect().unwrap(),
        vec![vec![Some(4), Some(1), Some(4)]]
    );
}
