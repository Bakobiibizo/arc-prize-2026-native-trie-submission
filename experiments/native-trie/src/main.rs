#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{Outcome, counter};
use std::{hint::black_box, time::Instant};

fn main() {
    println!(
        "case,width,repeat,load_ns,execution_ns,inspect_ns,cycles,allocation_requests,allocated_bytes"
    );
    for width in [1, 8, 64, 512, 4096, 32768] {
        for full_carry in [true, false] {
            for repeat in 0..5 {
                let start = Instant::now();
                let mut counter = counter::prepare(width, full_carry);
                let load_ns = start.elapsed().as_nanos();
                heap::start();
                let start = Instant::now();
                let outcome = counter.pulse(width + 3);
                let execution_ns = start.elapsed().as_nanos();
                let sample = heap::finish();
                assert_eq!(outcome, Outcome::Halted);
                assert_eq!(
                    counter.machine.cycles,
                    if full_carry { width as u64 + 1 } else { 2 }
                );
                assert_eq!(sample.requests, 0);
                assert_eq!(sample.bytes, 0);
                let start = Instant::now();
                let bits = counter.inspect();
                if full_carry {
                    assert!(bits.iter().all(|value| !*value));
                } else {
                    assert!(bits[0]);
                    assert!(bits[1..].iter().all(|value| !*value));
                }
                black_box(&bits);
                let inspect_ns = start.elapsed().as_nanos();
                println!(
                    "{},{width},{repeat},{load_ns},{execution_ns},{inspect_ns},{},{},{}",
                    if full_carry { "carry" } else { "short" },
                    counter.machine.cycles,
                    sample.requests,
                    sample.bytes
                );
            }
        }
    }
}
