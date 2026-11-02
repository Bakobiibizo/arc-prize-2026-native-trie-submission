use native_trie_seed::{
    Trie,
    episode_transduction::{LocalEpisodeTransducer, TransductionError},
};

fn values(trie: &Trie, row: &[u8]) -> Vec<native_trie_seed::Address> {
    row.iter()
        .map(|value| trie.provision(&[b"colors", &[*value]]))
        .collect()
}
fn inspect(row: &[native_trie_seed::Address]) -> Vec<u8> {
    row.iter().map(|value| value.token().unwrap()[0]).collect()
}

#[test]
fn paired_examples_induce_and_apply_blue_gap_pink_construction() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let input = values(&colors, &[0, 1, 0, 1, 0]);
    let output = values(&colors, &[0, 1, 6, 1, 0]);
    assert_eq!(learner.teach(&input, &output).unwrap(), 1);

    // A second translated occurrence reaches the same context/construction addresses and adds
    // evidence rather than creating a construction family.
    let input2 = values(&colors, &[0, 0, 1, 0, 1, 0]);
    let output2 = values(&colors, &[0, 0, 1, 6, 1, 0]);
    learner.validate(&input2, &output2).unwrap();
    let wrong2 = values(&colors, &[0, 0, 1, 4, 1, 0]);
    assert_eq!(
        learner.validate(&input2, &wrong2),
        Err(TransductionError::ValidationMismatch)
    );
    assert_eq!(learner.teach(&input2, &output2).unwrap(), 1);

    let query = values(&colors, &[0, 1, 0, 1, 0, 1, 0]);
    let result = learner.transduce(&query).unwrap();
    assert_eq!(inspect(&result), vec![0, 1, 6, 1, 6, 1, 0]);
}
