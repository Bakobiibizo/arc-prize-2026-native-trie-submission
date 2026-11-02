#[path = "../../support/word_fixture.rs"]
mod fixture;
#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::words::{WordResolution, Words};
use std::{hint::black_box, time::Instant};
fn main() {
    println!(
        "case,width,repeat,operations,setup_ns,setup_allocation_requests,setup_allocated_bytes,operation_ns,allocation_requests,allocated_bytes"
    );
    for case in ["path", "anchor", "replay", "evidence"] {
        for width in [1usize, 8, 64, 512] {
            for repeat in 0..5 {
                heap::start();
                let clock = Instant::now();
                let model = fixture::setup();
                let mut words = Words::new();
                // Synthetic supplied character sequence, not generated/recognized as a host word.
                let indices: Vec<_> = (0..width).map(|i| i % fixture::CHARACTERS.len()).collect();
                let events = fixture::events(&model, &indices, false);
                let characters: Vec<_> = events.iter().map(|e| e.character.clone()).collect();
                let source = model.source(b"original-span");
                let first = words.teach(&events, &source).unwrap();
                let endpoint = words.locate(&characters).unwrap().unwrap();
                let operations = if matches!(case, "path" | "anchor") {
                    4096
                } else {
                    100
                };
                let sources: Vec<_> = if case == "evidence" {
                    (0usize..operations)
                        .map(|i| model.source(&i.to_be_bytes()))
                        .collect()
                } else {
                    Vec::new()
                };
                let setup_ns = clock.elapsed().as_nanos();
                let setup_heap = heap::finish();
                heap::start();
                let clock = Instant::now();
                let mut source_iter = sources.iter();
                for _ in 0..operations {
                    match case {
                        "path" => {
                            let at = words.locate(&characters).unwrap().unwrap();
                            assert!(
                                matches!(words.resolve(&at),Ok(WordResolution::Complete { word,.. }) if word.same(&endpoint))
                            );
                            black_box(at);
                        }
                        "anchor" => assert!(
                            matches!(words.resolve(&endpoint),Ok(WordResolution::Complete { word,.. }) if word.same(&endpoint))
                        ),
                        "replay" => {
                            let r = words.teach(&events, &source).unwrap();
                            assert!(r.candidate.same(&first.candidate));
                            black_box(r);
                        }
                        "evidence" => {
                            let r = words.teach(&events, source_iter.next().unwrap()).unwrap();
                            assert!(r.candidate.same(&first.candidate));
                            black_box(r);
                        }
                        _ => unreachable!(),
                    }
                }
                let operation_ns = clock.elapsed().as_nanos();
                let sample = heap::finish();
                if case != "evidence" {
                    assert_eq!(sample.requests, 0);
                    assert_eq!(sample.bytes, 0);
                }
                let evidence =
                    native_trie_seed::records::inspect_evidence(&first.candidate).unwrap();
                assert_eq!(
                    evidence.len(),
                    if case == "evidence" {
                        operations + 1
                    } else {
                        1
                    }
                );
                println!(
                    "{case},{width},{repeat},{operations},{setup_ns},{},{},{operation_ns},{},{}",
                    setup_heap.requests, setup_heap.bytes, sample.requests, sample.bytes
                );
            }
        }
    }
}
