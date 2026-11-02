use native_trie_seed::byte_views::{ByteProbe, ProbeError};
#[test]
fn budget_counts_copies_and_observes_completion_without_an_extra_copy() {
    let mut probe = ByteProbe::new();
    let input = probe.load_bits(&[false; 8]);
    probe.start_lesson(&input, 0, b"exact-budget");
    assert_eq!(probe.execute(71), Ok(71));
    probe.attach(&input);
    probe.start_read();
    assert_eq!(probe.execute(66), Err(ProbeError::Limit));
    assert_eq!(probe.emitted_byte(), Err(ProbeError::NotCompleted));
    assert_eq!(probe.execute(1), Ok(67));
    assert_eq!(probe.emitted_byte(), Ok(Some(0)));
}
