#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{Address, Fault, byte_views::*, next_field, source_field, target_field};

// Teacher/oracle representation ONLY. The running model never calls this or computes a byte.
fn shown(value: u8) -> [bool; 8] {
    std::array::from_fn(|position| value & (1 << (7 - position)) != 0)
}
fn train(probe: &mut ByteProbe) {
    for value in 0..=255 {
        let input = probe.load_bits(&shown(value));
        let view = probe.start_lesson(&input, value, b"shown-eight-bit-byte");
        heap::start();
        let result = probe.execute(100);
        let sample = heap::finish();
        assert_eq!(result, Ok(71));
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
        assert!(probe.status_is(&probe.statuses.installed));
        let anchor = probe.bit_anchor().unwrap().unwrap();
        assert!(view.follow().unwrap().same(&anchor));
        assert!(
            anchor
                .child(b"holds")
                .unwrap()
                .child(b"raw-byte-msb-first")
                .unwrap()
                .follow()
                .unwrap()
                .same(&view)
        );
    }
}

#[test]
fn input_occurrences_share_actual_bit_addresses_and_outlive_loader_handles() {
    let mut probe = ByteProbe::new();
    let raw = shown(0xA5).to_vec();
    let input = probe.load_bits(&raw);
    for (field, bit) in input.payloads.iter().zip(&raw) {
        let canonical = probe
            .representations
            .root()
            .child(&[u8::from(*bit)])
            .unwrap();
        assert!(field.follow().unwrap().same(&canonical));
    }
    probe.attach(&input);
    drop(input);
    drop(raw);
    probe.start_read();
    assert_eq!(probe.execute(100), Ok(66));
    assert!(probe.status_is(&probe.statuses.unknown));
}

#[test]
fn absent_byte_view_remains_unknown_not_host_decoded() {
    let mut probe = ByteProbe::new();
    let input = probe.load_bits(&shown(0xA5));
    probe.attach(&input);
    probe.start_read();
    heap::start();
    let result = probe.execute(100);
    let sample = heap::finish();
    assert_eq!(result, Ok(66));
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    assert!(probe.status_is(&probe.statuses.unknown));
    assert_eq!(probe.emitted_byte(), Ok(None));
    let anchor = probe.bit_anchor().unwrap().unwrap();
    assert!(anchor.same(&probe.known_anchor(&shown(0xA5)).unwrap()));
    assert!(
        anchor
            .child(b"holds")
            .unwrap()
            .child(b"raw-byte-msb-first")
            .unwrap()
            .follow()
            .unwrap()
            .same(&probe.unknown_marker())
    );
    assert!(
        anchor
            .child(b"holds")
            .unwrap()
            .child(b"unicode-scalar")
            .is_none()
    );
}

#[test]
fn all_256_taught_views_round_trip_through_unchanged_controller() {
    let mut probe = ByteProbe::new();
    train(&mut probe);
    for value in 0..=255 {
        let input = probe.load_bits(&shown(value));
        probe.attach(&input);
        probe.start_read();
        heap::start();
        let result = probe.execute(100);
        let sample = heap::finish();
        assert_eq!(result, Ok(67));
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
        assert_eq!(probe.emitted_byte(), Ok(Some(value)));
        let view = probe.result.follow().unwrap();
        let anchor = probe.bit_anchor().unwrap().unwrap();
        assert!(view.follow().unwrap().same(&anchor));
        assert!(anchor.same(&probe.known_anchor(&shown(value)).unwrap()));
        let lesson = view.child(b"lesson").unwrap().follow().unwrap();
        assert_eq!(
            lesson
                .child(b"source-id")
                .unwrap()
                .follow()
                .unwrap()
                .token(),
            Some(b"shown-eight-bit-byte".as_slice())
        );
        assert!(lesson.child(b"input-occurrence").unwrap().follow().is_ok());
        assert!(view.child(b"construction").unwrap().follow().is_ok());
    }
}

#[test]
fn many_bytes_reuse_one_stream_cursor_without_host_bit_slicing() {
    let mut probe = ByteProbe::new();
    train(&mut probe);
    let values = [0, 255, 0xC3, 0xA9, 0x41, 0x41];
    let bits: Vec<_> = values.iter().flat_map(|value| shown(*value)).collect();
    let input = probe.load_bits(&bits);
    let before: Vec<_> = input
        .payloads
        .iter()
        .map(|cell| cell.follow().unwrap())
        .collect();
    probe.attach(&input);
    let mut first_a = None;
    for value in values {
        probe.start_read();
        heap::start();
        let cycles = probe.execute(100);
        let sample = heap::finish();
        assert_eq!(cycles, Ok(67));
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
        assert_eq!(probe.emitted_byte(), Ok(Some(value)));
        if value == 0x41 {
            let at = probe.bit_anchor().unwrap().unwrap();
            if let Some(ref earlier) = first_a {
                assert!(at.same(earlier));
            } else {
                first_a = Some(at);
            }
        }
    }
    probe.start_read();
    assert_eq!(probe.execute(30), Ok(10));
    assert!(probe.status_is(&probe.statuses.eof));
    assert_eq!(probe.emitted_byte(), Ok(None));
    assert!(probe.bit_anchor().unwrap().is_none());
    for (cell, original) in input.payloads.iter().zip(before) {
        assert!(cell.follow().unwrap().same(&original));
    }
}

#[test]
fn input_value_can_change_after_loading_without_emitting_decoder_code() {
    let mut probe = ByteProbe::new();
    train(&mut probe);
    let input = probe.load_bits(&[false; 8]);
    let one = probe.load_bits(&[true]);
    input.payloads[0].point_to(&one.payloads[0].follow().unwrap());
    probe.attach(&input);
    probe.start_read();
    assert_eq!(probe.execute(100), Ok(67));
    assert_eq!(probe.emitted_byte(), Ok(Some(128)));
}

#[test]
fn every_partial_byte_reports_truncation_and_never_emits_stale_byte() {
    let mut probe = ByteProbe::new();
    train(&mut probe);
    let complete = probe.load_bits(&shown(255));
    probe.attach(&complete);
    probe.start_read();
    probe.execute(100).unwrap();
    assert_eq!(probe.emitted_byte(), Ok(Some(255)));
    for length in 0..8 {
        let input = probe.load_bits(&vec![true; length]);
        probe.attach(&input);
        probe.start_read();
        heap::start();
        let result = probe.execute(100);
        let sample = heap::finish();
        assert_eq!(result, Ok(10 + 7 * length as u64));
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
        assert!(probe.status_is(if length == 0 {
            &probe.statuses.eof
        } else {
            &probe.statuses.truncated
        }));
        assert_eq!(probe.emitted_byte(), Ok(None));
        assert!(probe.bit_anchor().unwrap().is_none());
    }
}

#[test]
fn repeated_and_contradictory_lessons_defer_without_overwriting_evidence() {
    let mut probe = ByteProbe::new();
    let input = probe.load_bits(&shown(0x41));
    let original = probe.start_lesson(&input, 0x41, b"source-a");
    probe.execute(100).unwrap();
    let anchor = original.follow().unwrap();
    let original_source = original.child(b"lesson").unwrap().follow().unwrap();
    for value in [0x41, 0xFF] {
        let new_input = probe.load_bits(&shown(0x41));
        let candidate = probe.start_lesson(&new_input, value, b"source-a");
        assert_eq!(probe.execute(100), Ok(67));
        assert!(probe.status_is(&probe.statuses.needs_reconciliation));
        assert!(matches!(candidate.follow(), Err(Fault::Unset)));
        assert!(!candidate.same(&original));
        let candidate_source = candidate.child(b"lesson").unwrap().follow().unwrap();
        assert!(!candidate_source.same(&original_source));
        assert!(
            candidate_source
                .child(b"input-occurrence")
                .unwrap()
                .follow()
                .unwrap()
                .same(&new_input.first)
        );
        assert!(
            original_source
                .child(b"input-occurrence")
                .unwrap()
                .follow()
                .unwrap()
                .same(&input.first)
        );
        assert!(
            anchor
                .child(b"holds")
                .unwrap()
                .child(b"raw-byte-msb-first")
                .unwrap()
                .follow()
                .unwrap()
                .same(&original)
        );
    }
    probe.attach(&input);
    probe.start_read();
    probe.execute(100).unwrap();
    assert_eq!(probe.emitted_byte(), Ok(Some(0x41)));
}

fn definition_cells(at: &Address, depth: usize, out: &mut Vec<(Address, Address)>) {
    for cell in [
        source_field(at).unwrap(),
        target_field(at).unwrap(),
        next_field(at).unwrap(),
    ] {
        out.push((cell.clone(), cell.follow().unwrap()));
    }
    if depth < 8 {
        definition_cells(&at.child(&[0]).unwrap(), depth + 1, out);
        definition_cells(&at.child(&[1]).unwrap(), depth + 1, out);
    } else {
        let slot = at
            .child(b"holds")
            .unwrap()
            .child(b"raw-byte-msb-first")
            .unwrap();
        out.push((slot.clone(), slot.follow().unwrap()));
    }
}

#[test]
fn reading_reuses_model_definitions_and_shared_views_without_mutation() {
    let mut probe = ByteProbe::new();
    train(&mut probe);
    let mut definitions = Vec::new();
    definition_cells(&probe.representations.root(), 0, &mut definitions);
    let input = probe.load_bits(&shown(0x80));
    probe.attach(&input);
    probe.start_read();
    probe.execute(100).unwrap();
    let view = probe.result.follow().unwrap();
    let back = view.follow().unwrap();
    let input2 = probe.load_bits(&shown(0x80));
    probe.attach(&input2);
    probe.start_read();
    probe.execute(100).unwrap();
    assert!(probe.result.follow().unwrap().same(&view));
    assert!(view.follow().unwrap().same(&back));
    for (cell, before) in definitions {
        assert!(cell.follow().unwrap().same(&before));
    }
}

#[test]
fn read_and_lesson_limits_remain_unresolved_not_success() {
    let mut probe = ByteProbe::new();
    let input = probe.load_bits(&shown(0x41));
    let pending = probe.start_lesson(&input, 0x41, b"limited");
    assert_eq!(probe.execute(1), Err(ProbeError::Limit));
    assert_eq!(probe.emitted_byte(), Err(ProbeError::NotCompleted));
    assert!(matches!(pending.follow(), Err(Fault::Unset)));
    assert_eq!(probe.execute(100), Ok(71));
    assert!(probe.status_is(&probe.statuses.installed));
    probe.attach(&input);
    probe.start_read();
    assert_eq!(probe.emitted_byte(), Err(ProbeError::NotCompleted));
    assert_eq!(probe.execute(100), Ok(67));
    assert_eq!(probe.emitted_byte(), Ok(Some(0x41)));
}
