use native_trie_seed::{
    Address, Fault, Trie,
    arc_grids::{Grids, Side},
    reference_branch::{BranchError, BranchProjection, ReferenceBranch},
    source_field,
};
fn run(p: &mut ReferenceBranch, x: &Address, y: &Address) -> BranchProjection {
    p.bind(x, y);
    assert_eq!(p.execute(6).unwrap(), 6);
    let r = p.record().unwrap();
    assert_eq!(r.cycles, 6);
    r
}
#[test]
fn both_paths_and_orders_are_emitted_by_stored_code_in_six_copies() {
    let data = Trie::default();
    let x = data.provision(&[b"x"]);
    let y = data.provision(&[b"y"]);
    let mut p = ReferenceBranch::new();
    for (a, b, same) in [
        (&x, &x, true),
        (&x, &y, false),
        (&y, &x, false),
        (&y, &y, true),
    ] {
        let result = run(&mut p, a, b);
        assert!(result.value.same(&if same {
            p.same_tag()
        } else {
            p.different_tag()
        }));
    }
}
#[test]
fn equal_labels_contents_and_terminals_do_not_merge_distinct_references() {
    let data = Trie::default();
    let a = data.provision(&[b"left", b"same-label"]);
    let b = data.provision(&[b"right", b"same-label"]);
    let value = data.provision(&[b"shared-content"]);
    let mut p = ReferenceBranch::new();
    assert!(run(&mut p, &a, &b).value.same(&p.different_tag()));
    a.point_to(&value);
    b.point_to(&value);
    assert!(run(&mut p, &a, &b).value.same(&p.different_tag()));
    a.terminate();
    b.terminate();
    assert!(run(&mut p, &a, &b).value.same(&p.different_tag()));
    assert!(run(&mut p, &a, &a).value.same(&p.same_tag()));
}
#[test]
fn actual_arc_alias_values_bind_without_a_palette_decoder() {
    let grids = Grids::new();
    let source = grids.source(b"literal", b"fixture1,1,6").unwrap();
    let g = grids
        .admit(
            b"input",
            Side::QueryInput,
            &[vec![Some(1), Some(1), Some(6)]],
            &source,
        )
        .unwrap();
    let a = g
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let b = g
        .cell(0, 1)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let c = g
        .cell(0, 2)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let mut p = ReferenceBranch::new();
    assert!(run(&mut p, &a, &b).value.same(&p.same_tag()));
    assert!(run(&mut p, &a, &c).value.same(&p.different_tag()));
}
#[test]
fn private_views_never_mutate_supplied_graph_data() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let b = data.provision(&[b"b"]);
    let original = data.provision(&[b"original"]);
    a.point_to(&original);
    b.terminate();
    let mut p = ReferenceBranch::new();
    let first = run(&mut p, &a, &a);
    let second = run(&mut p, &a, &b);
    assert!(a.follow().unwrap().same(&original));
    assert!(b.is_terminal());
    assert!(first.value.same(&p.same_tag()));
    assert!(second.value.same(&p.different_tag()));
    let view = first
        .evidence
        .child(b"steps")
        .unwrap()
        .child(&4usize.to_be_bytes())
        .unwrap()
        .child(b"target")
        .unwrap()
        .follow()
        .unwrap();
    assert!(!view.same(&a));
    assert!(
        first
            .evidence
            .child(b"input-x")
            .unwrap()
            .follow()
            .unwrap()
            .same(&a)
    );
}
#[test]
fn fifth_step_follows_the_actual_post_write_alias_not_a_host_test() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let b = data.provision(&[b"b"]);
    let mut p = ReferenceBranch::new();
    for (right, same) in [(&a, true), (&b, false)] {
        let r = run(&mut p, &a, right);
        let step = r
            .evidence
            .child(b"steps")
            .unwrap()
            .child(&4usize.to_be_bytes())
            .unwrap();
        let target = step.child(b"target").unwrap().follow().unwrap();
        let next = step.child(b"next-cell").unwrap().follow().unwrap();
        assert_eq!(target.same(&next), same);
        let reached = step.child(b"after").unwrap().follow().unwrap();
        assert!(
            reached.same(
                &p.construction
                    .child(if same { b"same" } else { b"different" })
                    .unwrap()
            )
        );
        let emitted = r
            .evidence
            .child(b"steps")
            .unwrap()
            .child(&5usize.to_be_bytes())
            .unwrap()
            .child(b"copied-reference")
            .unwrap()
            .follow()
            .unwrap();
        assert!(emitted.same(&r.value));
    }
}
#[test]
fn five_copy_budget_is_incomplete_sixth_can_resume_and_publish() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let mut p = ReferenceBranch::new();
    p.bind(&a, &a);
    assert!(matches!(p.execute(0), Err(BranchError::Limit)));
    assert!(matches!(p.record(), Err(BranchError::NotReady)));
    assert!(matches!(p.execute(5), Err(BranchError::Limit)));
    assert!(matches!(p.record(), Err(BranchError::NotReady)));
    assert_eq!(p.execute(1).unwrap(), 6);
    assert!(p.record().unwrap().value.same(&p.same_tag()));
    assert!(matches!(p.record(), Err(BranchError::NotReady)));
    assert!(matches!(p.execute(6), Err(BranchError::NotReady)));
}
#[test]
fn repeated_invocations_have_private_arenas_and_immutable_result_snapshots() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let b = data.provision(&[b"b"]);
    let mut p = ReferenceBranch::new();
    let first = run(&mut p, &a, &a);
    for i in 0..100 {
        let expected = i % 2 == 0;
        let r = run(&mut p, &a, if expected { &a } else { &b });
        assert!(r.value.same(&if expected {
            p.same_tag()
        } else {
            p.different_tag()
        }));
        assert!(!r.evidence.same(&first.evidence));
    }
    assert!(first.value.same(&p.same_tag()));
    assert!(
        first
            .evidence
            .child(b"emitted")
            .unwrap()
            .follow()
            .unwrap()
            .same(&first.value)
    );
}
#[test]
fn rebinding_an_incomplete_run_never_reuses_old_private_data() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let b = data.provision(&[b"b"]);
    let mut p = ReferenceBranch::new();
    let old = p.bind(&a, &a);
    assert!(matches!(p.execute(5), Err(BranchError::Limit)));
    let next = p.bind(&a, &b);
    assert!(!old.same(&next));
    assert!(old.child(b"emitted").is_none());
    p.execute(6).unwrap();
    assert!(p.record().unwrap().value.same(&p.different_tag()));
}
#[test]
fn stored_continuation_mutation_controls_the_answer() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let mut p = ReferenceBranch::new();
    let root = p.machine.states.root();
    root.child(b"same-continuation")
        .unwrap()
        .point_to(&p.construction.child(b"different").unwrap());
    let r = run(&mut p, &a, &a);
    assert!(r.value.same(&p.different_tag()));
}
#[test]
fn fault_is_preserved_and_cannot_be_retried_over_same_trace() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let mut p = ReferenceBranch::new();
    let unset = p.machine.states.root().provision(b"unset-source");
    source_field(&p.construction.child(b"write-x-follow-y").unwrap())
        .unwrap()
        .point_to(&unset);
    let frame = p.bind(&a, &a);
    assert!(matches!(
        p.execute(6),
        Err(BranchError::Machine(Fault::Unset))
    ));
    assert!(matches!(p.record(), Err(BranchError::NotReady)));
    assert!(matches!(p.execute(6), Err(BranchError::NotReady)));
    let fault = frame
        .child(b"steps")
        .unwrap()
        .child(&4usize.to_be_bytes())
        .unwrap()
        .child(b"copied-read-fault")
        .unwrap()
        .follow()
        .unwrap();
    assert_eq!(fault.token(), Some(b"unset".as_slice()));
}
#[test]
fn early_halt_and_trace_bound_cannot_be_relabelled_complete() {
    let data = Trie::default();
    let a = data.provision(&[b"a"]);
    let mut p = ReferenceBranch::new();
    let root = p.machine.states.root();
    root.child(b"same-continuation")
        .unwrap()
        .point_to(&root.child(b"halt").unwrap());
    p.bind(&a, &a);
    assert_eq!(p.execute(6).unwrap(), 5);
    assert!(matches!(p.record(), Err(BranchError::Incomplete)));
    let mut looped = ReferenceBranch::new();
    looped
        .machine
        .states
        .root()
        .child(b"same-continuation")
        .unwrap()
        .point_to(&looped.construction.child(b"write-x-follow-y").unwrap());
    looped.bind(&a, &a);
    assert!(matches!(looped.execute(7), Err(BranchError::TraceLimit)));
    assert!(matches!(looped.record(), Err(BranchError::NotReady)));
}
#[test]
fn proof_and_argument_handles_survive_controller_and_source_root_drop() {
    let result = {
        let data = Trie::default();
        let a = data.provision(&[b"argument"]);
        let mut p = ReferenceBranch::new();
        run(&mut p, &a, &a)
    };
    assert_eq!(result.value.token(), Some(b"same".as_slice()));
    assert_eq!(
        result
            .evidence
            .child(b"input-x")
            .unwrap()
            .follow()
            .unwrap()
            .token(),
        Some(b"argument".as_slice())
    );
    let target = result
        .evidence
        .child(b"steps")
        .unwrap()
        .child(&5usize.to_be_bytes())
        .unwrap()
        .child(b"target")
        .unwrap()
        .follow()
        .unwrap();
    assert!(target.follow().unwrap().same(&result.value));
}
