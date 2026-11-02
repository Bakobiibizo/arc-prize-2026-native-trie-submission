#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::{Address, Fault, Trie, reference_branch::ReferenceBranch};
use std::{fmt::Write, time::Instant};
fn hex(at: &Address) -> String {
    let mut s = String::new();
    for b in at.token().unwrap() {
        write!(s, "{b:02x}").unwrap()
    }
    s
}
fn name(at: &Address) -> &str {
    std::str::from_utf8(at.token().unwrap()).unwrap()
}
fn main() {
    println!(
        "pool,repeat,left,right,setup_ns,bind_ns,bind_requests,bind_bytes,execution_trace_ns,execution_requests,execution_bytes,copies,publication_ns,publication_requests,publication_bytes,inspection_ns,actual,expected"
    );
    for size in [1usize, 2, 8, 16] {
        for repeat in 0..3 {
            let start = Instant::now();
            let data = Trie::default();
            let content = data.provision(&[b"shared-content"]);
            let values: Vec<_> = (0..size)
                .map(|i| {
                    let at = data.provision(&[&i.to_be_bytes(), b"value"]);
                    match i % 3 {
                        0 => (),
                        1 => at.terminate(),
                        _ => at.point_to(&content),
                    }
                    at
                })
                .collect();
            let mut program = ReferenceBranch::new();
            let setup_ns = start.elapsed().as_nanos();
            for (i, left) in values.iter().enumerate() {
                for (j, right) in values.iter().enumerate() {
                    heap::start();
                    let start = Instant::now();
                    program.bind(left, right);
                    let bind_ns = start.elapsed().as_nanos();
                    let binding = heap::finish();
                    heap::start();
                    let start = Instant::now();
                    let copies = program.execute(6).unwrap();
                    let execution_trace_ns = start.elapsed().as_nanos();
                    let execution = heap::finish();
                    assert_eq!(execution.requests, 0);
                    assert_eq!(execution.bytes, 0);
                    assert_eq!(copies, 6);
                    heap::start();
                    let start = Instant::now();
                    let result = program.record().unwrap();
                    let publication_ns = start.elapsed().as_nanos();
                    let publication = heap::finish();
                    let start = Instant::now();
                    let actual = name(&result.value);
                    let expected = if left.same(right) {
                        "same"
                    } else {
                        "different"
                    };
                    assert_eq!(actual, expected);
                    for at in [left, right] {
                        let k = if at.same(left) { i } else { j };
                        match k % 3 {
                            0 => assert!(matches!(at.follow(), Err(Fault::Unset))),
                            1 => assert!(at.is_terminal()),
                            _ => assert!(at.follow().unwrap().same(&content)),
                        }
                    }
                    let mut events = Vec::new();
                    for k in 0..6usize {
                        let step = result
                            .evidence
                            .child(b"steps")
                            .unwrap()
                            .child(&k.to_be_bytes())
                            .unwrap();
                        let before = step.child(b"before").unwrap().follow().unwrap();
                        let after = step.child(b"after").unwrap().follow().unwrap();
                        let target = step.child(b"target").unwrap().follow().unwrap();
                        let next = step.child(b"next-cell").unwrap().follow().unwrap();
                        let copied = step.child(b"copied-reference").unwrap().follow().unwrap();
                        events.push(format!("{{\"before\":\"{}\",\"after\":\"{}\",\"target_token_hex\":\"{}\",\"next_token_hex\":\"{}\",\"target_is_next\":{},\"copied_token_hex\":\"{}\"}}",name(&before),name(&after),hex(&target),hex(&next),target.same(&next),hex(&copied)));
                    }
                    let inspection_ns = start.elapsed().as_nanos();
                    println!(
                        "{size},{repeat},{i},{j},{setup_ns},{bind_ns},{},{},{execution_trace_ns},{},{},{copies},{publication_ns},{},{},{inspection_ns},{actual},{expected}",
                        binding.requests,
                        binding.bytes,
                        execution.requests,
                        execution.bytes,
                        publication.requests,
                        publication.bytes
                    );
                    eprintln!(
                        "{{\"pool\":{size},\"repeat\":{repeat},\"left\":{i},\"right\":{j},\"actual\":\"{actual}\",\"events\":[{}]}}",
                        events.join(",")
                    );
                }
            }
        }
    }
}
