use native_trie_seed::{
    Address, Trie, records, reference_set_count::SetCountWitness, region_witness::RegionWitness,
};
fn ordinal(root: &Address, n: usize) -> Address {
    let mut at = root
        .child(b"cardinalities")
        .unwrap()
        .child(b"zero")
        .unwrap();
    for _ in 0..n {
        at = at.child(b"successor").unwrap();
    }
    at
}
#[test]
fn counts_distinct_physical_references_not_occurrences_or_payloads() {
    let root = Trie::default().root();
    let set = root.provision(b"set");
    let a = root.provision(b"a");
    let b = root.provision(b"b");
    let c = root.provision(b"c");
    for (i, value) in [&a, &b, &c, &a, &b].iter().enumerate() {
        records::record(
            &set,
            b"reference",
            value,
            &root.provision(b"evidence").provision(&i.to_be_bytes()),
        )
        .unwrap();
    }
    let reserve = SetCountWitness::reserve_bytes(&set, b"reference", 0, &root);
    let before = RegionWitness::storage_charge(&root);
    let mut w =
        SetCountWitness::prepare(set, b"reference", &[], &root, b"count", Vec::new()).unwrap();
    assert!(RegionWitness::storage_charge(&root) - before < reserve);
    assert_eq!(w.execute(w.budget).unwrap(), 8);
    let p = w.record().unwrap();
    assert!(p.value.same(&ordinal(&root, 3)));
    assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
}
#[test]
fn empty_known_is_zero_but_unknown_guard_is_unavailable() {
    for unknown in [false, true] {
        let root = Trie::default().root();
        let set = root.provision(b"set");
        let missing = root.provision(b"missing");
        let slot = root.provision(b"input");
        slot.point_to(if unknown { &missing } else { &set });
        let mut w = SetCountWitness::prepare(
            set.clone(),
            b"reference",
            &[(slot, missing)],
            &root,
            b"count",
            Vec::new(),
        )
        .unwrap();
        w.execute(w.budget).unwrap();
        let p = w.record().unwrap();
        if unknown {
            assert_eq!(p.value.token(), Some(b"unavailable".as_slice()));
        } else {
            assert!(p.value.same(&ordinal(&root, 0)));
        }
    }
}
#[test]
fn incomplete_native_counter_does_not_publish_fact() {
    let root = Trie::default().root();
    let set = root.provision(b"set");
    let mut w =
        SetCountWitness::prepare(set, b"reference", &[], &root, b"count", Vec::new()).unwrap();
    let proof = w.root.clone();
    assert!(w.execute(0).is_err());
    assert!(w.record().is_err());
    assert!(proof.child(b"committed").is_none());
    assert!(proof.child(b"execution").is_some());
    assert!(proof.child(b"attempt-copies").is_some());
    assert!(proof.child(b"value").unwrap().follow().is_err());
}
#[test]
fn malformed_reference_and_faulted_control_are_not_count_facts() {
    let root = Trie::default().root();
    let set = root.provision(b"set");
    set.provision(b"holds")
        .provision(b"reference")
        .provision(b"candidates")
        .provision(b"bad");
    assert!(SetCountWitness::prepare(set, b"reference", &[], &root, b"bad", Vec::new()).is_err());
    let mut w = SetCountWitness::prepare(
        root.provision(b"empty"),
        b"reference",
        &[],
        &root,
        b"fault",
        Vec::new(),
    )
    .unwrap();
    let proof = w.root.clone();
    let current = w.machine.current().unwrap();
    native_trie_seed::source_field(&current)
        .unwrap()
        .point_to(&proof.provision(b"unset-source"));
    assert!(w.execute(w.budget).is_err());
    assert!(w.record().is_err());
    assert!(proof.child(b"committed").is_none());
    assert!(proof.child(b"attempt-copies").is_some());
}
#[test]
fn maximum_native_set_respects_reservation() {
    let root = Trie::default().root();
    let set = root.provision(b"set");
    for i in 0..900usize {
        let value = root.provision(b"members").provision(&i.to_be_bytes());
        records::record(&set, b"reference", &value, &value).unwrap();
    }
    let reserve = SetCountWitness::reserve_bytes(&set, b"reference", 0, &root);
    let before = RegionWitness::storage_charge(&root);
    let mut w =
        SetCountWitness::prepare(set, b"reference", &[], &root, b"count", Vec::new()).unwrap();
    assert!(RegionWitness::storage_charge(&root) - before < reserve);
    w.execute(w.budget).unwrap();
    assert!(w.record().unwrap().value.same(&ordinal(&root, 900)));
}
