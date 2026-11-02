#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::byte_views::ByteProbe;
use std::{hint::black_box, time::Instant};

// Offline shown-pair/synthetic-input generation, not part of the running decoder.
fn shown(value: u8) -> [bool; 8] {
    std::array::from_fn(|i| value & (1 << (7 - i)) != 0)
}
fn main() {
    println!(
        "bytes,repeat,model_and_training_ns,input_load_ns,execution_ns,inspection_ns,training_cycles,read_cycles,allocation_requests,allocated_bytes,model_allocation_requests,model_allocated_bytes,input_allocation_requests,input_allocated_bytes"
    );
    for length in [1usize, 8, 64, 512, 4096, 32768] {
        for repeat in 0..5 {
            heap::start();
            let clock = Instant::now();
            let mut probe = ByteProbe::new();
            let mut training_cycles = 0;
            for value in 0..=255 {
                let input = probe.load_bits(&shown(value));
                probe.start_lesson(&input, value, b"shown-byte-v1");
                let done = probe.execute(100).expect("lesson must complete");
                assert_eq!(done, 71);
                assert!(probe.status_is(&probe.statuses.installed));
                training_cycles += done;
            }
            let model_and_training_ns = clock.elapsed().as_nanos();
            let model_heap = heap::finish();
            heap::start();
            let clock = Instant::now();
            let expected: Vec<u8> = (0..length)
                .map(|i| (i.wrapping_mul(73).wrapping_add(19)) as u8)
                .collect();
            let bits: Vec<_> = expected.iter().flat_map(|value| shown(*value)).collect();
            let input = probe.load_bits(&bits);
            probe.attach(&input);
            drop(input);
            drop(bits);
            let input_load_ns = clock.elapsed().as_nanos();
            let input_heap = heap::finish();
            let mut execution_ns = 0;
            let mut inspection_ns = 0;
            let mut read_cycles = 0;
            let mut allocation_requests = 0;
            let mut allocated_bytes = 0;
            for wanted in expected {
                heap::start();
                let clock = Instant::now();
                probe.start_read();
                let done = probe.execute(100);
                execution_ns += clock.elapsed().as_nanos();
                let sample = heap::finish();
                assert_eq!(done, Ok(67));
                read_cycles += done.unwrap();
                allocation_requests += sample.requests;
                allocated_bytes += sample.bytes;
                let clock = Instant::now();
                let found = probe.emitted_byte().expect("valid output status");
                assert_eq!(found, Some(wanted));
                black_box(found);
                inspection_ns += clock.elapsed().as_nanos();
            }
            heap::start();
            let clock = Instant::now();
            probe.start_read();
            let done = probe.execute(30);
            execution_ns += clock.elapsed().as_nanos();
            let sample = heap::finish();
            assert_eq!(done, Ok(10));
            read_cycles += done.unwrap();
            allocation_requests += sample.requests;
            allocated_bytes += sample.bytes;
            assert!(probe.status_is(&probe.statuses.eof));
            assert_eq!(probe.emitted_byte(), Ok(None));
            assert_eq!(allocation_requests, 0);
            assert_eq!(allocated_bytes, 0);
            println!(
                "{length},{repeat},{model_and_training_ns},{input_load_ns},{execution_ns},{inspection_ns},{training_cycles},{read_cycles},{allocation_requests},{allocated_bytes},{},{},{},{}",
                model_heap.requests, model_heap.bytes, input_heap.requests, input_heap.bytes
            );
        }
    }
}
