use native_trie_seed::{Trie, episode_transduction::LocalEpisodeTransducer};

fn values(trie: &Trie, raw: &[u8]) -> Vec<native_trie_seed::Address> {
    raw.iter()
        .map(|value| trie.provision(&[b"colors", &[*value]]))
        .collect()
}
fn scene(
    trie: &Trie,
    width: usize,
    source_x: usize,
    moved: bool,
) -> Vec<native_trie_seed::Address> {
    let mut raw = vec![0; 5 * width];
    let y = 2;
    raw[y * width + source_x + 2] = 9;
    raw[y * width + source_x + usize::from(moved)] = 2;
    values(trie, &raw)
}

#[test]
fn paired_graphs_induce_position_redirect_and_native_move() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let input = scene(&colors, 5, 2, false);
    let output = scene(&colors, 5, 2, true);
    assert_eq!(learner.teach_grid(5, 5, &input, &output).unwrap(), 1);
    learner.validate_grid(5, 5, &input, &output).unwrap();

    let query = scene(&colors, 7, 3, false);
    let expected = scene(&colors, 7, 3, true);
    let result = learner.transduce_grid(5, 7, &query).unwrap();
    assert!(result.iter().zip(&expected).all(|(a, b)| a.same(b)));
}
