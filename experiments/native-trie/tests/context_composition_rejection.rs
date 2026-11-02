use native_trie_seed::{
    arc_grids::{Grids, Side},
    sparse_rewrite::SparseRewrite,
};
#[test]
fn same_component_different_labels_cannot_be_repaired_by_component_constant_context() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let src = s
        .source(
            b"input",
            b"declared contradictory component-uniform action fixture",
        )
        .unwrap();
    let i = s
        .admit(
            b"i",
            Side::TrainingInput,
            &[
                vec![Some(6), Some(6), Some(6), Some(6)],
                vec![Some(6), Some(9), Some(9), Some(6)],
                vec![Some(6), Some(6), Some(6), Some(6)],
            ],
            &src,
        )
        .unwrap();
    let o = s
        .admit(
            b"o",
            Side::TrainingOutput,
            &[
                vec![Some(6), Some(6), Some(6), Some(6)],
                vec![Some(6), Some(7), Some(9), Some(6)],
                vec![Some(6), Some(6), Some(6), Some(6)],
            ],
            &src,
        )
        .unwrap();
    let d = m.derive_groups(b"d", &i, &o).unwrap();
    assert!(m.agree_context_composition(&[d]).unwrap().is_none());
    assert!(
        m.root
            .child(b"context-action-assemblies")
            .unwrap()
            .child(&0usize.to_be_bytes())
            .unwrap()
            .child(b"source-prefixes")
            .is_some()
    );
}
