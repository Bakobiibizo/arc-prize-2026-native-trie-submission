#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Address, Trie,
    words::{CharacterEvent, WordResolution, Words},
};
use std::time::Instant;
fn main() {
    println!(
        "mode,n,repeat,setup_ns,input_ns,operation_ns,correct,allocation_requests,allocated_bytes"
    );
    for mode in ["path", "endpoint"] {
        for n in [1024usize, 8192, 32768] {
            for repeat in 0..5 {
                let start = Instant::now();
                let data = Trie::default();
                let mut words = Words::new();
                let mut corpus = Vec::new();
                // Explicit fixture boundaries/character identities. No runtime splitting or decoding.
                for line in include_str!("../../curriculum/language-comparison.tsv").lines() {
                    let mut fields = line.split('\t');
                    let known = fields.next().unwrap() == "known";
                    let text = fields.next().unwrap();
                    let ids = fields.next().unwrap();
                    let mut events = Vec::new();
                    for id in ids.split(' ') {
                        let character = data
                            .root()
                            .provision(b"characters")
                            .provision(id.as_bytes());
                        let origin = data.root().provision(b"origins").provision(id.as_bytes());
                        origin.point_to(&character);
                        events.push(CharacterEvent { character, origin });
                    }
                    if known {
                        let source = data
                            .root()
                            .provision(b"boundaries")
                            .provision(text.as_bytes());
                        words.teach(&events, &source).unwrap();
                    }
                    corpus.push((
                        known,
                        events
                            .iter()
                            .map(|e| e.character.clone())
                            .collect::<Vec<Address>>(),
                    ));
                }
                let setup_ns = start.elapsed().as_nanos();
                let start = Instant::now();
                let endpoints: Vec<_> = corpus
                    .iter()
                    .map(|(_, chars)| words.locate(chars).unwrap())
                    .collect();
                let input_ns = start.elapsed().as_nanos();
                for i in 0..256 {
                    let index = i % corpus.len();
                    let (_, chars) = &corpus[index];
                    let at = if mode == "path" {
                        words.locate(chars).unwrap()
                    } else {
                        endpoints[index].clone()
                    };
                    std::hint::black_box(at.map(|at| words.resolve(&at).unwrap()));
                }
                heap::start();
                let start = Instant::now();
                let mut correct = 0;
                for i in 0..n {
                    let index = i % corpus.len();
                    let (expected, characters) = &corpus[index];
                    let at = if mode == "path" {
                        words.locate(characters).unwrap()
                    } else {
                        endpoints[index].clone()
                    };
                    let found = at.is_some_and(|at| {
                        matches!(words.resolve(&at).unwrap(), WordResolution::Complete { .. })
                    });
                    assert_eq!(found, *expected);
                    correct += 1;
                }
                let operation_ns = start.elapsed().as_nanos();
                let sample = heap::finish();
                assert_eq!(sample.requests, 0);
                assert_eq!(sample.bytes, 0);
                println!(
                    "{mode},{n},{repeat},{setup_ns},{input_ns},{operation_ns},{correct},{},{}",
                    sample.requests, sample.bytes
                );
            }
        }
    }
}
