#[path = "../../support/alias_fixture.rs"]
mod fixture;
#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::representations::Resolution;
use std::{hint::black_box, time::Instant};
fn main() {
    println!(
        "mode,n,repeat,setup_ns,query_ns,input_allocation_requests,input_allocated_bytes,query_allocation_requests,query_allocated_bytes,copy_cycles"
    );
    for mode in ["path", "anchor", "emit"] {
        for n in [1usize, 8, 64, 512, 4096, 32768] {
            for repeat in 0..5 {
                let clock = Instant::now();
                let mut model = fixture::teach();
                let setup_ns = clock.elapsed().as_nanos();
                heap::start();
                let mut paths = Vec::new();
                let mut anchors = Vec::new();
                if mode == "anchor" {
                    anchors.reserve_exact(n);
                    for i in 0..n {
                        anchors.push(
                            model
                                .root()
                                .locate(
                                    fixture::LESSONS[i % fixture::LESSONS.len()]
                                        .0
                                        .iter()
                                        .copied(),
                                )
                                .unwrap(),
                        );
                    }
                } else {
                    paths.reserve_exact(n);
                    for i in 0..n {
                        paths.push(fixture::LESSONS[i % fixture::LESSONS.len()].0);
                    }
                }
                let input_heap = heap::finish();
                let mut results = Vec::with_capacity(n);
                let mut copy_cycles = 0;
                heap::start();
                let clock = Instant::now();
                for i in 0..n {
                    let resolved = if mode == "anchor" {
                        model.at(&anchors[i])
                    } else {
                        model.lookup(paths[i].iter().copied())
                    };
                    let character = match resolved.unwrap() {
                        Resolution::Complete { character, .. } => character,
                        _ => panic!("complete expected"),
                    };
                    if mode == "emit" {
                        copy_cycles += model.emit(&character).unwrap();
                        results.push(model.result.follow().unwrap());
                    } else {
                        results.push(character);
                    }
                }
                let query_ns = clock.elapsed().as_nanos();
                let query_heap = heap::finish();
                for (i, target) in results.iter().enumerate() {
                    let (path, id) = fixture::LESSONS[i % fixture::LESSONS.len()];
                    assert_eq!(target.token(), Some(id));
                    assert!(target.same(&fixture::target(&model, path)));
                    black_box(target);
                }
                assert_eq!(query_heap.requests, 0);
                assert_eq!(query_heap.bytes, 0);
                assert_eq!(copy_cycles, if mode == "emit" { n as u64 } else { 0 });
                println!(
                    "{mode},{n},{repeat},{setup_ns},{query_ns},{},{},{},{},{copy_cycles}",
                    input_heap.requests, input_heap.bytes, query_heap.requests, query_heap.bytes
                );
            }
        }
    }
}
