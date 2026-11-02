#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::character_prefixes::*;
use std::{hint::black_box, time::Instant};
// Offline input fixture/oracle only. No scalar numbers enter the runtime model.
fn path(value: u32) -> ScalarPath {
    std::array::from_fn(|i| [((value >> (20 - i)) & 1) as u8])
}
fn main() {
    println!(
        "phase,cases,accepted,rejected,elapsed_ns,copy_cycles,allocation_requests,allocated_bytes,interned_values"
    );
    let mut model = CharacterIdentity::new();
    let mut accepted = 0;
    let mut rejected = 0;
    heap::start();
    let clock = Instant::now();
    for value in 0..(1 << 21) {
        let found = model.select(&path(value));
        let expected = value <= 0x10FFFF && !(0xD800..=0xDFFF).contains(&value);
        match found {
            Ok(found) => {
                assert!(expected);
                assert!(found.construction.same(&model.model.construction));
                accepted += 1;
                black_box(found);
            }
            Err(CharacterError::Unrecorded) => {
                assert!(!expected);
                rejected += 1;
            }
            Err(error) => panic!("unexpected {error:?}"),
        }
    }
    let ns = clock.elapsed().as_nanos();
    let heap = heap::finish();
    assert_eq!(accepted, 1_112_064);
    assert_eq!(rejected, 985_088);
    assert_eq!(heap.requests, 0);
    println!(
        "selection,2097152,{accepted},{rejected},{ns},0,{},{},{}",
        heap.requests,
        heap.bytes,
        model.interned_values()
    );
    // All valid input data are interned in the actual value trie: no output/answer entry per scalar.
    let mut cases = 0;
    let mut copies = 0;
    heap::start();
    let clock = Instant::now();
    for value in 0..=0x10FFFF {
        if (0xD800..=0xDFFF).contains(&value) {
            continue;
        }
        let input = path(value);
        let output = model.construct(&input).expect("valid scalar construction");
        let label: [u8; 21] = std::array::from_fn(|i| input[i][0]);
        assert_eq!(output.value.token(), Some(label.as_slice()));
        assert!(output.construction.same(&model.model.construction));
        assert_eq!(model.model.machine.cycles, 1);
        copies += model.model.machine.cycles;
        cases += 1;
        black_box(output);
    }
    let ns = clock.elapsed().as_nanos();
    let heap = heap::finish();
    assert_eq!(cases, 1_112_064);
    assert_eq!(copies, cases);
    assert_eq!(model.interned_values(), cases as usize);
    println!(
        "construct,{cases},{cases},0,{ns},{copies},{},{},{}",
        heap.requests,
        heap.bytes,
        model.interned_values()
    );
    let input = path(0x43);
    let expected = model.intern_value(&input).unwrap();
    copies = 0;
    heap::start();
    let clock = Instant::now();
    for _ in 0..65536 {
        let output = model.construct(&input).unwrap();
        assert!(output.value.same(&expected));
        copies += model.model.machine.cycles;
        black_box(output);
    }
    let ns = clock.elapsed().as_nanos();
    let heap = heap::finish();
    assert_eq!(copies, 65536);
    assert_eq!(heap.requests, 0);
    assert_eq!(heap.bytes, 0);
    println!(
        "repeat,65536,65536,0,{ns},{copies},{},{},{}",
        heap.requests,
        heap.bytes,
        model.interned_values()
    );
}
