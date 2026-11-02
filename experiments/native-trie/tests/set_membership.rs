use native_trie_seed::{
    Trie, records, reference_set_membership::SetMembershipWitness, region_witness::RegionWitness,
    source_field,
};
#[test]
fn empty_present_distinct_reference_and_unavailable_are_native_results() {
    for case in 0..4 {
        let root = Trie::default().root();
        let set = root.provision(b"set");
        let a = root.provision(b"a");
        let b = root.provision(b"b");
        let payload = root.provision(b"same-payload");
        a.point_to(&payload);
        b.point_to(&payload);
        let slot = root.provision(b"expected");
        slot.point_to(if case == 2 { &b } else { &a });
        if case != 0 {
            records::record(&set, b"reference", &a, &a).unwrap();
            records::record(&set, b"reference", &a, &b).unwrap();
        }
        let unknown = root.provision(b"unknown");
        let guard = root.provision(b"scope");
        guard.point_to(if case == 3 { &unknown } else { &set });
        let reserve = SetMembershipWitness::reserve_bytes(&set, b"reference", 1);
        let before = RegionWitness::storage_charge(&root);
        let mut w = SetMembershipWitness::prepare(
            set,
            b"reference",
            &slot,
            &[(guard, unknown)],
            &root,
            b"p",
            Vec::new(),
        )
        .unwrap();
        assert!(RegionWitness::storage_charge(&root) - before < reserve);
        w.execute(w.budget).unwrap();
        let p = w.record().unwrap();
        assert_eq!(
            p.value.token(),
            Some(match case {
                1 => b"present".as_slice(),
                3 => b"unavailable",
                _ => b"absent",
            })
        );
        assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
    }
}
#[test]
fn actual_native_source_mutation_and_partial_states_remain_authoritative() {
    for partial in [false, true] {
        let root = Trie::default().root();
        let set = root.provision(b"set");
        let a = root.provision(b"a");
        records::record(&set, b"reference", &a, &a).unwrap();
        let slot = root.provision(b"expected");
        slot.point_to(&a);
        let mut w =
            SetMembershipWitness::prepare(set, b"reference", &slot, &[], &root, b"p", Vec::new())
                .unwrap();
        let proof = w.root.clone();
        if partial {
            assert!(w.execute(0).is_err());
            assert!(w.record().is_err());
            assert!(proof.child(b"committed").is_none());
        } else {
            let present = proof
                .child(b"execution")
                .unwrap()
                .child(b"construction")
                .unwrap()
                .child(b"present")
                .unwrap();
            let source = root.provision(b"replacement");
            source.point_to(&root.provision(b"set-membership-tags").provision(b"absent"));
            source_field(&present).unwrap().point_to(&source);
            w.execute(w.budget).unwrap();
            assert_eq!(
                w.record().unwrap().value.token(),
                Some(b"absent".as_slice())
            );
        }
    }
}
#[test]
fn nine_hundred_member_reservation_and_lifetime() {
    let p = {
        let root = Trie::default().root();
        let set = root.provision(b"set");
        for i in 0..900usize {
            let r = root.provision(b"refs").provision(&i.to_be_bytes());
            records::record(&set, b"reference", &r, &r).unwrap();
        }
        let expected = root.provision(b"expected");
        expected.point_to(&root.provision(b"missing"));
        let reserve = SetMembershipWitness::reserve_bytes(&set, b"reference", 0);
        let before = RegionWitness::storage_charge(&root);
        let mut w = SetMembershipWitness::prepare(
            set,
            b"reference",
            &expected,
            &[],
            &root,
            b"p",
            Vec::new(),
        )
        .unwrap();
        assert!(RegionWitness::storage_charge(&root) - before < reserve);
        assert_eq!(w.execute(w.budget).unwrap(), 1801);
        w.record().unwrap()
    };
    assert_eq!(p.value.token(), Some(b"absent".as_slice()));
    assert_eq!(RegionWitness::dangling_links(&p.proof), 0);
}
