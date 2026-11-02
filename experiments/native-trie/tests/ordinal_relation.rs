use native_trie_seed::{
    Address, Trie,
    ordinal_relation::{OrdinalBank, OrdinalInput},
    records,
    reference_set_count::SetCountWitness,
    region_witness::RegionWitness,
    source_field,
};
use std::rc::Rc;
fn count(root: &Address, id: &[u8], n: usize, unknown: bool) -> Rc<OrdinalInput> {
    let set = root.provision(b"input-sets").provision(id);
    for i in 0..n {
        let value = set.provision(b"values").provision(&i.to_be_bytes());
        records::record(&set, b"reference", &value, &value).unwrap();
    }
    let missing = root.provision(b"set-count-tags").provision(b"unavailable");
    let slot = set.provision(b"availability");
    slot.point_to(if unknown { &missing } else { &set });
    let mut w = SetCountWitness::prepare(
        set,
        b"reference",
        &[(slot, missing.clone())],
        root,
        id,
        Vec::new(),
    )
    .unwrap();
    w.execute(w.budget).unwrap();
    OrdinalInput::from_set(w.record().unwrap(), missing)
}
fn run(
    bank: &mut OrdinalBank,
    a: Rc<OrdinalInput>,
    b: Rc<OrdinalInput>,
    id: &[u8],
) -> native_trie_seed::ordinal_relation::OrdinalProjection {
    let mut w = bank.compare(a, b, id).unwrap();
    w.execute(w.budget).unwrap();
    w.record().unwrap()
}
#[test]
fn native_order_boundary_matrix_and_constructor_invocation_reservations() {
    let root = Trie::default().root();
    let inputs: Vec<_> = [0usize, 1, 899, 900]
        .iter()
        .map(|n| count(&root, &n.to_be_bytes(), *n, false))
        .collect();
    let zero = RegionWitness::cardinality_zero(&root);
    for (j, b) in inputs.iter().enumerate() {
        let before = RegionWitness::storage_charge(&root);
        let mut bank = OrdinalBank::prepare(&zero, b.clone(), &root, &j.to_be_bytes()).unwrap();
        let growth = RegionWitness::storage_charge(&root) - before;
        assert!(growth < OrdinalBank::reserve_bytes());
        println!(
            "ordinal-bank target_index={j} constructor_proxy={growth} reserve={}",
            OrdinalBank::reserve_bytes()
        );
        for (i, a) in inputs.iter().enumerate() {
            let before = RegionWitness::storage_charge(&root);
            let p = run(&mut bank, a.clone(), b.clone(), &i.to_be_bytes());
            let growth = RegionWitness::storage_charge(&root) - before;
            assert!(growth < OrdinalBank::invocation_reserve_bytes());
            assert!(p.value.same(&if i < j {
                bank.greater_tag()
            } else {
                bank.not_greater_tag()
            }));
            assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
            println!(
                "ordinal-comparison a_index={i} b_index={j} native_copies={} invocation_proxy={growth}",
                p.copies
            );
        }
    }
    assert!(RegionWitness::storage_charge(&root) < 286720000);
}
#[test]
fn unknown_and_foreign_bank_are_unavailable_not_false() {
    let root = Trie::default().root();
    let known = count(&root, b"k", 1, false);
    let unknown = count(&root, b"u", 0, true);
    let zero = RegionWitness::cardinality_zero(&root);
    let mut b = OrdinalBank::prepare(&zero, known.clone(), &root, b"known").unwrap();
    let p = run(&mut b, unknown.clone(), known.clone(), b"a");
    assert!(p.value.same(&b.unavailable_tag()));
    let mut b = OrdinalBank::prepare(&zero, unknown.clone(), &root, b"unknown").unwrap();
    let p = run(&mut b, known.clone(), unknown, b"b");
    assert!(p.value.same(&b.unavailable_tag()));
    let foreign = Trie::default().root();
    let other = count(&foreign, b"f", 1, false);
    let mut b = OrdinalBank::prepare(&zero, known.clone(), &root, b"foreign").unwrap();
    let p = run(&mut b, other, known, b"c");
    assert!(p.value.same(&b.unavailable_tag()));
}
#[test]
fn native_instruction_and_actual_operand_bindings_are_authoritative() {
    let root = Trie::default().root();
    let a = count(&root, b"a", 0, false);
    let b = count(&root, b"b", 1, false);
    let two = count(&root, b"two", 2, false);
    let zero = RegionWitness::cardinality_zero(&root);
    let mut bank = OrdinalBank::prepare(&zero, b.clone(), &root, b"bank").unwrap();
    let old = run(&mut bank, a.clone(), b.clone(), b"old");
    assert!(old.value.same(&bank.greater_tag()));
    let state = bank
        .root
        .child(b"construction")
        .unwrap()
        .child(b"greater-return")
        .unwrap();
    let no = bank.not_greater_tag();
    let replacement = bank.root.provision(b"mutated-source");
    replacement.point_to(&no);
    source_field(&state).unwrap().point_to(&replacement);
    let changed = run(&mut bank, a.clone(), b.clone(), b"new");
    assert!(changed.value.same(&bank.not_greater_tag()));
    assert!(old.value.same(&bank.greater_tag()));
    b.proof
        .child(b"value")
        .unwrap()
        .point_to(&two.current().unwrap());
    assert!(bank.compare(a, b, b"stale-binding").is_err());
}
#[test]
fn dropped_partial_and_faulted_witness_poison_shared_activation() {
    for fault in [false, true] {
        let root = Trie::default().root();
        let a = count(&root, b"a", 0, false);
        let b = count(&root, b"b", 1, false);
        let zero = RegionWitness::cardinality_zero(&root);
        let mut bank = OrdinalBank::prepare(&zero, b.clone(), &root, b"bank").unwrap();
        let proof;
        {
            let mut w = bank.compare(a.clone(), b.clone(), b"partial").unwrap();
            proof = w.root.clone();
            if fault {
                let state = w.machine.current().unwrap();
                source_field(&state)
                    .unwrap()
                    .point_to(&proof.provision(b"unset"));
                assert!(w.execute(w.budget).is_err());
            } else {
                assert!(w.execute(1).is_err());
            }
            assert!(w.record().is_err());
        }
        assert!(bank.compare(a, b, b"forbidden-reuse").is_err());
        assert!(proof.child(b"committed").is_none());
        assert!(proof.child(b"attempt-copies").is_some());
        assert!(bank.root.child(b"poisoned-by").is_some());
    }
}
#[test]
fn canonical_domain_must_belong_to_retained_parent_before_any_allocation() {
    let root = Trie::default().root();
    let target = count(&root, b"t", 1, false);
    let zero = RegionWitness::cardinality_zero(&root);
    let other = Trie::default().root();
    assert!(matches!(
        OrdinalBank::prepare(&zero, target.clone(), &other, b"bad"),
        Err(native_trie_seed::region_witness::RegionError::Grid(
            native_trie_seed::arc_grids::GridError::ForeignSpace
        ))
    ));
    assert!(other.child(b"ordinal-relation-banks").is_none());
    let mut relative =
        native_trie_seed::relative_ordinal_count::RelativeOrdinalCount::new(&other, &zero);
    assert!(
        relative
            .prepare(target, &[], &[], b"bad", |_| Ok(()))
            .is_err()
    );
    assert!(relative.root.child(b"proofs").is_none());
}
#[test]
fn completed_proof_keeps_native_owners_alive_after_caller_drop() {
    let p = {
        let root = Trie::default().root();
        let a = count(&root, b"a", 0, false);
        let b = count(&root, b"b", 1, false);
        let zero = RegionWitness::cardinality_zero(&root);
        let mut bank = OrdinalBank::prepare(&zero, b.clone(), &root, b"bank").unwrap();
        run(&mut bank, a, b, b"proof")
    };
    assert_eq!(p.value.token(), Some(b"greater".as_slice()));
    assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
}
