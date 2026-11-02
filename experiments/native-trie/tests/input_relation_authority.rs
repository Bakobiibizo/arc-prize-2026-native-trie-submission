//! Intentional post-learning mutations invalidate fit; execution must obey actual stored fields.
use native_trie_seed::{
    arc_grids::{Grids, Side},
    sparse_rewrite::SparseRewrite,
};
#[test]
fn stored_clause_source_scope_and_literal_slot_are_authoritative_with_history_preserved() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let src = s.source(b"s", b"authority fixture").unwrap();
    let i = s
        .admit(
            b"i",
            Side::TrainingInput,
            &[vec![Some(1), Some(0), Some(1)]],
            &src,
        )
        .unwrap();
    let o = s
        .admit(
            b"o",
            Side::TrainingOutput,
            &[vec![Some(1), Some(4), Some(1)]],
            &src,
        )
        .unwrap();
    let original = o
        .cell(0, 1)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let a = m.derive_input_relations(b"a", &i, &o).unwrap();
    let b = m.derive_input_relations(b"b", &i, &o).unwrap();
    let r = SparseRewrite::agree(&[a, b]).unwrap();
    assert_eq!(
        m.apply(&mut s, &i, &r).unwrap().inspect().unwrap()[0][1],
        Some(4)
    );
    let extra = s
        .admit(b"five", Side::TrainingInput, &[vec![Some(5)]], &src)
        .unwrap();
    let five = extra
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    r.root
        .child(b"literal-action-slot")
        .unwrap()
        .follow()
        .unwrap()
        .point_to(&five);
    assert_eq!(
        m.apply(&mut s, &i, &r).unwrap().inspect().unwrap()[0][1],
        Some(5)
    );
    let sign = r
        .root
        .child(b"relations")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .child(b"expected-sign")
        .unwrap();
    sign.point_to(&m.root.child(b"tags").unwrap().child(b"true").unwrap());
    assert_eq!(
        m.apply(&mut s, &i, &r).unwrap().inspect().unwrap()[0][1],
        Some(0)
    );
    sign.point_to(&m.root.child(b"tags").unwrap().child(b"false").unwrap());
    let one = i
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    r.root.child(b"source-colour").unwrap().point_to(&one);
    let q = s
        .admit(
            b"q",
            Side::QueryInput,
            &[vec![Some(0), Some(1), Some(0)]],
            &src,
        )
        .unwrap();
    assert_eq!(
        m.apply(&mut s, &q, &r).unwrap().inspect().unwrap(),
        vec![vec![Some(0), Some(5), Some(0)]]
    );
    let snapshot = m
        .root
        .child(b"queries")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .child(b"rows")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap()
        .child(b"cells")
        .unwrap()
        .child(&1usize.to_be_bytes())
        .unwrap()
        .child(b"source-value-after-copy")
        .unwrap()
        .follow()
        .unwrap();
    assert!(snapshot.same(&original));
}
