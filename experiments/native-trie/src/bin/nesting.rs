#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::constructions::Builder;
use std::{hint::black_box, time::Instant};

fn main() {
    println!(
        "case,size,repeat,load_ns,execution_ns,inspection_ns,cycles,output_bytes,allocation_requests,allocated_bytes"
    );
    for case in ["left", "right", "empty", "literal"] {
        let sizes: &[usize] = if case == "literal" {
            &[1, 8, 64, 512, 4096, 32768]
        } else {
            &[0, 1, 8, 64, 512, 4096]
        };
        for &size in sizes {
            for repeat in 0..5 {
                let clock = Instant::now();
                let mut b = Builder::new();
                let mut root = if case == "literal" {
                    b.literal(&vec![b'a'; size])
                } else if case == "empty" {
                    b.literal(b"")
                } else {
                    b.literal(b"bob")
                };
                if case != "literal" {
                    for _ in 0..size {
                        let operand = b.literal(if case == "empty" { b"" } else { b"!" });
                        root = if case == "right" {
                            b.join(operand, root)
                        } else {
                            b.join(root, operand)
                        };
                    }
                }
                let mut evaluation = b.finish(root);
                let load_ns = clock.elapsed().as_nanos();
                let expected = match case {
                    "literal" => 2 * size + 10,
                    "empty" => 53 * size + 10,
                    _ => 55 * size + 16,
                };
                heap::start();
                let clock = Instant::now();
                let cycles = evaluation.execute(expected + 1);
                let execution_ns = clock.elapsed().as_nanos();
                let memory = heap::finish();
                assert_eq!(cycles, Ok(expected as u64));
                assert_eq!(memory.requests, 0);
                assert_eq!(memory.bytes, 0);
                let clock = Instant::now();
                let output = evaluation
                    .inspect(2 * size + 8)
                    .expect("valid constructed output");
                match case {
                    "literal" => assert_eq!(output, vec![b'a'; size]),
                    "empty" => assert!(output.is_empty()),
                    "left" => {
                        assert_eq!(output.len(), size + 3);
                        assert_eq!(&output[..3], b"bob");
                        assert!(output[3..].iter().all(|b| *b == b'!'));
                    }
                    "right" => {
                        assert_eq!(output.len(), size + 3);
                        assert_eq!(&output[size..], b"bob");
                        assert!(output[..size].iter().all(|b| *b == b'!'));
                    }
                    _ => unreachable!(),
                }
                black_box(&output);
                let inspection_ns = clock.elapsed().as_nanos();
                println!(
                    "{case},{size},{repeat},{load_ns},{execution_ns},{inspection_ns},{},{},{},{}",
                    evaluation.machine.cycles,
                    output.len(),
                    memory.requests,
                    memory.bytes
                );
            }
        }
    }
}
