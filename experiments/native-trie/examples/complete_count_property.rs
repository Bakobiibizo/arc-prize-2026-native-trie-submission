//! Declared fixtures/import/export only. No ARC expected outputs or host count/class solver.
use native_trie_seed::{
    Trie,
    arc_grids::{Grids, Side},
    complete_count_witness::CompleteCountWitness,
    graph_export,
    region_witness::RegionWitness,
};
fn main() {
    let s = Grids::new();
    let root = Trie::default().root();
    let mut horizontal = vec![vec![Some(1); 6]; 4];
    for row in horizontal.iter_mut().take(3).skip(1) {
        for c in row.iter_mut().take(5).skip(1) {
            *c = Some(0);
        }
    }
    let mut vertical = vec![vec![Some(2); 4]; 6];
    for row in vertical.iter_mut().take(5).skip(1) {
        for c in row.iter_mut().take(3).skip(1) {
            *c = Some(0);
        }
    }
    let mut unknown = horizontal.clone();
    unknown[1][2] = None;
    let cases = [
        (b"horizontal".as_slice(), horizontal, (1, 1)),
        (b"vertical".as_slice(), vertical, (1, 1)),
        (b"bounded-unknown".as_slice(), unknown, (1, 1)),
        (
            b"escape-unknown".as_slice(),
            vec![vec![Some(0), None], vec![Some(0), Some(0)]],
            (0, 0),
        ),
        (b"seed-unknown".as_slice(), vec![vec![None]], (0, 0)),
        (b"maximum".as_slice(), vec![vec![Some(0); 30]; 30], (0, 0)),
    ];
    let mut keep = Vec::new();
    for (id, pixels, seed) in cases {
        let source = s
            .source(
                id,
                b"complete-count-property declared fixture geometry, no expected value imported",
            )
            .unwrap();
        let input = s.admit(id, Side::TrainingInput, &pixels, &source).unwrap();
        let n = pixels.len() * pixels[0].len();
        // Standalone root+active-constructor proxy, NOT the five-root ARC model/RSS.
        assert!(RegionWitness::storage_charge(&root) + 512000 + (n + 1) * 38000 <= 286720000);
        let mut w = RegionWitness::prepare_counted(
            &s,
            &input,
            &input.cell(seed.0, seed.1).unwrap(),
            &root,
            id,
            &root.provision(b"cache").provision(id),
        )
        .unwrap();
        let region_copies = w.execute(w.budget).unwrap();
        let region = w.record().unwrap();
        let reserve = CompleteCountWitness::reserve_bytes(&region).unwrap();
        let before = RegionWitness::storage_charge(&root);
        assert!(before + reserve <= 286720000);
        let mut c = CompleteCountWitness::prepare(region, &root, id).unwrap();
        let after = RegionWitness::storage_charge(&root);
        assert!(after - before < reserve);
        let copies = c.execute(c.budget).unwrap();
        let p = c.record().unwrap();
        // EXPORT decoder only: walk the canonical unary bank to serialize an ALREADY
        // native-computed result. The witness/learners consume its reference, not this integer.
        let mut at = root
            .child(b"cardinalities")
            .unwrap()
            .child(b"zero")
            .unwrap();
        let mut ordinal = None;
        for i in 0..=900 {
            if at.same(&p.value) {
                ordinal = Some(i);
                break;
            }
            match at.child(b"successor") {
                Some(next) => at = next,
                None => break,
            }
        }
        let value = ordinal.map_or_else(|| "null".to_owned(), |n| n.to_string());
        println!(
            "{{\"kind\":\"complete-count-property\",\"id\":\"{}\",\"status\":\"{}\",\"exported_unary_ordinal\":{},\"region_copies\":{},\"property_copies\":{},\"constructor_reserve\":{},\"constructor_charge_growth\":{},\"standalone_root_charge\":{}}}",
            String::from_utf8_lossy(id),
            String::from_utf8_lossy(p.status.token().unwrap()),
            value,
            region_copies,
            copies,
            reserve,
            after - before,
            RegionWitness::storage_charge(&root)
        );
        keep.push(p);
    }
    graph_export::write(&[root], &mut std::io::stderr()).unwrap();
    drop(keep);
}
