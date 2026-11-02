#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Trie,
    byte_views::{ByteProbe, LookupError},
};
fn shown(value: u8) -> [bool; 8] {
    std::array::from_fn(|i| value & (1 << (7 - i)) != 0)
}
fn path(value: u8) -> [[u8; 1]; 8] {
    shown(value).map(|bit| [u8::from(bit)])
}
fn trained() -> ByteProbe {
    let mut model = ByteProbe::new();
    for value in 0..=255 {
        let input = model.load_bits(&shown(value));
        model.start_lesson(&input, value, b"shown-byte-v1");
        assert_eq!(model.execute(71), Ok(71));
    }
    model
}
#[test]
fn generic_addressing_preserves_token_shapes_without_decoding_or_provisioning() {
    let trie = Trie::default();
    let literal = trie.provision(&[b"a/b", b"", &[0, 255]]);
    let split = trie.provision(&[b"a", b"b", b"", &[0, 255]]);
    heap::start();
    let found = trie.root().locate([b"a/b".as_slice(), b"", &[0, 255]]);
    let sample = heap::finish();
    assert!(found.unwrap().same(&literal));
    assert!(!literal.same(&split));
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    assert!(trie.root().locate([b"absent".as_slice()]).is_none());
    assert!(trie.root().child(b"absent").is_none());
    assert!(
        trie.root()
            .locate(std::iter::empty())
            .unwrap()
            .same(&trie.root())
    );
}
#[test]
fn all_byte_addresses_resolve_without_a_single_copy_or_query_record() {
    let model = trained();
    for value in 0..=255 {
        let tokens = path(value);
        let before = model.machine.cycles;
        heap::start();
        let view = model.locate_view(tokens.iter().map(|t| t.as_slice()));
        let sample = heap::finish();
        let view = view.unwrap();
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
        assert_eq!(model.machine.cycles, before);
        assert_eq!(view.token(), Some([value].as_slice()));
        let anchor = view.follow().unwrap();
        heap::start();
        let direct = model.view_at(&anchor);
        let sample = heap::finish();
        assert!(direct.unwrap().same(&view));
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
    }
}
#[test]
fn optional_emission_executes_only_the_found_construction() {
    let mut model = trained();
    let tokens = path(0xC3);
    let view = model
        .locate_view(tokens.iter().map(|t| t.as_slice()))
        .unwrap();
    heap::start();
    model.start_view(&view);
    let done = model.execute(1);
    let sample = heap::finish();
    assert_eq!(done, Ok(1));
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    assert!(model.result.follow().unwrap().same(&view));
    assert!(
        view.follow()
            .unwrap()
            .child(b"holds")
            .unwrap()
            .child(b"unicode-scalar")
            .is_none()
    );
}
#[test]
fn missing_paths_and_unrecorded_views_are_not_decoded_or_halted() {
    let model = ByteProbe::new();
    let tokens = path(0xA5);
    assert!(matches!(
        model.locate_view(tokens.iter().map(|t| t.as_slice())),
        Err(LookupError::Unrecorded)
    ));
    assert!(matches!(
        model.locate_view([b"missing".as_slice()]),
        Err(LookupError::MissingPath)
    ));
    assert!(matches!(
        model.view_at(&model.representations.root()),
        Err(LookupError::MissingView)
    ));
}
#[test]
fn supplied_anchor_reads_current_trie_reference_not_a_cached_answer() {
    let model = trained();
    let a = path(0x41);
    let b = path(0x42);
    let first = model.locate_view(a.iter().map(|t| t.as_slice())).unwrap();
    let second = model.locate_view(b.iter().map(|t| t.as_slice())).unwrap();
    let anchor = first.follow().unwrap();
    let slot = anchor
        .child(b"holds")
        .unwrap()
        .child(b"raw-byte-msb-first")
        .unwrap();
    // Deliberate low-level rewiring: proves direct addressing reads the actual slot.
    slot.point_to(&second);
    assert!(model.view_at(&anchor).unwrap().same(&second));
    slot.point_to(&first);
}
