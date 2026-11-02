#[path = "../support/alias_fixture.rs"]
mod fixture;
#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{records, representations::*};
#[test]
fn supplied_representations_converge_on_actual_character_addresses() {
    let model = fixture::teach();
    for (path, id) in fixture::LESSONS {
        heap::start();
        let result = model.lookup(path.iter().copied());
        let sample = heap::finish();
        match result.unwrap() {
            Resolution::Complete { character, .. } => assert_eq!(character.token(), Some(*id)),
            _ => panic!("complete expected"),
        }
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
    }
    let c = fixture::target(&model, &[b"utf8", &[0x43]]);
    assert!(c.same(&fixture::target(&model, &[b"scalar", b"U+0043"])));
    assert!(c.same(&fixture::target(&model, fixture::LESSONS[0].0)));
    let e = fixture::target(&model, &[b"utf8", &[0xC3], &[0xA9]]);
    assert!(e.same(&fixture::target(&model, &[b"scalar", b"U+00E9"])));
    assert!(e.same(&fixture::target(&model, &[b"latin1", &[0xE9]])));
}
#[test]
fn prefix_is_not_completion_and_can_be_resumed_without_replay() {
    let model = fixture::teach();
    let prefix = match model.lookup([b"utf8".as_slice(), &[0xC3]]).unwrap() {
        Resolution::Prefix { anchor } => anchor,
        _ => panic!("incomplete prefix"),
    };
    for (suffix, id) in [(0xA9, b"e-acute".as_slice()), (0xB1, b"n-tilde".as_slice())] {
        let at = prefix.child(&[suffix]).unwrap();
        assert!(
            matches!(model.at(&at),Ok(Resolution::Complete { character,.. }) if character.token()==Some(id))
        );
    }
    assert!(matches!(
        model.lookup([b"utf8".as_slice(), &[0xE2], &[0x82]]),
        Ok(Resolution::Prefix { .. })
    ));
}
#[test]
fn representation_selector_is_not_inferred_from_identical_value_tokens() {
    let model = fixture::teach();
    assert!(matches!(
        model.lookup([b"utf8".as_slice(), &[0xE9]]),
        Ok(Resolution::Missing)
    ));
    assert_eq!(
        fixture::target(&model, &[b"latin1", &[0xE9]]).token(),
        Some(b"e-acute".as_slice())
    );
    // Not a validity checker or implicit normalization: unrecorded paths stay missing.
    for path in [
        &[b"utf8".as_slice(), &[0xC0], &[0xAF]][..],
        &[b"utf8".as_slice(), &[0x65], &[0xCC], &[0x81]][..],
    ] {
        assert!(matches!(
            model.lookup(path.iter().copied()),
            Ok(Resolution::Missing)
        ));
    }
}
#[test]
fn evidence_keeps_representation_path_source_and_shared_construction() {
    let mut model = fixture::teach();
    let path = &[b"utf8".as_slice(), &[0x43]];
    let c = fixture::target(&model, path);
    let source = model.source(b"second-occurrence");
    model.teach(path, &c, &source).unwrap();
    let (anchor, candidate) = match model.lookup(path.iter().copied()).unwrap() {
        Resolution::Complete {
            anchor, candidate, ..
        } => (anchor, candidate),
        _ => panic!("complete expected"),
    };
    let evidence = records::inspect_evidence(&candidate).unwrap();
    assert_eq!(evidence.len(), 2);
    for entry in evidence {
        let observation = entry.follow().unwrap();
        assert!(
            observation
                .child(b"representation")
                .unwrap()
                .follow()
                .unwrap()
                .same(&model.root().child(b"utf8").unwrap())
        );
        assert!(
            observation
                .child(b"path")
                .unwrap()
                .follow()
                .unwrap()
                .same(&anchor)
        );
        assert!(observation.child(b"source").unwrap().follow().is_ok());
        assert!(
            observation
                .child(b"construction")
                .unwrap()
                .follow()
                .unwrap()
                .same(&model.construction)
        );
    }
}
#[test]
fn alternatives_are_explicit_and_do_not_replace_character_identity() {
    let mut model = fixture::teach();
    let wrong = model.character(b"other-claim");
    let source = model.source(b"alternative");
    model.teach(&[b"utf8", &[0x43]], &wrong, &source).unwrap();
    assert!(matches!(
        model.lookup([b"utf8".as_slice(), &[0x43]]),
        Ok(Resolution::Ambiguous {
            alternatives: 2,
            ..
        })
    ));
    assert_eq!(
        fixture::target(&model, &[b"scalar", b"U+0043"]).token(),
        Some(b"C".as_slice())
    );
}
#[test]
fn one_shared_value_construction_emits_all_representation_results() {
    let mut model = fixture::teach();
    let construction = model.construction.clone();
    for (path, _) in fixture::LESSONS {
        let target = fixture::target(&model, path);
        heap::start();
        let copies = model.emit(&target);
        let sample = heap::finish();
        assert_eq!(copies, Ok(1));
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
        assert!(model.result.follow().unwrap().same(&target));
        assert!(model.construction.same(&construction));
    }
}
#[test]
fn a_completed_address_can_still_have_longer_completed_paths() {
    let mut model = Representations::new();
    let a = model.character(b"short");
    let ab = model.character(b"long");
    let source = model.source(b"supplied-prefix-example");
    model
        .teach(&[b"toy-representation", b"a"], &a, &source)
        .unwrap();
    model
        .teach(&[b"toy-representation", b"a", b"b"], &ab, &source)
        .unwrap();
    assert!(fixture::target(&model, &[b"toy-representation", b"a"]).same(&a));
    assert!(fixture::target(&model, &[b"toy-representation", b"a", b"b"]).same(&ab));
}

#[test]
fn character_structure_and_semantics_are_separate_from_aliases() {
    let model = fixture::teach();
    let c = fixture::target(&model, &[b"utf8", &[0x43]]);
    let structure = model
        .root()
        .provision(b"structures")
        .provision(b"C-structure");
    let semantics = model
        .root()
        .provision(b"semantics")
        .provision(b"C-semantics");
    c.provision(b"structure").point_to(&structure);
    c.provision(b"semantics").point_to(&semantics);
    let alias = fixture::target(&model, &[b"scalar", b"U+0043"]);
    assert!(
        alias
            .child(b"structure")
            .unwrap()
            .follow()
            .unwrap()
            .same(&structure)
    );
    assert!(
        alias
            .child(b"semantics")
            .unwrap()
            .follow()
            .unwrap()
            .same(&semantics)
    );
    assert!(!structure.same(&semantics));
}
