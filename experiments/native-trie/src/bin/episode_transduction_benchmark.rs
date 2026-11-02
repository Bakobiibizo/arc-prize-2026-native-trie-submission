use native_trie_seed::{Trie, episode_transduction::LocalEpisodeTransducer};
use std::time::Instant;

fn values(trie: &Trie, row: &[u8]) -> Vec<native_trie_seed::Address> {
    row.iter()
        .map(|value| trie.provision(&[b"colors", &[*value]]))
        .collect()
}

fn main() {
    for size in [1_001usize, 2_001, 4_001, 8_001, 16_001, 32_001] {
        let colors = Trie::default();
        let mut learner = LocalEpisodeTransducer::default();
        let training_input = values(&colors, &[0, 1, 0, 1, 0]);
        let training_output = values(&colors, &[0, 1, 6, 1, 0]);
        let teach_started = Instant::now();
        learner.teach(&training_input, &training_output).unwrap();
        let teach_ns = teach_started.elapsed().as_nanos();
        let query = (0..size)
            .map(|index| if index % 2 == 1 { 1 } else { 0 })
            .collect::<Vec<_>>();
        let query = values(&colors, &query);
        let transduce_started = Instant::now();
        let result = learner.transduce(&query).unwrap();
        let transduce_ns = transduce_started.elapsed().as_nanos();
        let changed = result
            .iter()
            .zip(&query)
            .filter(|(a, b)| !a.same(b))
            .count();
        assert_eq!(changed, (size - 3) / 2);
        println!(
            "{{\"cells\":{size},\"teach_ns\":{teach_ns},\"transduce_ns\":{transduce_ns},\"applications\":{},\"copy_cycles\":{}}}",
            learner.applications, learner.copy_cycles
        );
    }
}
