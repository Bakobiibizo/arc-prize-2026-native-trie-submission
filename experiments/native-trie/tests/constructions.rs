#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{Fault, Outcome, constructions::*};

#[test]
fn nested_example_retains_bindings_and_parent_in_the_trie() {
    let mut builder = Builder::new();
    let bob = builder.literal(b"bob");
    let bob_first = bob.first.clone();
    let bob_end = bob.end.clone();
    let alice = builder.literal(b"alice");
    let inner = builder.join(bob, alice);
    let inner_first = inner.first.clone();
    let inner_end = inner.end.clone();
    let inner_context = inner.context.clone();
    let suffix = builder.literal(b"!");
    let root = builder.join(inner, suffix);
    let parent_entry = root.entry.clone();
    let mut eval = builder.finish(root);
    assert!(matches!(inner_first.follow(), Err(Fault::Unset)));
    assert!(!inner_end.is_terminal());
    heap::start();
    let result = eval.execute(200);
    let allocations = heap::finish();
    assert_eq!(result, Ok(134));
    assert_eq!(allocations.requests, 0);
    assert_eq!(allocations.bytes, 0);
    assert_eq!(eval.inspect(32).unwrap(), b"bobalice!");
    assert_eq!(
        eval.inspect_view(&inner_first, Some(&inner_end), 32)
            .unwrap(),
        b"bobalice"
    );
    assert_eq!(
        eval.inspect_view(&bob_first, Some(&bob_end), 32).unwrap(),
        b"bob"
    );
    assert!(
        inner_context
            .child(b"parent")
            .unwrap()
            .follow()
            .unwrap()
            .same(&parent_entry)
    );
    assert!(
        inner_context
            .child(b"bound-first")
            .unwrap()
            .follow()
            .unwrap()
            .same(&bob_first)
    );
    assert!(
        inner_context
            .child(b"bound-end")
            .unwrap()
            .follow()
            .unwrap()
            .same(&bob_end)
    );
    assert!(eval.end().unwrap().is_terminal());
    assert!(
        eval.root_context
            .child(b"parent")
            .unwrap()
            .follow()
            .unwrap()
            .same(&eval.halt())
    );
}

#[test]
fn literal_values_and_append_literals_are_not_executor_operations() {
    for (left, right, append) in [
        ("bob", "alice", "!"),
        ("π", "λ", "?"),
        ("", "alice", ""),
        ("bob", "", "!"),
        ("", "", ""),
        ("mark", "join", "alice"),
    ] {
        let mut b = Builder::new();
        let l = b.literal(left.as_bytes());
        let r = b.literal(right.as_bytes());
        let c0 = b.join(l, r);
        let suffix = b.literal(append.as_bytes());
        let c1 = b.join(c0, suffix);
        let mut eval = b.finish(c1);
        let expected = [left.as_bytes(), right.as_bytes(), append.as_bytes()].concat();
        heap::start();
        let result = eval.execute(400);
        let allocations = heap::finish();
        assert_eq!(result, Ok(116 + 2 * expected.len() as u64));
        assert_eq!(allocations.requests, 0);
        assert_eq!(allocations.bytes, 0);
        assert_eq!(eval.inspect(128).unwrap(), expected);
    }
}

#[test]
fn every_byte_value_is_data_including_zero() {
    let bytes: Vec<u8> = (0..=255).collect();
    let mut b = Builder::new();
    let literal = b.literal(&bytes);
    let mut eval = b.finish(literal);
    heap::start();
    let result = eval.execute(523);
    let sample = heap::finish();
    assert_eq!(result, Ok(522));
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    assert_eq!(eval.inspect(257).unwrap(), bytes);
}

#[test]
fn input_references_are_consulted_at_runtime_not_compiled_into_a_result() {
    let mut b = Builder::new();
    let left = b.literal(b"left");
    let original_right = b.literal(b"original");
    let replacement = b.literal(b"changed");
    let root = b.join(left, original_right);
    let operand = root.context.child(b"right").unwrap();
    let mut eval = b.finish(root);
    // Only data changes after the assembler has been dropped. No new program is emitted.
    operand.point_to(&replacement.entry);
    assert_eq!(eval.execute(300), Ok(43 + 20 + 2 * 11));
    assert_eq!(eval.inspect(64).unwrap(), b"leftchanged");
}

#[test]
fn left_and_right_nesting_keep_distinct_contexts() {
    let mut b = Builder::new();
    let a = b.literal(b"a");
    let bb = b.literal(b"b");
    let c = b.literal(b"c");
    let d = b.literal(b"d");
    let left = b.join(a, bb);
    let right = b.join(c, d);
    let root = b.join(left, right);
    let mut eval = b.finish(root);
    assert_eq!(eval.execute(300), Ok(43 * 3 + 10 * 4 + 2 * 4));
    assert_eq!(eval.inspect(32).unwrap(), b"abcd");
}

#[test]
fn nesting_depth_has_affine_copy_work_and_zero_execution_allocations() {
    for depth in [0, 1, 8, 64, 512, 4096] {
        for right_nested in [false, true] {
            let mut b = Builder::new();
            let mut root = b.literal(b"bob");
            for _ in 0..depth {
                let punctuation = b.literal(b"!");
                root = if right_nested {
                    b.join(punctuation, root)
                } else {
                    b.join(root, punctuation)
                };
            }
            let mut eval = b.finish(root);
            heap::start();
            let result = eval.execute(55 * depth + 17);
            let allocations = heap::finish();
            assert_eq!(result, Ok((55 * depth + 16) as u64));
            assert_eq!(allocations.requests, 0);
            assert_eq!(allocations.bytes, 0);
            let actual = eval.inspect(2 * depth + 8).unwrap();
            assert_eq!(actual.len(), depth + 3);
            if right_nested {
                assert_eq!(&actual[depth..], b"bob");
                assert!(actual[..depth].iter().all(|b| *b == b'!'));
            } else {
                assert_eq!(&actual[..3], b"bob");
                assert!(actual[3..].iter().all(|b| *b == b'!'));
            }
        }
    }
}

#[test]
fn fixed_small_host_stack_handles_nested_contexts() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let depth = 512;
            let mut b = Builder::new();
            let mut root = b.literal(b"x");
            for _ in 0..depth {
                let tail = b.literal(b"!");
                root = b.join(root, tail);
            }
            let mut eval = b.finish(root);
            heap::start();
            let result = eval.execute(55 * depth + 13);
            let sample = heap::finish();
            assert_eq!(result, Ok((55 * depth + 12) as u64));
            assert_eq!(sample.requests, 0);
            assert_eq!(sample.bytes, 0);
            assert_eq!(eval.inspect(2 * depth + 4).unwrap().len(), depth + 1);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn missing_child_is_a_fault_not_a_null_terminal() {
    let mut b = Builder::new();
    let left = b.literal(b"bob");
    let right = b.literal(b"alice");
    let root = b.join(left, right);
    let absent = root.context.provision(b"absent");
    root.context.child(b"right").unwrap().point_to(&absent);
    let mut eval = b.finish(root);
    assert_eq!(
        eval.execute(200),
        Err(EvaluationError::Machine(Fault::MissingField))
    );
    assert!(!absent.is_terminal());
    assert_eq!(eval.machine.cycle(), Err(Fault::MissingField));
}

#[test]
fn cycle_and_output_budgets_are_reported_as_limits() {
    let mut b = Builder::new();
    let input = b.literal(b"abc");
    let mut eval = b.finish(input);
    assert_eq!(eval.execute(1), Err(EvaluationError::Limit));
    assert_eq!(eval.execute(100), Ok(16));
    assert_eq!(eval.inspect(1), Err(InspectionError::Limit));
    assert_eq!(eval.inspect(8).unwrap(), b"abc");
    assert_eq!(eval.machine.cycle(), Ok(Outcome::Halted));
}

#[test]
fn early_explicit_terminal_is_not_reported_as_completed_outer_construction() {
    let mut b = Builder::new();
    let left = b.literal(b"a");
    let right = b.literal(b"b");
    let root = b.join(left, right);
    let stopped = root.context.provision(b"intentional-stop");
    stopped.terminate();
    root.context.child(b"right").unwrap().point_to(&stopped);
    let mut eval = b.finish(root);
    assert_eq!(eval.execute(200), Err(EvaluationError::UnexpectedTerminal));
}
