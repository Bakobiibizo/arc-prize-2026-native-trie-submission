#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Trie,
    arc_grids::{Grids, Side},
    region_witness::RegionWitness,
    singleton_witness::SingletonWitness,
};
use std::time::Instant;
fn main() {
    println!(
        "case,status,candidate,original_reference,role_copies,role_budget,region_copies,setup_ns,execute_ns,record_ns,setup_requests,setup_bytes,execute_allocations,proof_nodes"
    );
    let cases = vec![
        ("single", vec![vec![Some(8)]], (0, 0)),
        (
            "center",
            vec![
                vec![Some(0); 3],
                vec![Some(0), Some(4), Some(0)],
                vec![Some(0); 3],
            ],
            (1, 1),
        ),
        ("plural", vec![vec![Some(4), Some(4)]], (0, 0)),
        ("unknown_seed", vec![vec![None]], (0, 0)),
        ("escape_unknown", vec![vec![Some(7), None]], (0, 0)),
        (
            "bounded_unknown",
            vec![
                vec![Some(0), None, Some(0)],
                vec![Some(0), Some(5), Some(0)],
                vec![Some(0); 3],
            ],
            (1, 1),
        ),
    ];
    for (id, data, (y, x)) in cases {
        let s = Grids::new();
        let raw = format!("{data:?}");
        let source = s.source(id.as_bytes(), raw.as_bytes()).unwrap();
        let g = s
            .admit(id.as_bytes(), Side::QueryInput, &data, &source)
            .unwrap();
        let owner = Trie::default();
        let root = owner.root();
        let seed = g.cell(y, x).unwrap();
        let expected_ref = seed.child(b"value").unwrap().follow().unwrap();
        let mut region = RegionWitness::prepare_counted(
            &s,
            &g,
            &seed,
            &root,
            b"region",
            &root.provision(b"cache"),
        )
        .unwrap();
        let region_copies = region.execute(region.budget).unwrap();
        let region = region.record().unwrap();
        heap::start();
        let t = Instant::now();
        let mut role = SingletonWitness::prepare(region, &root, b"role").unwrap();
        let setup_ns = t.elapsed().as_nanos();
        let setup = heap::finish();
        let budget = role.budget;
        heap::start();
        let t = Instant::now();
        let copies = role.execute(budget).unwrap();
        let execute_ns = t.elapsed().as_nanos();
        let execution = heap::finish();
        assert_eq!(execution.requests, 0);
        let t = Instant::now();
        let p = role.record().unwrap();
        let record_ns = t.elapsed().as_nanos();
        let original = p.candidate.as_ref().is_some_and(|c| c.same(&expected_ref));
        println!(
            "{id},{},{},{original},{copies},{budget},{region_copies},{setup_ns},{execute_ns},{record_ns},{},{},{},{}",
            std::str::from_utf8(p.status.token().unwrap()).unwrap(),
            p.candidate.is_some(),
            setup.requests,
            setup.bytes,
            execution.requests,
            RegionWitness::owned_nodes(&p.proof)
        );
        assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
    }
}
