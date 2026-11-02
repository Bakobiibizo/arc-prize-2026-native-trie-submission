#[path = "../support/word_fixture.rs"]
mod fixture;
#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{
    Address, Fault, Trie,
    records::{self, Change},
    words::*,
};
fn chars(events: &[CharacterEvent]) -> Vec<Address> {
    events.iter().map(|e| e.character.clone()).collect()
}
fn complete(words: &Words, path: &[Address]) -> Address {
    let at = words.locate(path).unwrap().unwrap();
    match words.resolve(&at).unwrap() {
        WordResolution::Complete { word, .. } => word,
        _ => panic!("expected word"),
    }
}
#[test]
fn alias_representations_build_one_canonical_word_path() {
    let model = fixture::setup();
    let mut words = Words::new();
    let utf = fixture::events(&model, &[0, 1, 2], false);
    let scalar = fixture::events(&model, &[0, 1, 2], true);
    let a = model.source(b"utf8-span");
    let b = model.source(b"scalar-span");
    let first = words.teach(&utf, &a).unwrap();
    let second = words.teach(&scalar, &b).unwrap();
    assert!(first.candidate.same(&second.candidate));
    assert_eq!(
        records::inspect_evidence(&first.candidate).unwrap().len(),
        2
    );
    assert!(complete(&words, &chars(&utf)).same(&complete(&words, &chars(&scalar))));
    assert!(!utf[0].origin.same(&scalar[0].origin));
    assert!(utf[0].character.same(&scalar[0].character));
}
#[test]
fn related_words_share_prefix_and_completed_words_can_extend() {
    let model = fixture::setup();
    let mut words = Words::new();
    let boundary = model.source(b"given-boundary");
    for input in [
        &[0, 1, 2][..],
        &[0, 1, 3][..],
        &[0, 1, 2, 4][..],
        &[0, 1, 5, 6][..],
    ] {
        words
            .teach(&fixture::events(&model, input, false), &boundary)
            .unwrap();
    }
    let car = chars(&fixture::events(&model, &[0, 1, 2], false));
    let cat = chars(&fixture::events(&model, &[0, 1, 3], false));
    let prefix = words.locate(&car[..2]).unwrap().unwrap();
    assert!(prefix.same(&words.locate(&cat[..2]).unwrap().unwrap()));
    assert!(matches!(words.resolve(&prefix), Ok(WordResolution::Prefix)));
    let at = complete(&words, &car);
    assert!(!at.is_terminal());
    let s = fixture::events(&model, &[4], false);
    let cars = words.step(&at, &s[0].character).unwrap().unwrap();
    assert!(matches!(
        words.resolve(&cars),
        Ok(WordResolution::Complete { .. })
    ));
    assert!(!cars.same(&at));
    let cafe = chars(&fixture::events(&model, &[0, 1, 5, 6], true));
    assert!(matches!(
        words.resolve(&words.locate(&cafe).unwrap().unwrap()),
        Ok(WordResolution::Complete { .. })
    ));
}
#[test]
fn query_is_read_only_and_incoming_labels_are_not_used_as_strings() {
    let model = fixture::setup();
    let mut words = Words::new();
    let events = fixture::events(&model, &[0, 1, 2], false);
    words.teach(&events, &model.source(b"span")).unwrap();
    let input = chars(&events);
    heap::start();
    let at = words.locate(&input).unwrap().unwrap();
    let result = words.resolve(&at);
    let sample = heap::finish();
    assert!(matches!(result, Ok(WordResolution::Complete { .. })));
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    assert!(
        words
            .locate(&chars(&fixture::events(&model, &[0, 1, 3], false)))
            .unwrap()
            .is_none()
    );
}
#[test]
fn same_episode_replay_is_idempotent_but_new_boundaries_retain_evidence() {
    let model = fixture::setup();
    let mut words = Words::new();
    let events = fixture::events(&model, &[0, 1, 2], false);
    let boundary = model.source(b"span");
    let first = words.teach(&events, &boundary).unwrap();
    heap::start();
    let repeated = words.teach(&events, &boundary);
    let sample = heap::finish();
    let repeated = repeated.unwrap();
    assert_eq!(repeated.change, Change::AlreadyRecorded);
    assert!(repeated.evidence.same(&first.evidence));
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    words.teach(&events, &model.source(b"other-span")).unwrap();
    assert_eq!(
        records::inspect_evidence(&first.candidate).unwrap().len(),
        2
    );
}
#[test]
fn evidence_retains_boundary_constituents_and_actual_alias_origins() {
    let model = fixture::setup();
    let mut words = Words::new();
    let events = fixture::events(&model, &[0, 1, 5, 6], false);
    let boundary = model.source(b"raw-source-span");
    let result = words.teach(&events, &boundary).unwrap();
    let episode = result.evidence.follow().unwrap();
    assert!(
        episode
            .child(b"boundary")
            .unwrap()
            .follow()
            .unwrap()
            .same(&boundary)
    );
    assert!(episode.child(b"constituents").is_none());
    let entries = words.inspect_episode(&episode, events.len()).unwrap();
    assert_eq!(entries.len(), events.len());
    for (stored, event) in entries.iter().zip(&events) {
        assert!(stored.character.same(&event.character));
        assert!(stored.origin.same(&event.origin));
    }
    assert!(matches!(
        words.inspect_episode(&episode, events.len() - 1),
        Err(WordError::InspectionLimit)
    ));
}
#[test]
fn bad_origins_and_empty_input_do_not_publish_words() {
    let model = fixture::setup();
    let mut words = Words::new();
    let boundary = model.source(b"span");
    let mut events = fixture::events(&model, &[0, 1, 2], false);
    events[0].origin = events[1].origin.clone();
    assert!(matches!(
        words.teach(&events, &boundary),
        Err(WordError::OriginMismatch)
    ));
    assert!(matches!(words.teach(&[], &boundary), Err(WordError::Empty)));
    assert!(matches!(
        words.resolve(&words.root()),
        Ok(WordResolution::Prefix)
    ));
}
#[test]
fn equal_labels_do_not_merge_distinct_character_addresses() {
    let data = Trie::default();
    let a = data.provision(&[b"a", b"same"]);
    let b = data.provision(&[b"b", b"same"]);
    let oa = data.provision(&[b"origin-a"]);
    oa.point_to(&a);
    let ob = data.provision(&[b"origin-b"]);
    ob.point_to(&b);
    let mut words = Words::new();
    words
        .teach(
            &[CharacterEvent {
                character: a.clone(),
                origin: oa,
            }],
            &data.root(),
        )
        .unwrap();
    words
        .teach(
            &[CharacterEvent {
                character: b.clone(),
                origin: ob,
            }],
            &data.root(),
        )
        .unwrap();
    assert!(!complete(&words, &[a]).same(&complete(&words, &[b])));
}
#[test]
fn competing_low_level_claims_are_not_silently_selected() {
    let model = fixture::setup();
    let mut words = Words::new();
    let events = fixture::events(&model, &[0, 1, 2], false);
    let source = model.source(b"span");
    words.teach(&events, &source).unwrap();
    let at = complete(&words, &chars(&events));
    let other = model.character(b"other-claim");
    records::record(&at, b"word", &other, &source).unwrap();
    assert!(matches!(
        words.resolve(&at),
        Ok(WordResolution::Ambiguous { alternatives: 2 })
    ));
}
#[test]
fn character_owner_expiry_is_a_fault_not_a_missing_word() {
    let mut words = Words::new();
    let source = Trie::default();
    let character = {
        let model = fixture::setup();
        let events = fixture::events(&model, &[0], false);
        let character = events[0].character.clone();
        words.teach(&events, &source.root()).unwrap();
        character
    };
    let root = words.root();
    let at = words.step(&root, &character).unwrap().unwrap();
    drop(character);
    assert!(matches!(at.follow(), Err(Fault::Dangling)));
}
