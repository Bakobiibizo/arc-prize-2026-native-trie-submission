//! Origin full001/05a7 resource failure: empty diagnostic reservations are not written facts.
use native_trie_seed::reference_branch::ReferenceBranch;
use native_trie_seed::{Address, Fault, source_field};
const FAULTS: [&[u8]; 4] = [
    b"source-read-fault",
    b"target-read-fault",
    b"next-read-fault",
    b"copied-read-fault",
];
fn step(frame: &Address, i: usize) -> Address {
    frame
        .child(b"steps")
        .unwrap()
        .child(&i.to_be_bytes())
        .unwrap()
}
#[test]
fn compact_success_retires_only_empty_reservations_preserving_actual_step_nodes() {
    let mut b = ReferenceBranch::new();
    let x = b.machine.states.root().provision(b"fixture-input");
    let f = b.bind(&x, &x);
    b.execute(6).unwrap();
    let original = step(&f, 0).child(b"copied-reference").unwrap();
    let value = original.follow().unwrap();
    let p = b.record_compact().unwrap();
    assert!(p.value.same(&b.same_tag()));
    assert!(
        step(&f, 0)
            .child(b"copied-reference")
            .unwrap()
            .same(&original)
    );
    assert!(original.follow().unwrap().same(&value));
    for i in 0..6 {
        for k in FAULTS {
            assert!(step(&f, i).child(k).is_none());
        }
    }
    let f = b.bind(&x, &x);
    b.execute(6).unwrap();
    b.record().unwrap();
    for k in FAULTS {
        assert!(matches!(
            step(&f, 0).child(k).unwrap().follow(),
            Err(Fault::Unset)
        ));
    }
}
#[test]
fn compact_keeps_written_unset_fault_tag_terminal_and_nonempty_reservations() {
    let mut b = ReferenceBranch::new();
    let x = b.machine.states.root().provision(b"fixture-input");
    let f = b.bind(&x, &x);
    b.execute(6).unwrap();
    let s = step(&f, 0);
    let tag = b
        .machine
        .states
        .root()
        .child(b"read-faults")
        .unwrap()
        .child(b"unset")
        .unwrap();
    let written = s.child(FAULTS[0]).unwrap();
    written.point_to(&tag);
    let terminal = s.child(FAULTS[1]).unwrap();
    terminal.terminate();
    let nonempty = s.child(FAULTS[2]).unwrap();
    nonempty.provision(b"fixture-proof").terminate();
    b.record_compact().unwrap();
    assert!(s.child(FAULTS[0]).unwrap().same(&written));
    assert!(written.follow().unwrap().same(&tag));
    assert!(s.child(FAULTS[1]).unwrap().same(&terminal));
    assert!(s.child(FAULTS[2]).unwrap().same(&nonempty));
    assert!(s.child(FAULTS[3]).is_none());
}
#[test]
fn partial_and_native_faulted_frames_never_retire() {
    let mut b = ReferenceBranch::new();
    let x = b.machine.states.root().provision(b"fixture-input");
    let f = b.bind(&x, &x);
    assert!(b.execute(2).is_err());
    assert!(b.record_compact().is_err());
    for i in 0..6 {
        for k in FAULTS {
            assert!(step(&f, i).child(k).is_some());
        }
    }
    let f = b.bind(&x, &x);
    source_field(&b.machine.current().unwrap())
        .unwrap()
        .terminate();
    assert!(b.execute(6).is_err());
    assert!(b.record_compact().is_err());
    let actual = step(&f, 0)
        .child(b"source-read-fault")
        .unwrap()
        .follow()
        .unwrap();
    assert!(
        actual.same(
            &b.machine
                .states
                .root()
                .child(b"read-faults")
                .unwrap()
                .child(b"terminal")
                .unwrap()
        )
    );
    for i in 0..6 {
        for k in FAULTS {
            assert!(step(&f, i).child(k).is_some());
        }
    }
}
