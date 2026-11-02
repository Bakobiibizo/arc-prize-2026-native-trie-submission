use native_trie_seed::byte_views::{ByteProbe, ProbeError};
#[test]
fn an_interrupted_admission_never_publishes_a_view_without_its_bit_backlink() {
    for budget in 0..=71 {
        let mut probe = ByteProbe::new();
        let bits = [false, true, false, false, false, false, false, true];
        let input = probe.load_bits(&bits);
        let view = probe.start_lesson(&input, 0x41, b"publication-boundary");
        let result = probe.execute(budget);
        assert_eq!(
            result,
            if budget < 71 {
                Err(ProbeError::Limit)
            } else {
                Ok(71)
            }
        );
        let anchor = probe.known_anchor(&bits).unwrap();
        let held = anchor
            .child(b"holds")
            .unwrap()
            .child(b"raw-byte-msb-first")
            .unwrap()
            .follow()
            .unwrap();
        if !held.same(&probe.unknown_marker()) {
            assert!(held.same(&view));
            assert!(
                view.follow().is_ok_and(|bits| bits.same(&anchor)),
                "published view lacks backlink at copy budget {budget}"
            );
        }
    }
}
