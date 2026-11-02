#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::{Address, byte_views::ByteProbe};
use std::{hint::black_box, time::Instant};
// Supplied lessons/benchmark input generation only. Not part of address lookup.
fn shown(value: u8) -> [bool; 8] {
    std::array::from_fn(|i| value & (1 << (7 - i)) != 0)
}
fn path(value: u8) -> [[u8; 1]; 8] {
    shown(value).map(|bit| [u8::from(bit)])
}
fn expected(i: usize) -> u8 {
    i.wrapping_mul(73).wrapping_add(19) as u8
}
fn main() {
    println!(
        "mode,addresses,repeat,model_setup_ns,input_setup_ns,query_ns,inspection_ns,input_allocation_requests,input_allocated_bytes,query_allocation_requests,query_allocated_bytes,copy_cycles"
    );
    for mode in ["path", "anchor", "path_emit"] {
        for n in [1usize, 8, 64, 512, 4096, 32768] {
            for repeat in 0..5 {
                let clock = Instant::now();
                let mut model = ByteProbe::new();
                for value in 0..=255 {
                    let input = model.load_bits(&shown(value));
                    model.start_lesson(&input, value, b"shown-byte-v1");
                    assert_eq!(model.execute(71), Ok(71));
                }
                let model_setup_ns = clock.elapsed().as_nanos();
                // Requests are complete addresses. In anchor mode the caller already supplies
                // native anchors: fixture resolution is accounted for here, not in query_ns.
                heap::start();
                let clock = Instant::now();
                let mut paths = Vec::new();
                let mut anchors: Vec<Address> = Vec::new();
                if mode == "anchor" {
                    anchors.reserve_exact(n);
                    for i in 0..n {
                        let tokens = path(expected(i));
                        anchors.push(
                            model
                                .representations
                                .root()
                                .locate(tokens.iter().map(|t| t.as_slice()))
                                .unwrap(),
                        );
                    }
                } else {
                    paths.reserve_exact(n);
                    for i in 0..n {
                        paths.push(path(expected(i)));
                    }
                }
                let input_setup_ns = clock.elapsed().as_nanos();
                let input_heap = heap::finish();
                // Opaque result collection only, not a semantic cache/model mirror. Reserved before timing.
                let mut results = Vec::with_capacity(n);
                let mut copy_cycles = 0;
                heap::start();
                let clock = Instant::now();
                for i in 0..n {
                    let view = if mode == "anchor" {
                        model.view_at(&anchors[i]).unwrap()
                    } else {
                        model
                            .locate_view(paths[i].iter().map(|t| t.as_slice()))
                            .unwrap()
                    };
                    if mode == "path_emit" {
                        model.start_view(&view);
                        let done = model.execute(1).unwrap();
                        assert_eq!(done, 1);
                        copy_cycles += done;
                        results.push(model.result.follow().unwrap());
                    } else {
                        results.push(view);
                    }
                }
                let query_ns = clock.elapsed().as_nanos();
                let query_heap = heap::finish();
                let clock = Instant::now();
                for (i, view) in results.iter().enumerate() {
                    assert_eq!(view.token(), Some([expected(i)].as_slice()));
                    let at = view.follow().unwrap();
                    assert!(model.view_at(&at).unwrap().same(view));
                    black_box(view);
                }
                let inspection_ns = clock.elapsed().as_nanos();
                assert_eq!(query_heap.requests, 0);
                assert_eq!(query_heap.bytes, 0);
                assert_eq!(copy_cycles, if mode == "path_emit" { n as u64 } else { 0 });
                println!(
                    "{mode},{n},{repeat},{model_setup_ns},{input_setup_ns},{query_ns},{inspection_ns},{},{},{},{},{copy_cycles}",
                    input_heap.requests, input_heap.bytes, query_heap.requests, query_heap.bytes
                );
            }
        }
    }
}
