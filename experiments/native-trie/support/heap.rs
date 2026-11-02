//! Allocation-request probe, scoped to the executing thread. Not part of trie semantics.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

#[derive(Clone, Copy, Default, Debug)]
pub struct Sample {
    pub requests: u64,
    pub bytes: u64,
}
thread_local! {
    static PROBE:Cell<(bool,Sample)>=const { Cell::new((false,Sample { requests:0,bytes:0 })) };
}
struct ObservedSystem;
#[global_allocator]
static HEAP: ObservedSystem = ObservedSystem;
fn account(bytes: usize) {
    let _ = PROBE.try_with(|probe| {
        let (active, mut sample) = probe.get();
        if active {
            sample.requests += 1;
            sample.bytes += bytes as u64;
            probe.set((true, sample));
        }
    });
}
unsafe impl GlobalAlloc for ObservedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        account(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        account(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, old: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        account(size);
        unsafe { System.realloc(old, layout, size) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}
pub fn start() {
    PROBE.with(|probe| probe.set((true, Sample::default())));
}
pub fn finish() -> Sample {
    PROBE.with(|probe| {
        let (_, sample) = probe.replace((false, Sample::default()));
        sample
    })
}
