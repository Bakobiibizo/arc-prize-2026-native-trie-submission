#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Trie,
    character_prefixes::*,
    prefixes::{self, PrefixSelection},
    records,
    representations::Resolution,
    words::{CharacterEvent, Words},
};
// Independent query/oracle generation, never called by model selection/execution.
fn path(value: u32) -> ScalarPath {
    std::array::from_fn(|i| [((value >> (20 - i)) & 1) as u8])
}
#[test]
fn generic_prefix_selection_is_specific_and_keeps_ambiguity_explicit() {
    let data = Trie::default();
    let root = data.root();
    let a = root.provision(b"a");
    let ab = a.provision(b"b");
    let broad = root.provision(b"broad");
    let specific = root.provision(b"specific");
    let source = root.provision(b"source");
    records::record(&a, b"construction", &broad, &source).unwrap();
    records::record(&ab, b"construction", &specific, &source).unwrap();
    match prefixes::select(&root, [b"a".as_slice(), b"b", b"tail"]).unwrap() {
        PrefixSelection::Match(m) => {
            assert!(m.construction.same(&specific));
            assert_eq!(m.depth, 2);
        }
        _ => panic!("specific expected"),
    }
    records::record(&ab, b"construction", &broad, &source).unwrap();
    assert!(matches!(
        prefixes::select(&root, [b"a".as_slice(), b"b"]),
        Ok(PrefixSelection::Ambiguous {
            alternatives: 2,
            ..
        })
    ));
}
#[test]
fn valid_boundaries_forward_stable_canonical_value_identity() {
    let mut model = CharacterIdentity::new();
    for scalar in [
        0, 0x43, 0xE9, 0x7FF, 0x800, 0xD7FF, 0xE000, 0xFFFF, 0x10000, 0x1F600, 0x10FFFF,
    ] {
        let input = path(scalar);
        let first = model.construct(&input).unwrap();
        heap::start();
        let second = model.construct(&input);
        let sample = heap::finish();
        let second = second.unwrap();
        assert!(first.value.same(&second.value));
        assert!(second.construction.same(&model.model.construction));
        assert_eq!(model.model.machine.cycles, 1);
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
    }
}
#[test]
fn surrogates_and_above_unicode_have_no_construction() {
    let model = CharacterIdentity::new();
    for scalar in [0xD800, 0xDBFF, 0xDFFF, 0x110000, 0x1FFFFF] {
        assert!(matches!(
            model.select(&path(scalar)),
            Err(CharacterError::Unrecorded)
        ));
    }
    let mut invalid = path(0x43);
    invalid[20] = [2];
    assert!(matches!(
        model.select(&invalid),
        Err(CharacterError::InvalidBit)
    ));
}
#[test]
fn prefix_knowledge_is_causal_and_selected_construction_is_executed() {
    let mut model = CharacterIdentity::new();
    let input = path(0x43);
    let found = model.select(&input).unwrap();
    // Change the actual selected construction's source binding: execution must observe trie code.
    let source = native_trie_seed::source_field(&found.construction).unwrap();
    let before = source.follow().unwrap();
    let replacement = model.model.source(b"replacement-value-cell");
    let alternate = model.model.character(b"explicit-other-output");
    replacement.point_to(&alternate);
    source.point_to(&replacement);
    let output = model.construct(&input).unwrap();
    assert!(output.value.same(&alternate));
    source.point_to(&before);
    found.candidate.terminate();
    assert!(matches!(
        model.select(&input),
        Err(CharacterError::Record(_))
    ));
}
#[test]
fn scalar_constructed_value_can_share_utf8_alias_and_word_edge() {
    let mut model = CharacterIdentity::new();
    let value = model.construct(&path(0x43)).unwrap().value;
    let source = model.model.source(b"shown-C-utf8");
    model
        .model
        .teach(&[b"utf8", &[0x43]], &value, &source)
        .unwrap();
    let origin = match model.model.lookup([b"utf8".as_slice(), &[0x43]]).unwrap() {
        Resolution::Complete {
            character,
            candidate,
            ..
        } => {
            assert!(character.same(&value));
            candidate
        }
        _ => panic!("alias expected"),
    };
    let mut words = Words::new();
    let result = words
        .teach(
            &[CharacterEvent {
                character: value.clone(),
                origin,
            }],
            &source,
        )
        .unwrap();
    assert!(
        result
            .candidate
            .follow()
            .unwrap()
            .same(&words.locate(&[value]).unwrap().unwrap())
    );
}
