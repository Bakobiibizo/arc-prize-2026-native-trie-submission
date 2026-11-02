#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::{Trie, records::*};
use std::{hint::black_box, time::Instant};
fn main() {
    println!(
        "case,n,repeat,fixture_ns,fixture_allocation_requests,fixture_allocated_bytes,operation_ns,allocation_requests,allocated_bytes,candidates,evidence"
    );
    for case in [
        "new",
        "evidence",
        "repeat",
        "unique",
        "ambiguous",
        "unknown",
    ] {
        for n in [1usize, 8, 64, 512, 4096, 32768] {
            for repeat in 0..5 {
                heap::start();
                let clock = Instant::now();
                let data = Trie::default();
                let root = data.root();
                let target = root.provision(b"target");
                let initial_source = root.provision(b"initial-source");
                let anchor_count = if case == "new" { n } else { 1 };
                let anchors: Vec<_> = (0..anchor_count)
                    .map(|i| root.provision(b"anchors").provision(&i.to_be_bytes()))
                    .collect();
                let source_count = if matches!(case, "evidence" | "unique" | "new") {
                    n
                } else {
                    0
                };
                let sources: Vec<_> = (0..source_count)
                    .map(|i| {
                        root.provision(b"supplied-evidence")
                            .provision(&i.to_be_bytes())
                    })
                    .collect();
                let mut targets = Vec::new();
                match case {
                    "repeat" | "evidence" => {
                        record(&anchors[0], b"view", &target, &initial_source).unwrap();
                    }
                    "unique" => {
                        for source in &sources {
                            record(&anchors[0], b"view", &target, source).unwrap();
                        }
                    }
                    "ambiguous" => {
                        for i in 0..n.max(2) {
                            let at = root.provision(b"targets").provision(&i.to_be_bytes());
                            record(&anchors[0], b"view", &at, &initial_source).unwrap();
                            targets.push(at);
                        }
                    }
                    _ => {}
                }
                let fixture_ns = clock.elapsed().as_nanos();
                let fixture_heap = heap::finish();
                heap::start();
                let clock = Instant::now();
                for i in 0..n {
                    match case {
                        "new" => {
                            let r = record(&anchors[i], b"view", &target, &sources[i]).unwrap();
                            assert_eq!(r.change, Change::NewTarget);
                            black_box(r);
                        }
                        "evidence" => {
                            let r = record(&anchors[0], b"view", &target, &sources[i]).unwrap();
                            assert_eq!(r.change, Change::AddedEvidence);
                            black_box(r);
                        }
                        "repeat" => {
                            let r = record(&anchors[0], b"view", &target, &initial_source).unwrap();
                            assert_eq!(r.change, Change::AlreadyRecorded);
                            black_box(r);
                        }
                        "unique" => match resolve(&anchors[0], b"view").unwrap() {
                            Selection::Unique { target: found, .. } => assert!(found.same(&target)),
                            _ => panic!("unique expected"),
                        },
                        "ambiguous" => assert!(
                            matches!(resolve(&anchors[0],b"view"),Ok(Selection::Ambiguous { alternatives }) if alternatives==n.max(2))
                        ),
                        "unknown" => assert!(matches!(
                            resolve(&anchors[0], b"view"),
                            Ok(Selection::Unknown)
                        )),
                        _ => unreachable!(),
                    }
                }
                let operation_ns = clock.elapsed().as_nanos();
                let sample = heap::finish();
                if matches!(case, "repeat" | "unique" | "ambiguous" | "unknown") {
                    assert_eq!(sample.requests, 0);
                    assert_eq!(sample.bytes, 0);
                }
                // Inspection only: cardinality validation never participates in choosing a stored target.
                let mut candidates = 0;
                let mut evidence = 0;
                for anchor in &anchors {
                    for candidate in inspect_candidates(anchor, b"view") {
                        let value = candidate.follow().unwrap();
                        candidates += 1;
                        if case != "ambiguous" {
                            assert!(value.same(&target));
                        }
                        let entries = inspect_evidence(&candidate).unwrap();
                        evidence += entries.len();
                        for entry in entries {
                            assert!(entry.follow().is_ok());
                            assert!(
                                entry
                                    .child(b"anchor")
                                    .unwrap()
                                    .follow()
                                    .unwrap()
                                    .same(anchor)
                            );
                        }
                    }
                }
                let expected = match case {
                    "new" => (n, n),
                    "evidence" => (1, n + 1),
                    "repeat" => (1, 1),
                    "unique" => (1, n),
                    "ambiguous" => (n.max(2), n.max(2)),
                    "unknown" => (0, 0),
                    _ => unreachable!(),
                };
                assert_eq!((candidates, evidence), expected);
                println!(
                    "{case},{n},{repeat},{fixture_ns},{},{},{operation_ns},{},{},{candidates},{evidence}",
                    fixture_heap.requests, fixture_heap.bytes, sample.requests, sample.bytes
                );
            }
        }
    }
}
