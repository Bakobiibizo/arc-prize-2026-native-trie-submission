use native_trie_seed::{
    arc_grids::{Grids, Side},
    sparse_rewrite::SparseRewrite,
};
#[test]
fn one_native_counterexample_rejects_without_claiming_unchecked_negatives_passed() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let src = s.source(b"s", b"counterexample stop fixture").unwrap();
    let input = vec![vec![Some(0); 3]; 3];
    let mut output = input.clone();
    for (y, x) in [(0, 0), (0, 2), (2, 0), (2, 2)] {
        output[y][x] = Some(4);
    }
    let i = s.admit(b"i", Side::TrainingInput, &input, &src).unwrap();
    let o = s.admit(b"o", Side::TrainingOutput, &output, &src).unwrap();
    let d = m.derive_input_relations(b"d", &i, &o).unwrap();
    assert!(d.rule.is_none());
    assert_eq!(
        (
            d.changed,
            d.positive_tests,
            d.negative_checks,
            d.negative_unchecked
        ),
        (4, 40, 1, 4)
    );
    let unchanged = d.evidence.child(b"unchanged").unwrap();
    let cs = native_trie_seed::records::inspect_candidates(&unchanged, b"source-colour");
    assert_eq!(cs.len(), 1);
    assert_eq!(
        native_trie_seed::records::inspect_evidence(&cs[0])
            .unwrap()
            .len(),
        5
    );
    assert!(
        d.evidence
            .child(b"negative-validation-short-circuited")
            .unwrap()
            .is_terminal()
    );
    assert!(
        d.evidence
            .child(b"negative-unchecked")
            .unwrap()
            .child(&4usize.to_be_bytes())
            .is_some()
    );
    assert!(
        d.evidence
            .child(b"counterexamples")
            .unwrap()
            .child(b"holds")
            .is_some()
    );
}
