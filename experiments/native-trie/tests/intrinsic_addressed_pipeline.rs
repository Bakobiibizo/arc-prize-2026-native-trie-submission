use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::SparseRewrite,
};

type Pixels = Vec<Vec<Option<u8>>>;

fn grid(grids: &Grids, id: &[u8], side: Side, pixels: &Pixels) -> Grid {
    let source = grids
        .source(id, b"intrinsic addressed pipeline test")
        .unwrap();
    grids.admit(id, side, pixels, &source).unwrap()
}

#[test]
fn compatibility_entry_routes_directly_to_intrinsic_constructions() {
    let mut grids = Grids::new();
    let mut learner = SparseRewrite::new(&grids);
    let input = vec![
        vec![Some(0); 4],
        vec![Some(0), Some(1), Some(1), Some(0)],
        vec![Some(0); 4],
    ];
    let output = vec![
        vec![Some(0); 4],
        vec![Some(0), Some(1), Some(1), Some(1)],
        vec![Some(0); 4],
    ];
    let training_input = grid(&grids, b"train-in", Side::TrainingInput, &input);
    let training_output = grid(&grids, b"train-out", Side::TrainingOutput, &output);
    let lesson = learner
        .derive_groups(b"demonstration", &training_input, &training_output)
        .unwrap();
    let program = learner.agree_cyclic_actions(&[lesson]).unwrap().unwrap();
    assert!(
        program
            .root
            .child(b"episode-transduction-program")
            .is_some()
    );
    assert!(program.root.child(b"cyclic-lattice-program").is_none());

    let query_pixels = vec![
        vec![Some(0); 4],
        vec![Some(0), Some(1), Some(1), Some(0)],
        vec![Some(0); 4],
    ];
    let expected = vec![
        vec![Some(0); 4],
        vec![Some(0), Some(1), Some(1), Some(1)],
        vec![Some(0); 4],
    ];
    let query = grid(&grids, b"query", Side::QueryInput, &query_pixels);
    assert!(learner.source_pair_applicable(&query, &program).unwrap());
    assert_eq!(
        learner
            .apply_groups(&mut grids, &query, &program)
            .unwrap()
            .inspect()
            .unwrap(),
        expected
    );
    let disk_root = learner.persistent_arc_store_root().unwrap();
    let executions = disk_root
        .locate([b"arc".as_slice(), b"executions"])
        .unwrap()
        .unwrap()
        .children()
        .unwrap();
    assert_eq!(executions.len(), 2);
    assert!(
        executions
            .iter()
            .all(|execution| execution.child(b"complete").unwrap().is_some())
    );
    let states = disk_root
        .locate([b"arc".as_slice(), b"states"])
        .unwrap()
        .unwrap();
    assert_eq!(
        states
            .child(b"input")
            .unwrap()
            .unwrap()
            .children()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        states
            .child(b"output")
            .unwrap()
            .unwrap()
            .children()
            .unwrap()
            .len(),
        3
    );
    let masks = disk_root
        .locate([b"arc".as_slice(), b"masks"])
        .unwrap()
        .unwrap()
        .children()
        .unwrap();
    assert_eq!(masks.len(), 3);
    let receipts = disk_root
        .child(b"receipts")
        .unwrap()
        .unwrap()
        .children()
        .unwrap();
    assert_eq!(receipts.len(), 1);
    assert!(
        receipts[0]
            .child(b"context")
            .unwrap()
            .unwrap()
            .follow()
            .is_ok()
    );
    assert!(
        receipts[0]
            .child(b"prediction")
            .unwrap()
            .unwrap()
            .follow()
            .is_ok()
    );
    assert!(
        receipts[0]
            .locate([
                b"outcomes".as_slice(),
                b"execution-complete",
                b"assessment",
                b"unclassified",
            ])
            .unwrap()
            .is_some()
    );
}

#[test]
fn later_example_is_held_out_before_its_evidence_is_admitted() {
    let grids = Grids::new();
    let mut learner = SparseRewrite::new(&grids);
    let first_input = vec![
        vec![Some(0); 5],
        vec![Some(0), Some(1), Some(1), Some(0), Some(0)],
        vec![Some(0); 5],
    ];
    let first_output = vec![
        vec![Some(0); 5],
        vec![Some(0), Some(1), Some(1), Some(1), Some(0)],
        vec![Some(0); 5],
    ];
    let second_input = vec![
        vec![Some(0); 6],
        vec![Some(0), Some(0), Some(1), Some(1), Some(0), Some(0)],
        vec![Some(0); 6],
    ];
    let second_output = vec![
        vec![Some(0); 6],
        vec![Some(0), Some(0), Some(1), Some(1), Some(1), Some(0)],
        vec![Some(0); 6],
    ];
    let first = learner
        .derive_groups(
            b"first",
            &grid(&grids, b"first-in", Side::TrainingInput, &first_input),
            &grid(&grids, b"first-out", Side::TrainingOutput, &first_output),
        )
        .unwrap();
    let second = learner
        .derive_groups(
            b"second",
            &grid(&grids, b"second-in", Side::TrainingInput, &second_input),
            &grid(&grids, b"second-out", Side::TrainingOutput, &second_output),
        )
        .unwrap();
    let program = learner
        .agree_cyclic_actions(&[first, second])
        .unwrap()
        .unwrap();
    assert!(program.root.child(b"teaching-continuation").is_some());
}

#[test]
fn held_out_mask_mismatch_rejects_before_admission() {
    let grids = Grids::new();
    let mut learner = SparseRewrite::new(&grids);
    let input = vec![
        vec![Some(0); 4],
        vec![Some(0), Some(1), Some(1), Some(0)],
        vec![Some(0); 4],
    ];
    let pink = vec![
        vec![Some(0); 4],
        vec![Some(0), Some(1), Some(1), Some(6)],
        vec![Some(0); 4],
    ];
    let red = vec![
        vec![Some(0); 4],
        vec![Some(0), Some(1), Some(1), Some(2)],
        vec![Some(0); 4],
    ];
    let first = learner
        .derive_groups(
            b"first",
            &grid(&grids, b"mismatch-first-in", Side::TrainingInput, &input),
            &grid(&grids, b"mismatch-first-out", Side::TrainingOutput, &pink),
        )
        .unwrap();
    let second = learner
        .derive_groups(
            b"second",
            &grid(&grids, b"mismatch-second-in", Side::TrainingInput, &input),
            &grid(&grids, b"mismatch-second-out", Side::TrainingOutput, &red),
        )
        .unwrap();
    assert!(
        learner
            .agree_cyclic_actions(&[first, second])
            .unwrap()
            .is_none()
    );
}

#[test]
fn missing_intrinsic_route_returns_unknown_without_legacy_fallback() {
    let grids = Grids::new();
    let mut learner = SparseRewrite::new(&grids);
    let input = vec![
        vec![Some(0); 3],
        vec![Some(1), Some(0), Some(1)],
        vec![Some(0); 3],
    ];
    let output = vec![
        vec![Some(0); 3],
        vec![Some(1), Some(6), Some(1)],
        vec![Some(0); 3],
    ];
    let training_input = grid(&grids, b"train-in", Side::TrainingInput, &input);
    let training_output = grid(&grids, b"train-out", Side::TrainingOutput, &output);
    let lesson = learner
        .derive_groups(b"demonstration", &training_input, &training_output)
        .unwrap();
    let program = learner.agree_cyclic_actions(&[lesson]).unwrap().unwrap();
    let unrelated = grid(
        &grids,
        b"unrelated",
        Side::QueryInput,
        &vec![vec![Some(2)], vec![Some(2)]],
    );
    assert!(
        !learner
            .source_pair_applicable(&unrelated, &program)
            .unwrap()
    );
}
