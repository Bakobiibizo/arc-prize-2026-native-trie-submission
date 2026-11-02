#[path = "../../support/live_heap.rs"]
mod heap;
use native_trie_seed::{
    Trie,
    arc_grids::{Grids, Side},
    region_witness::RegionWitness,
};
fn main() {
    let counted = std::env::args().any(|a| a == "--counted");
    println!(
        "size,input_cells,before_bytes,active_increment_bytes,retained_increment_bytes,activation_nodes,proof_nodes,recorded_views,copies,result,fixed_bank_bytes"
    );
    for n in [1usize, 2, 3, 5, 9, 15, 30] {
        let s = Grids::new();
        let source = s.source(b"source", b"controlled closed outline").unwrap();
        let data = (0..n)
            .map(|y| {
                (0..n)
                    .map(|x| {
                        Some(if x == 0 || y == 0 || x + 1 == n || y + 1 == n {
                            2
                        } else {
                            0
                        })
                    })
                    .collect()
            })
            .collect::<Vec<_>>();
        let g = s.admit(b"input", Side::QueryInput, &data, &source).unwrap();
        let owner = Trie::default();
        let root = owner.root();
        let cache = root.provision(b"cache");
        let bank_before = heap::live();
        if counted {
            RegionWitness::cardinality_zero(&root);
        }
        let fixed_bank_bytes = heap::live() - bank_before;
        let before = heap::live();
        let prepare = if counted {
            RegionWitness::prepare_counted
        } else {
            RegionWitness::prepare
        };
        let mut w = prepare(
            &s,
            &g,
            &g.cell(n / 2, n / 2).unwrap(),
            &root,
            b"proof",
            &cache,
        )
        .unwrap();
        let active = heap::live();
        let nodes = RegionWitness::owned_nodes(&w.activation_root());
        let copies = w.execute(w.budget).unwrap();
        let projection = w.record().unwrap();
        let retained = heap::live();
        let proof_nodes = RegionWitness::owned_nodes(&projection.proof);
        println!(
            "{n},{},{before},{},{},{nodes},{proof_nodes},{},{copies},{},{fixed_bank_bytes}",
            n * n,
            active - before,
            retained - before,
            projection.recorded_views,
            std::str::from_utf8(projection.value.token().unwrap()).unwrap()
        );
        assert!(retained - before < (active - before) / 3);
        assert_eq!(
            projection.value.token(),
            Some(if n < 3 {
                b"escape".as_slice()
            } else {
                b"bounded".as_slice()
            })
        );
    }
}
