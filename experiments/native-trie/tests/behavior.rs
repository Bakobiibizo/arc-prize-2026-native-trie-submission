use native_trie_seed::*;

#[test]
fn completion_and_missing_construction_are_different() {
    let mut m = Machine::default();
    let missing = m.states.root().provision(b"missing");
    m.enter(&missing);
    assert_eq!(m.cycle(), Err(Fault::MissingField));
    let end = m.states.root().provision(b"end");
    end.terminate();
    m.enter(&end);
    assert_eq!(m.cycle(), Ok(Outcome::Halted));
    assert_eq!(m.cycles, 0);
}

#[test]
fn post_write_continuation_controls_the_next_address() {
    let mut m = Machine::default();
    let root = m.states.root();
    let before = root.provision(b"before");
    before.terminate();
    let after = root.provision(b"after");
    after.terminate();
    let literal = root.provision(b"literal");
    literal.point_to(&after);
    let dispatch = root.provision(b"dispatch");
    dispatch.point_to(&before);
    let state = root.provision(b"change-dispatch");
    install(&state, &literal, &dispatch, &dispatch);
    m.enter(&state);
    assert_eq!(m.cycle(), Ok(Outcome::Advanced));
    assert!(m.current().unwrap().same(&after));
    assert_eq!(m.cycle(), Ok(Outcome::Halted));
}

#[test]
fn failure_does_not_change_either_root_or_pointer() {
    let mut m = Machine::default();
    let root = m.states.root();
    let old = root.provision(b"old");
    let new = root.provision(b"new");
    let source = root.provision(b"source-value");
    source.point_to(&new);
    let target = m.output.root().provision(b"result");
    target.point_to(&old);
    let unset = root.provision(b"unbound-next");
    let at = root.provision(b"copy");
    install(&at, &source, &target, &unset);
    m.enter(&at);
    assert_eq!(m.cycle(), Err(Fault::Unset));
    assert!(m.current().unwrap().same(&at));
    assert!(target.follow().unwrap().same(&old));
    assert_eq!(m.cycles, 0);
}

#[test]
fn invalid_self_aliased_continuation_is_atomic() {
    let mut m = Machine::default();
    let root = m.states.root();
    let terminal = root.provision(b"terminal");
    terminal.terminate();
    let value = root.provision(b"value");
    let previous = root.provision(b"previous");
    value.point_to(&previous);
    let at = root.provision(b"bad-copy");
    install(&at, &terminal, &value, &value);
    m.enter(&at);
    assert_eq!(m.cycle(), Err(Fault::TerminalUsedAsReference));
    assert!(value.follow().unwrap().same(&previous));
    assert!(!value.is_terminal());
}

#[test]
fn terminal_markers_copy_across_physical_tries() {
    let mut m = Machine::default();
    let root = m.states.root();
    let seed = root.provision(b"terminal-seed");
    seed.terminate();
    let made = m.output.root().provision(b"made-terminal");
    let next = root.provision(b"next-cell");
    next.point_to(&made);
    let at = root.provision(b"copy-marker");
    install(&at, &seed, &made, &next);
    m.enter(&at);
    assert_eq!(m.cycle(), Ok(Outcome::Advanced));
    assert!(made.is_terminal());
    assert_eq!(m.cycle(), Ok(Outcome::Halted));
}

#[test]
fn instructions_construct_another_instruction_then_execute_it() {
    let mut m = Machine::default();
    let root = m.states.root();
    let done = root.provision(b"done");
    done.terminate();
    let return_cell = root.provision(b"return");
    return_cell.point_to(&done);
    let answer = root.provision(b"answer");
    let constant = root.provision(b"constant");
    constant.point_to(&answer);
    let result = m.output.root().provision(b"result");
    let original = root.provision(b"original");
    install(&original, &constant, &result, &return_cell);
    let built = root.provision(b"built");
    let built_source = built.provision(b"source");
    let built_target = built.provision(b"target");
    let built_next = built.provision(b"next");
    assert!(matches!(built_source.follow(), Err(Fault::Unset)));
    let first = root.provision(b"build-source");
    let second = root.provision(b"build-target");
    let third = root.provision(b"build-next");
    let next_second = root.provision(b"next-second");
    next_second.point_to(&second);
    let next_third = root.provision(b"next-third");
    next_third.point_to(&third);
    let next_built = root.provision(b"next-built");
    next_built.point_to(&built);
    install(
        &first,
        &source_field(&original).unwrap(),
        &built_source,
        &next_second,
    );
    install(
        &second,
        &target_field(&original).unwrap(),
        &built_target,
        &next_third,
    );
    install(
        &third,
        &next_field(&original).unwrap(),
        &built_next,
        &next_built,
    );
    m.enter(&first);
    for _ in 0..3 {
        assert_eq!(m.cycle(), Ok(Outcome::Advanced));
    }
    assert!(m.current().unwrap().same(&built));
    assert_eq!(m.cycle(), Ok(Outcome::Advanced));
    assert!(result.follow().unwrap().same(&answer));
    assert_eq!(m.cycle(), Ok(Outcome::Halted));
    assert_eq!(m.cycles, 4);
}

#[test]
fn source_field_is_mutable_trie_structure() {
    let mut m = Machine::default();
    let root = m.states.root();
    let at = root.provision(b"self");
    let first = root.provision(b"first-source");
    let second = root.provision(b"second-source");
    let value = root.provision(b"value");
    first.point_to(&second);
    second.point_to(&value);
    let again = root.provision(b"again");
    again.point_to(&at);
    let self_source = at.provision(b"source");
    install(&at, &first, &self_source, &again);
    m.enter(&at);
    m.cycle().unwrap();
    assert!(self_source.follow().unwrap().same(&second));
    m.cycle().unwrap();
    assert!(self_source.follow().unwrap().same(&value));
}

#[test]
fn expired_foreign_references_are_detected() {
    let trie = Trie::default();
    let link = trie.provision(&[b"link"]);
    {
        let temporary = Trie::default();
        link.point_to(&temporary.root());
    }
    assert!(matches!(link.follow(), Err(Fault::Dangling)));
}

#[test]
fn instruction_pointer_can_be_an_ordinary_destination() {
    let mut m = Machine::default();
    let root = m.states.root();
    let after = root.provision(b"after");
    after.terminate();
    let literal = root.provision(b"literal");
    literal.point_to(&after);
    let at = root.provision(b"jump");
    let pointer = m.pointer();
    install(&at, &literal, &pointer, &pointer);
    m.enter(&at);
    assert_eq!(m.cycle(), Ok(Outcome::Advanced));
    assert!(m.current().unwrap().same(&after));
    assert_eq!(m.cycle(), Ok(Outcome::Halted));
}

#[test]
fn trie_tokens_and_edge_sequences_do_not_collapse() {
    let trie = Trie::default();
    let a = trie.provision(&[b"a", b"b"]);
    let b = trie.provision(&[b"ab"]);
    let empty = trie.provision(&[b""]);
    let binary = trie.provision(&[&[0, 16, 255]]);
    assert!(!a.same(&b));
    assert!(!empty.same(&trie.root()));
    assert!(!binary.same(&empty));
    assert!(a.same(&trie.root().child(b"a").unwrap().child(b"b").unwrap()));
}

#[test]
fn counters_use_the_same_cycle_for_every_bit_and_overflow() {
    for width in 1..=8 {
        let mut c = counter::prepare(width, false);
        for pulse in 1..=1usize << width {
            c.pulse(width + 3);
            let expected = pulse % (1 << width);
            for (number, actual) in c.inspect().iter().enumerate() {
                assert_eq!(*actual, expected & (1 << number) != 0);
            }
            assert!(c.machine.current().unwrap().same(if expected == 0 {
                &c.overflowed
            } else {
                &c.returned
            }));
        }
    }
}

#[test]
fn full_and_short_carry_counts_scale_as_expected() {
    for width in [1, 8, 64, 512, 4096, 32768] {
        let mut long = counter::prepare(width, true);
        long.pulse(width + 3);
        assert_eq!(long.machine.cycles, width as u64 + 1);
        assert!(long.inspect().iter().all(|bit| !*bit));
        let mut short = counter::prepare(width, false);
        short.pulse(4);
        assert_eq!(short.machine.cycles, 2);
    }
}
