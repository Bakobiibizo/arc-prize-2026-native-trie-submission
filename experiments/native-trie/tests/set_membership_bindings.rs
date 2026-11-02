use native_trie_seed::{Trie, records, reference_set_membership::SetMembershipWitness};
#[test]
fn stale_expected_set_and_guard_bindings_or_native_fault_never_commit() {
    for case in 0..4 {
        let root = Trie::default().root();
        let set = root.provision(b"set");
        let a = root.provision(b"a");
        let b = root.provision(b"b");
        records::record(&set, b"reference", &a, &a).unwrap();
        let expected = root.provision(b"expected");
        expected.point_to(&a);
        let unknown = root.provision(b"unknown");
        let scope = root.provision(b"scope");
        scope.point_to(&set);
        let mut w = SetMembershipWitness::prepare(
            set.clone(),
            b"reference",
            &expected,
            &[(scope.clone(), unknown.clone())],
            &root,
            b"p",
            Vec::new(),
        )
        .unwrap();
        let proof = w.root.clone();
        match case {
            0 => expected.point_to(&b),
            1 => {
                records::record(&set, b"reference", &b, &b).unwrap();
            }
            2 => scope.point_to(&unknown),
            _ => w.machine.pointer().terminate(),
        };
        assert!(w.execute(w.budget).is_err());
        assert!(w.record().is_err());
        assert!(proof.child(b"committed").is_none());
        assert!(proof.child(b"attempt-copies").is_some());
    }
}
#[test]
fn post_execution_binding_change_preserves_native_value_but_refuses_certificate() {
    let root = Trie::default().root();
    let a = root.provision(b"a");
    let set = root.provision(b"set");
    records::record(&set, b"reference", &a, &a).unwrap();
    let expected = root.provision(b"expected");
    expected.point_to(&a);
    let mut w =
        SetMembershipWitness::prepare(set, b"reference", &expected, &[], &root, b"p", Vec::new())
            .unwrap();
    let proof = w.root.clone();
    w.execute(w.budget).unwrap();
    expected.point_to(&root.provision(b"b"));
    assert!(w.record().is_err());
    assert!(proof.child(b"committed").is_none());
    assert_eq!(
        proof.child(b"value").unwrap().follow().unwrap().token(),
        Some(b"present".as_slice())
    );
}
