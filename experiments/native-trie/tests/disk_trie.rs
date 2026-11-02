use native_trie_seed::disk_trie::{
    DiskAttemptStatus, DiskEpisodes, DiskFault, DiskMachine, DiskOutcome, DiskReceipts, DiskTrie,
    OutcomeAssessment, attempt_all_isolated, install_disk,
};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn store(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "native-trie-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn disk_edges_references_and_terminal_state_survive_reopen() {
    let path = store("reopen");
    let trie = DiskTrie::create(&path).unwrap();
    let source = trie.provision(&[b"input", b"source"]).unwrap();
    let target = trie.provision(&[b"construction", b"target"]).unwrap();
    source.point_to(&target).unwrap();
    target.provision(b"done").unwrap().terminate().unwrap();
    let target_key = target.identity_key();
    drop(trie);

    let reopened = DiskTrie::open(&path).unwrap();
    let source = reopened
        .root()
        .locate([b"input".as_slice(), b"source"])
        .unwrap()
        .unwrap();
    assert_eq!(source.follow().unwrap().identity_key(), target_key);
    let done = reopened
        .root()
        .locate([b"construction".as_slice(), b"target", b"done"])
        .unwrap()
        .unwrap();
    assert!(matches!(
        done.follow(),
        Err(DiskFault::TerminalUsedAsReference)
    ));
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn native_copy_and_continuation_execute_from_disk_after_reopen() {
    let path = store("machine");
    let machine = DiskMachine::create(&path).unwrap();
    let source_value = machine.states().provision(b"source-value").unwrap();
    source_value.terminate().unwrap();
    let source_slot = machine.states().provision(b"source-slot").unwrap();
    source_slot.point_to(&source_value).unwrap();
    let target_slot = machine.output().provision(b"target-slot").unwrap();
    let halt = machine.states().provision(b"halt").unwrap();
    halt.terminate().unwrap();
    let next = machine.states().provision(b"next-cell").unwrap();
    next.point_to(&halt).unwrap();
    let instruction = machine.states().provision(b"copy-instruction").unwrap();
    install_disk(&instruction, &source_slot, &target_slot, &next).unwrap();
    drop(machine);

    let mut reopened = DiskMachine::open(&path).unwrap();
    let instruction = reopened
        .states()
        .child(b"copy-instruction")
        .unwrap()
        .unwrap();
    reopened.enter(&instruction).unwrap();
    assert_eq!(reopened.cycle().unwrap(), DiskOutcome::Advanced);
    assert_eq!(reopened.cycle().unwrap(), DiskOutcome::Halted);
    let copied = reopened
        .output()
        .child(b"target-slot")
        .unwrap()
        .unwrap()
        .follow()
        .unwrap();
    assert!(copied.is_terminal().unwrap());
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn episodic_retrieval_returns_every_candidate_and_evidence_only_contexts() {
    let path = store("episodes");
    let trie = DiskTrie::create(&path).unwrap();
    let namespace = trie.root().provision(b"arc").unwrap();
    let episodes = DiskEpisodes::new(&namespace).unwrap();
    let evidence_a = trie.provision(&[b"evidence", b"a"]).unwrap();
    let evidence_b = trie.provision(&[b"evidence", b"b"]).unwrap();
    let construction_a = trie.provision(&[b"constructions", b"a"]).unwrap();
    let construction_b = trie.provision(&[b"constructions", b"b"]).unwrap();
    episodes
        .record(&[b"cue", b"ambiguous"], Some(&construction_a), &evidence_a)
        .unwrap();
    episodes
        .record(&[b"cue", b"ambiguous"], Some(&construction_b), &evidence_b)
        .unwrap();
    episodes
        .record(&[b"cue", b"evidence-only"], None, &evidence_a)
        .unwrap();

    let ambiguous = episodes.retrieve(&[b"cue", b"ambiguous"]).unwrap().unwrap();
    assert_eq!(ambiguous.constructions.len(), 2);
    assert_eq!(ambiguous.evidence.len(), 2);
    let evidence_only = episodes
        .retrieve(&[b"cue", b"evidence-only"])
        .unwrap()
        .unwrap();
    assert!(evidence_only.constructions.is_empty());
    assert_eq!(evidence_only.evidence.len(), 1);
    assert!(episodes.retrieve(&[b"missing"]).unwrap().is_none());
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn every_retrieved_candidate_is_attempted_and_effects_are_observable() {
    let path = store("attempts");
    let mut machine = DiskMachine::create(&path).unwrap();
    let namespace = machine.root().provision(b"attempt-profile").unwrap();
    let episodes = DiskEpisodes::new(&namespace).unwrap();
    let evidence = namespace.provision(b"evidence").unwrap();
    let halt = machine.states().provision(b"halt-attempts").unwrap();
    halt.terminate().unwrap();
    let malformed = namespace
        .provision(b"constructions")
        .unwrap()
        .provision(b"malformed")
        .unwrap();

    let build = |name: &[u8], mutate: bool| {
        let construction = namespace
            .provision(b"constructions")
            .unwrap()
            .provision(name)
            .unwrap();
        let source = construction.provision(b"source-value").unwrap();
        source.terminate().unwrap();
        let effect = construction.provision(b"effect-slot").unwrap();
        if !mutate {
            effect.terminate().unwrap();
        }
        let next = construction.provision(b"next").unwrap();
        next.point_to(&halt).unwrap();
        let instruction = construction.provision(b"instruction").unwrap();
        install_disk(&instruction, &source, &effect, &next).unwrap();
        construction
            .provision(b"entry")
            .unwrap()
            .point_to(&instruction)
            .unwrap();
        construction
            .provision(b"effect")
            .unwrap()
            .point_to(&effect)
            .unwrap();
        construction
    };
    let mutating = build(b"mutating", true);
    let unchanged = build(b"unchanged", false);
    episodes
        .record(&[b"cue"], Some(&malformed), &evidence)
        .unwrap();
    episodes
        .record(&[b"cue"], Some(&mutating), &evidence)
        .unwrap();
    episodes
        .record(&[b"cue"], Some(&unchanged), &evidence)
        .unwrap();
    let retrieval = episodes.retrieve(&[b"cue"]).unwrap().unwrap();
    let attempts = attempt_all_isolated(&mut machine, &retrieval.constructions, 4).unwrap();
    assert_eq!(attempts.len(), 3);
    assert_eq!(
        attempts[0].status,
        DiskAttemptStatus::Fault(DiskFault::Corrupt)
    );
    assert_eq!(
        attempts
            .iter()
            .filter(|attempt| attempt.status == DiskAttemptStatus::Completed)
            .count(),
        2
    );
    assert_eq!(
        attempts
            .iter()
            .filter(|attempt| attempt.before != attempt.after)
            .count(),
        1
    );
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn delayed_outcome_remains_associated_with_prediction_and_context() {
    let path = store("receipts");
    let trie = DiskTrie::create(&path).unwrap();
    let namespace = trie.root().provision(b"agent").unwrap();
    let receipts = DiskReceipts::new(&namespace).unwrap();
    let context = trie.provision(&[b"contexts", b"request-1"]).unwrap();
    let prediction = trie.provision(&[b"predictions", b"choice-b"]).unwrap();
    let evidence = trie.provision(&[b"evidence", b"structural-match"]).unwrap();
    receipts
        .issue(
            b"receipt-1",
            &context,
            &prediction,
            &[evidence.clone(), evidence],
            b"recorded-first-candidate",
        )
        .unwrap();
    drop(receipts);
    drop(trie);

    let reopened = DiskTrie::open(&path).unwrap();
    let namespace = reopened.root().child(b"agent").unwrap().unwrap();
    let receipts = DiskReceipts::new(&namespace).unwrap();
    receipts
        .associate_outcome(
            b"receipt-1",
            b"later-llm-turn",
            OutcomeAssessment::Unclassified,
        )
        .unwrap();
    let receipt = receipts.receipt(b"receipt-1").unwrap().unwrap();
    assert!(
        receipt
            .child(b"context")
            .unwrap()
            .unwrap()
            .follow()
            .unwrap()
            .same(&context)
    );
    assert!(
        receipt
            .child(b"prediction")
            .unwrap()
            .unwrap()
            .follow()
            .unwrap()
            .same(&prediction)
    );
    assert_eq!(
        receipt
            .child(b"evidence")
            .unwrap()
            .unwrap()
            .children()
            .unwrap()
            .len(),
        1
    );
    assert!(
        receipt
            .locate([
                b"outcomes".as_slice(),
                b"later-llm-turn",
                b"assessment",
                b"unclassified"
            ])
            .unwrap()
            .is_some()
    );
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn handles_do_not_own_a_host_child_map() {
    let path = store("addressing");
    let trie = DiskTrie::create(&path).unwrap();
    let first = trie.provision(&[b"episode", b"cue"]).unwrap();
    let again = trie.provision(&[b"episode", b"cue"]).unwrap();
    assert!(first.same(&again));
    assert_eq!(first.token().unwrap(), b"cue");
    assert!(path.join("nodes").is_dir());
    drop(trie);
    fs::remove_dir_all(path).unwrap();
}
