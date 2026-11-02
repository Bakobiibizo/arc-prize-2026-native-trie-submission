use native_trie_seed::byte_views::ByteProbe;
fn shown(value: u8) -> [bool; 8] {
    std::array::from_fn(|i| value & (1 << (7 - i)) != 0)
}
fn main() {
    let mut probe = ByteProbe::new();
    let input = probe.load_bits(&shown(0xC3));
    probe.attach(&input);
    probe.start_read();
    let cycles = probe.execute(100).unwrap();
    assert!(probe.status_is(&probe.statuses.unknown));
    println!("untaught bits: unknown byte view ({cycles} copies)");
    for value in [0xC3, 0xA9] {
        let input = probe.load_bits(&shown(value));
        let view = probe.start_lesson(&input, value, b"shown-byte-pair");
        let cycles = probe.execute(100).unwrap();
        assert!(probe.status_is(&probe.statuses.installed));
        let anchor = probe.bit_anchor().unwrap().unwrap();
        assert!(view.follow().unwrap().same(&anchor));
        println!(
            "admitted shown byte {value:02X}: bit anchor -> view -> same bit anchor ({cycles} copies)"
        );
    }
    let bits: Vec<_> = [0xC3, 0xA9].into_iter().flat_map(shown).collect();
    let input = probe.load_bits(&bits);
    probe.attach(&input);
    drop(input);
    drop(bits);
    for expected in [0xC3, 0xA9] {
        probe.start_read();
        let cycles = probe.execute(100).unwrap();
        assert_eq!(probe.emitted_byte(), Ok(Some(expected)));
        let anchor = probe.bit_anchor().unwrap().unwrap();
        let view = probe.result.follow().unwrap();
        assert!(view.follow().unwrap().same(&anchor));
        assert!(
            anchor
                .child(b"holds")
                .unwrap()
                .child(b"unicode-scalar")
                .is_none()
        );
        println!(
            "fresh stream: byte {expected:02X}, backlink verified, no scalar view ({cycles} copies)"
        );
    }
    probe.start_read();
    let cycles = probe.execute(30).unwrap();
    assert!(probe.status_is(&probe.statuses.eof));
    println!("end of supplied stream ({cycles} copies); no Unicode decoding performed");
}
