#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Trie,
    arc_grids::{Grids, Side},
    region_witness::RegionWitness,
};
use std::{fs, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    for raw in fs::read_to_string(&args[1]).unwrap().lines() {
        let f: Vec<_> = raw.split('\t').collect();
        assert_eq!(f.len(), 4);
        let id = f[0];
        let y: usize = f[1].parse().unwrap();
        let x: usize = f[2].parse().unwrap();
        let data: Vec<Vec<Option<u8>>> = f[3]
            .split('/')
            .map(|r| {
                r.split(',')
                    .map(|v| {
                        if v == "?" {
                            None
                        } else {
                            Some(v.parse().unwrap())
                        }
                    })
                    .collect()
            })
            .collect();
        let start = Instant::now();
        let s = Grids::new();
        let source = s.source(id.as_bytes(), raw.as_bytes()).unwrap();
        let input = s
            .admit(id.as_bytes(), Side::QueryInput, &data, &source)
            .unwrap();
        let input_ns = start.elapsed().as_nanos();
        let storage = Trie::default();
        let parent = storage.root();
        let cache = parent.provision(b"cache");
        heap::start();
        let start = Instant::now();
        let mut w = RegionWitness::prepare(
            &s,
            &input,
            &input.cell(y, x).unwrap(),
            &parent,
            b"proof",
            &cache,
        )
        .unwrap();
        let prepare_ns = start.elapsed().as_nanos();
        let prep = heap::finish();
        heap::start();
        let start = Instant::now();
        let copies = w.execute(w.budget).unwrap();
        let execute_ns = start.elapsed().as_nanos();
        let exec = heap::finish();
        assert_eq!(exec.requests, 0);
        assert_eq!(exec.bytes, 0);
        let start = Instant::now();
        let (members, checked, expanded) = w.inspect().unwrap();
        let exhausted = w
            .root
            .child(b"frontier-exhausted")
            .unwrap()
            .follow()
            .is_ok();
        let mut facts = Vec::new();
        for yy in 0..input.height() {
            for xx in 0..input.width() {
                let source = input.cell(yy, xx).unwrap();
                let view = w.view(&source).unwrap();
                let class = view.child(b"class").unwrap().follow().ok();
                let name = class
                    .as_ref()
                    .and_then(|a| a.token())
                    .and_then(|b| std::str::from_utf8(b).ok())
                    .unwrap_or("unvisited");
                let edges = [b"north".as_slice(), b"west", b"south", b"east"].map(|p| {
                    view.child(b"edges")
                        .unwrap()
                        .child(p)
                        .unwrap()
                        .child(b"checked")
                        .unwrap()
                        .follow()
                        .is_ok()
                });
                facts.push(format!(
                    "{{\"y\":{yy},\"x\":{xx},\"class\":\"{name}\",\"checked\":{:?}}}",
                    edges
                ));
            }
        }
        let inspect_ns = start.elapsed().as_nanos();
        let proof = w.record().unwrap();
        let result = std::str::from_utf8(proof.value.token().unwrap()).unwrap();
        println!(
            "{{\"id\":\"{id}\",\"result\":\"{result}\",\"copies\":{copies},\"members\":{members},\"checked\":{checked},\"expanded\":{expanded},\"exhausted\":{exhausted},\"input_ns\":{input_ns},\"prepare_ns\":{prepare_ns},\"prepare_requests\":{},\"prepare_bytes\":{},\"execute_ns\":{execute_ns},\"execution_allocations\":{},\"inspect_ns\":{inspect_ns}}}",
            prep.requests, prep.bytes, exec.requests
        );
        eprintln!("{{\"id\":\"{id}\",\"facts\":[{}]}}", facts.join(","));
    }
}
