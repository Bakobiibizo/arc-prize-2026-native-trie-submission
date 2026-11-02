use native_trie_seed::{Trie, records::*};
fn main() {
    let data = Trie::default();
    // Supplied address and shown values, not an encoding algorithm.
    let anchor = data.provision(&[b"bits", &[1], &[1], &[0], &[0], &[0], &[0], &[1], &[1]]);
    let target = data.provision(&[b"shown-values", &[0xC3]]);
    let other = data.provision(&[b"other-claim", &[0x41]]);
    let first = data.provision(&[b"sources", b"observation-1"]);
    let second = data.provision(&[b"sources", b"observation-2"]);
    let pending = prepare(&anchor, b"raw-byte-msb-first", &target, &first).unwrap();
    assert!(matches!(
        resolve(&anchor, b"raw-byte-msb-first"),
        Ok(Selection::Unknown)
    ));
    println!("prepared: no public candidate");
    let one = pending.publish().unwrap();
    assert_eq!(one.change, Change::NewTarget);
    let two = record(&anchor, b"raw-byte-msb-first", &target, &second).unwrap();
    assert_eq!(two.change, Change::AddedEvidence);
    assert!(one.candidate.same(&two.candidate));
    assert_eq!(inspect_evidence(&one.candidate).unwrap().len(), 2);
    println!("same target: one canonical candidate, two original evidence references");
    let repeated = record(&anchor, b"raw-byte-msb-first", &target, &first).unwrap();
    assert_eq!(repeated.change, Change::AlreadyRecorded);
    assert!(repeated.evidence.same(&one.evidence));
    println!("same evidence identity: idempotent, no replacement");
    record(&anchor, b"raw-byte-msb-first", &other, &second).unwrap();
    assert!(matches!(
        resolve(&anchor, b"raw-byte-msb-first"),
        Ok(Selection::Ambiguous { alternatives: 2 })
    ));
    println!(
        "different target: both alternatives retained; resolution is ambiguous, no first-value fallback"
    );
    for candidate in inspect_candidates(&anchor, b"raw-byte-msb-first") {
        assert!(
            candidate
                .child(b"anchor")
                .unwrap()
                .follow()
                .unwrap()
                .same(&anchor)
        );
    }
    println!("all candidate backlinks verified; no byte/Unicode interpretation or VM execution");
}
