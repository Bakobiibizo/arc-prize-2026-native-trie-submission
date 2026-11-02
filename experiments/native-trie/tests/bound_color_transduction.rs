use native_trie_seed::{Trie, episode_transduction::LocalEpisodeTransducer};

fn values(trie: &Trie, raw: &[u8]) -> Vec<native_trie_seed::Address> {
    raw.iter()
        .map(|value| trie.provision(&[b"colors", &[*value]]))
        .collect()
}
fn raw(values: &[native_trie_seed::Address]) -> Vec<u8> {
    values
        .iter()
        .map(|value| value.token().unwrap()[0])
        .collect()
}
fn example(
    trie: &Trie,
    north: u8,
    other: u8,
) -> (
    Vec<native_trie_seed::Address>,
    Vec<native_trie_seed::Address>,
) {
    let input = values(
        trie,
        &[other, north, other, other, 0, other, other, other, other],
    );
    let mut output = input.clone();
    output[4] = trie.provision(&[b"colors", &[north]]);
    (input, output)
}

#[test]
fn color_construction_binds_unique_neighbour_reference_across_palettes() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let (input_a, output_a) = example(&colors, 2, 1);
    let (input_b, output_b) = example(&colors, 7, 3);
    learner.teach_grid(3, 3, &input_a, &output_a).unwrap();
    learner.validate_grid(3, 3, &input_a, &output_a).unwrap();
    learner.teach_grid(3, 3, &input_b, &output_b).unwrap();
    learner.validate_grid(3, 3, &input_b, &output_b).unwrap();
    assert_eq!(
        raw(&learner.transduce_grid(3, 3, &input_a).unwrap()),
        raw(&output_a)
    );
    assert_eq!(
        raw(&learner.transduce_grid(3, 3, &input_b).unwrap()),
        raw(&output_b)
    );
}
