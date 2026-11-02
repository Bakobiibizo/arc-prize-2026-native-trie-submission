#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::counter;

#[test]
fn seeded_counter_execution_requests_no_heap_memory() {
    for width in [1, 8, 64, 512, 4096, 32768] {
        let mut full = counter::prepare(width, true);
        heap::start();
        full.pulse(width + 3);
        let full_sample = heap::finish();
        assert_eq!(
            full_sample.requests, 0,
            "full carry width={width}, {full_sample:?}"
        );
        assert_eq!(full_sample.bytes, 0);
        let mut short = counter::prepare(width, false);
        heap::start();
        short.pulse(4);
        let short_sample = heap::finish();
        assert_eq!(
            short_sample.requests, 0,
            "short carry width={width}, {short_sample:?}"
        );
        assert_eq!(short_sample.bytes, 0);
    }
    let mut repeated = counter::prepare(8, false);
    heap::start();
    for _ in 0..256 {
        repeated.pulse(11);
    }
    let sample = heap::finish();
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
}
