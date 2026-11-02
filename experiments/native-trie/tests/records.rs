#[path = "../support/heap.rs"]
mod heap;
use native_trie_seed::{Address, Fault, Trie, records::*};
fn unique(anchor: &Address, kind: &[u8]) -> (Address, Address) {
    match resolve(anchor, kind).unwrap() {
        Selection::Unique { candidate, target } => (candidate, target),
        _ => panic!("expected unique reference"),
    }
}
#[test]
fn shown_byte_views_share_bit_addresses_and_consolidate_evidence() {
    let data = Trie::default();
    let bits = data.root().provision(b"bits");
    for value in 0u8..=255 {
        // Supplied lesson representation/oracle, not a running byte decoder.
        let path: [[u8; 1]; 8] = std::array::from_fn(|i| [(value >> (7 - i)) & 1]);
        let anchor = path.iter().fold(bits.clone(), |at, key| at.provision(key));
        let target = data.provision(&[b"shown-values", &[value]]);
        let source = data.provision(&[b"source-a", &[value]]);
        let first = record(&anchor, b"raw-byte-msb-first", &target, &source).unwrap();
        let source2 = data.provision(&[b"source-b", &[value]]);
        let added = record(&anchor, b"raw-byte-msb-first", &target, &source2).unwrap();
        assert!(first.candidate.same(&added.candidate));
        assert_eq!(inspect_evidence(&first.candidate).unwrap().len(), 2);
        let located = bits.locate(path.iter().map(|t| t.as_slice())).unwrap();
        heap::start();
        let resolved = resolve(&located, b"raw-byte-msb-first");
        let sample = heap::finish();
        match resolved.unwrap() {
            Selection::Unique { target: result, .. } => assert!(result.same(&target)),
            _ => panic!("expected shown reference"),
        }
        assert_eq!(sample.requests, 0);
        assert_eq!(sample.bytes, 0);
        assert!(matches!(
            resolve(&located, b"unicode-scalar"),
            Ok(Selection::Unknown)
        ));
    }
}

#[test]
fn first_record_keeps_reference_and_inverse_links() {
    let data = Trie::default();
    let anchor = data.provision(&[b"bits", &[0], &[1]]);
    let target = data.provision(&[b"values", b"opaque"]);
    let source = data.provision(&[b"sources", b"one"]);
    let result = record(&anchor, b"byte", &target, &source).unwrap();
    assert_eq!(result.change, Change::NewTarget);
    let (candidate, value) = unique(&anchor, b"byte");
    assert!(candidate.same(&result.candidate));
    assert!(value.same(&target));
    assert!(
        candidate
            .child(b"anchor")
            .unwrap()
            .follow()
            .unwrap()
            .same(&anchor)
    );
    assert!(result.evidence.follow().unwrap().same(&source));
    assert!(
        result
            .evidence
            .child(b"target")
            .unwrap()
            .follow()
            .unwrap()
            .same(&target)
    );
}
#[test]
fn equal_reference_adds_evidence_but_never_duplicates_the_candidate() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let target = data.provision(&[b"target"]);
    let source = data.provision(&[b"source"]);
    let original = record(&anchor, b"view", &target, &source).unwrap();
    for i in 0usize..128 {
        let source = data.provision(&[b"observations", &i.to_be_bytes()]);
        let result = record(&anchor, b"view", &target, &source).unwrap();
        assert_eq!(result.change, Change::AddedEvidence);
        assert!(result.candidate.same(&original.candidate));
        assert!(result.evidence.follow().unwrap().same(&source));
    }
    assert_eq!(inspect_candidates(&anchor, b"view").len(), 1);
    assert_eq!(inspect_evidence(&original.candidate).unwrap().len(), 129);
}
#[test]
fn exact_repetition_is_idempotent_and_allocates_nothing() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let target = data.provision(&[b"target"]);
    let source = data.provision(&[b"source"]);
    let original = record(&anchor, b"view", &target, &source).unwrap();
    heap::start();
    let result = record(&anchor, b"view", &target, &source);
    let sample = heap::finish();
    let result = result.unwrap();
    assert_eq!(result.change, Change::AlreadyRecorded);
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    assert!(result.candidate.same(&original.candidate));
    assert!(result.evidence.same(&original.evidence));
}
#[test]
fn distinct_addresses_remain_alternatives_even_with_identical_labels() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let first = data.provision(&[b"left", b"same-label"]);
    let second = data.provision(&[b"right", b"same-label"]);
    let source = data.provision(&[b"source"]);
    record(&anchor, b"view", &first, &source).unwrap();
    record(&anchor, b"view", &second, &source).unwrap();
    heap::start();
    let selected = resolve(&anchor, b"view");
    let sample = heap::finish();
    assert!(matches!(
        selected,
        Ok(Selection::Ambiguous { alternatives: 2 })
    ));
    assert_eq!(sample.requests, 0);
    assert_eq!(sample.bytes, 0);
    let candidates = inspect_candidates(&anchor, b"view");
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().any(|c| c.follow().unwrap().same(&first)));
    assert!(candidates.iter().any(|c| c.follow().unwrap().same(&second)));
}
#[test]
fn prepared_new_record_is_invisible_until_one_publication() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let target = data.provision(&[b"target"]);
    let source = data.provision(&[b"source"]);
    let pending = prepare(&anchor, b"view", &target, &source).unwrap();
    assert!(matches!(resolve(&anchor, b"view"), Ok(Selection::Unknown)));
    drop(pending);
    assert!(matches!(resolve(&anchor, b"view"), Ok(Selection::Unknown)));
    assert!(inspect_candidates(&anchor, b"view").is_empty());
    prepare(&anchor, b"view", &target, &source)
        .unwrap()
        .publish()
        .unwrap();
    assert!(unique(&anchor, b"view").1.same(&target));
}
#[test]
fn prepared_evidence_and_alternative_do_not_change_prior_visibility() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let target = data.provision(&[b"target"]);
    let other = data.provision(&[b"other"]);
    let source = data.provision(&[b"source"]);
    let new_source = data.provision(&[b"new-source"]);
    let old = record(&anchor, b"view", &target, &source).unwrap();
    let evidence = prepare(&anchor, b"view", &target, &new_source).unwrap();
    let alternative = prepare(&anchor, b"view", &other, &new_source).unwrap();
    assert_eq!(inspect_evidence(&old.candidate).unwrap().len(), 1);
    assert!(unique(&anchor, b"view").1.same(&target));
    evidence.publish().unwrap();
    assert_eq!(inspect_evidence(&old.candidate).unwrap().len(), 2);
    assert!(unique(&anchor, b"view").1.same(&target));
    alternative.publish().unwrap();
    assert!(matches!(
        resolve(&anchor, b"view"),
        Ok(Selection::Ambiguous { alternatives: 2 })
    ));
}
#[test]
fn interleaved_preparations_converge_without_losing_evidence() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let target = data.provision(&[b"target"]);
    let a = data.provision(&[b"a"]);
    let b = data.provision(&[b"b"]);
    let pa = prepare(&anchor, b"view", &target, &a).unwrap();
    let pb = prepare(&anchor, b"view", &target, &b).unwrap();
    let duplicate = prepare(&anchor, b"view", &target, &a).unwrap();
    let rb = pb.publish().unwrap();
    let ra = pa.publish().unwrap();
    let repeated = duplicate.publish().unwrap();
    assert_eq!(rb.change, Change::NewTarget);
    assert_eq!(ra.change, Change::AddedEvidence);
    assert_eq!(repeated.change, Change::AlreadyRecorded);
    assert!(rb.candidate.same(&ra.candidate));
    assert!(ra.candidate.same(&repeated.candidate));
    assert_eq!(inspect_evidence(&ra.candidate).unwrap().len(), 2);
}
#[test]
fn interleaved_distinct_targets_publish_complete_alternatives() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let a = data.provision(&[b"a"]);
    let b = data.provision(&[b"b"]);
    let source = data.provision(&[b"source"]);
    let pa = prepare(&anchor, b"view", &a, &source).unwrap();
    let pb = prepare(&anchor, b"view", &b, &source).unwrap();
    pa.publish().unwrap();
    assert!(unique(&anchor, b"view").1.same(&a));
    pb.publish().unwrap();
    assert!(matches!(
        resolve(&anchor, b"view"),
        Ok(Selection::Ambiguous { alternatives: 2 })
    ));
    for candidate in inspect_candidates(&anchor, b"view") {
        assert!(
            candidate
                .child(b"anchor")
                .unwrap()
                .follow()
                .unwrap()
                .same(&anchor)
        );
        assert_eq!(inspect_evidence(&candidate).unwrap().len(), 1);
    }
}

#[test]
fn view_names_and_arbitrary_address_tokens_do_not_merge() {
    let data = Trie::default();
    let literal = data.provision(&[b"a/b", b"", &[255, 0]]);
    let split = data.provision(&[b"a", b"b", b"", &[255, 0]]);
    let target = data.provision(&[b"target"]);
    let source = data.provision(&[b"source"]);
    record(&literal, b"encoding/one", &target, &source).unwrap();
    record(&literal, b"encoding/two", &target, &source).unwrap();
    record(&split, b"encoding/one", &target, &source).unwrap();
    let a = unique(&literal, b"encoding/one").0;
    let b = unique(&literal, b"encoding/two").0;
    let c = unique(&split, b"encoding/one").0;
    assert!(!a.same(&b));
    assert!(!a.same(&c));
    assert!(matches!(
        resolve(&literal, b"unknown"),
        Ok(Selection::Unknown)
    ));
}
#[test]
fn dangling_target_and_evidence_are_detected_not_overwritten() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let source = data.provision(&[b"source"]);
    {
        let owner = Trie::default();
        let target = owner.provision(&[b"temporary"]);
        record(&anchor, b"view", &target, &source).unwrap();
    }
    assert!(matches!(
        resolve(&anchor, b"view"),
        Err(RecordError::Reference(Fault::Dangling))
    ));
    let target = data.provision(&[b"target"]);
    let result = {
        let owner = Trie::default();
        let source = owner.provision(&[b"temporary"]);
        record(&anchor, b"other", &target, &source).unwrap()
    };
    assert!(matches!(result.evidence.follow(), Err(Fault::Dangling)));
}
#[test]
fn corrupted_identity_or_missing_schema_is_a_fault_not_a_new_record() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let target = data.provision(&[b"target"]);
    let wrong = data.provision(&[b"wrong"]);
    let source = data.provision(&[b"source"]);
    let original = record(&anchor, b"view", &target, &source).unwrap();
    original.candidate.point_to(&wrong);
    assert!(matches!(
        record(&anchor, b"view", &target, &source),
        Err(RecordError::IdentityMismatch)
    ));
    anchor.provision(b"holds").provision(b"malformed");
    assert!(matches!(
        resolve(&anchor, b"malformed"),
        Err(RecordError::Malformed)
    ));
}
#[test]
fn terminal_data_reference_is_not_an_instruction_to_halt_lookup() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let end = data.provision(&[b"end"]);
    end.terminate();
    let source = data.provision(&[b"source"]);
    record(&anchor, b"view", &end, &source).unwrap();
    assert!(unique(&anchor, b"view").1.same(&end));
}
#[test]
fn supplied_evidence_payload_remains_the_original_record_not_a_summary() {
    let data = Trie::default();
    let anchor = data.provision(&[b"anchor"]);
    let target = data.provision(&[b"target"]);
    let source = data.provision(&[b"raw-observations", b"source-17", &[1, 0, 1, 0, 0, 1, 0, 1]]);
    let result = record(&anchor, b"raw-byte", &target, &source).unwrap();
    assert_eq!(
        result.evidence.follow().unwrap().token(),
        Some([1, 0, 1, 0, 0, 1, 0, 1].as_slice())
    );
}
