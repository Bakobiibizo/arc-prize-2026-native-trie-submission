//! Local native proof acquisition for docs/native/NATIVE-ORDINAL-BANK-PROTOTYPE.md.
//! Supplied fixture topology, no learned ARC rule or decoded-count comparison.
use native_trie_seed::{
    Address, Trie,
    arc_grids::{Grids, Side},
    complete_count_witness::CompleteCountWitness,
    graph_export,
    ordinal_relation::{OrdinalBank, OrdinalInput},
    records,
    reference_set_count::SetCountWitness,
    region_witness::{RegionError, RegionWitness},
    relative_ordinal_count::{OrdinalCandidate, RelativeOrdinalCount},
};
use std::rc::Rc;
fn guard(root: &Address, n: usize) -> Result<(), RegionError> {
    if RegionWitness::storage_charge(root) + n > 286720000 {
        Err(RegionError::Limit)
    } else {
        Ok(())
    }
}
fn input(root: &Address, id: &[u8], n: usize) -> Rc<OrdinalInput> {
    let set = root.provision(b"supplied-fixture-sets").provision(id);
    for i in 0..n {
        let v = set.provision(b"references").provision(&i.to_be_bytes());
        records::record(&set, b"reference", &v, &v).unwrap();
    }
    guard(
        root,
        SetCountWitness::reserve_bytes(&set, b"reference", 0, root),
    )
    .unwrap();
    let mut w = SetCountWitness::prepare(set, b"reference", &[], root, id, Vec::new()).unwrap();
    w.execute(w.budget).unwrap();
    OrdinalInput::from_set(
        w.record().unwrap(),
        root.provision(b"set-count-tags").provision(b"unavailable"),
    )
}
fn main() {
    let boundary = Trie::default().root();
    let fixtures = [0usize, 1, 899, 900];
    let inputs: Vec<_> = fixtures
        .iter()
        .map(|n| input(&boundary, &n.to_be_bytes(), *n))
        .collect();
    let zero = RegionWitness::cardinality_zero(&boundary);
    for (j, b) in inputs.iter().enumerate() {
        guard(&boundary, OrdinalBank::reserve_bytes()).unwrap();
        let before = RegionWitness::storage_charge(&boundary);
        let mut bank = OrdinalBank::prepare(&zero, b.clone(), &boundary, &j.to_be_bytes()).unwrap();
        let growth = RegionWitness::storage_charge(&boundary) - before;
        println!(
            "{{\"kind\":\"bank-cost\",\"target_fixture\":{},\"constructor_proxy\":{},\"reserve\":{}}}",
            fixtures[j],
            growth,
            OrdinalBank::reserve_bytes()
        );
        for (i, a) in inputs.iter().enumerate() {
            guard(&boundary, OrdinalBank::invocation_reserve_bytes()).unwrap();
            let before = RegionWitness::storage_charge(&boundary);
            let mut w = bank
                .compare(a.clone(), b.clone(), &i.to_be_bytes())
                .unwrap();
            w.execute(w.budget).unwrap();
            let p = w.record().unwrap();
            let report = boundary
                .provision(b"fixture-reports")
                .provision(&j.to_be_bytes())
                .provision(&i.to_be_bytes());
            report.provision(b"proof").point_to(&p.proof);
            println!(
                "{{\"kind\":\"native-comparison\",\"a_fixture\":{},\"b_fixture\":{},\"tag\":\"{}\",\"copies\":{},\"invocation_proxy\":{}}}",
                fixtures[i],
                fixtures[j],
                std::str::from_utf8(p.value.token().unwrap()).unwrap(),
                p.copies,
                RegionWitness::storage_charge(&boundary) - before
            );
        }
    }
    let mut roots = vec![boundary];
    let mut retained = Vec::new();
    for (case, sizes, colour, bg) in [
        (0usize, [8usize, 5, 4, 2], 9u8, 6u8),
        (1, [14usize, 10, 9, 6], 4u8, 0u8),
    ] {
        let s = Grids::new();
        let root = Trie::default().root();
        let mut pixels = vec![vec![Some(bg); 24]; 20];
        for (j, size) in sizes.iter().enumerate() {
            for x in 0..*size {
                pixels[2 + j * 4][2 + x] = Some(colour);
            }
        }
        let src = s
            .source(
                b"fixture",
                b"declared independent native region/relative ordinal fixture",
            )
            .unwrap();
        let grid = s
            .admit(b"input", Side::TrainingInput, &pixels, &src)
            .unwrap();
        let mut cs = Vec::new();
        for j in 0..4usize {
            let mut w = RegionWitness::prepare_counted(
                &s,
                &grid,
                &grid.cell(2 + j * 4, 2).unwrap(),
                &root,
                &j.to_be_bytes(),
                &root.provision(b"cache"),
            )
            .unwrap();
            w.execute(w.budget).unwrap();
            let p = w.record().unwrap();
            let reference = p.proof.clone();
            let mut w = CompleteCountWitness::prepare(p, &root, &j.to_be_bytes()).unwrap();
            w.execute(w.budget).unwrap();
            cs.push(OrdinalCandidate {
                reference,
                count: OrdinalInput::from_complete(
                    w.record().unwrap(),
                    root.provision(b"complete-count-tags")
                        .provision(b"unavailable"),
                ),
            });
        }
        let zero = RegionWitness::cardinality_zero(&root);
        let mut f = RelativeOrdinalCount::new(&root, &zero);
        for j in 0..4usize {
            let p = f
                .prepare(cs[j].count.clone(), &cs, &[], &j.to_be_bytes(), |n| {
                    guard(&root, n)
                })
                .unwrap();
            let r = root
                .provision(b"fixture-reports")
                .provision(&j.to_be_bytes());
            r.provision(b"rank-count-proof").point_to(&p.count().proof);
            r.provision(b"subject").point_to(&cs[j].reference);
            retained.push(OrdinalInput::from_relative(
                p,
                root.provision(b"set-count-tags").provision(b"unavailable"),
            ));
        }
        println!(
            "{{\"kind\":\"native-relative-cost\",\"case\":{case},\"subjects\":4,\"banks\":{},\"comparison_copies\":{},\"count_copies\":{},\"final_proxy\":{}}}",
            f.bank_count(),
            f.comparison_copies,
            f.count_copies,
            RegionWitness::storage_charge(&root)
        );
        roots.push(root);
    }
    graph_export::write(&roots, &mut std::io::stderr()).unwrap();
    drop(retained);
}
