use native_trie_seed::{disk_arc::DiskArc, disk_trie::DiskFault};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "disk-arc-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn canonical_digest_reuse_verifies_complete_typed_bytes() {
    let path = path();
    let arc = DiskArc::create(&path).unwrap();
    let evidence = arc.store_root().provision(b"evidence").unwrap();
    let grid = vec![vec![1, 2], vec![3, 4]];
    arc.encode(b"input", b"first", &grid, &evidence).unwrap();
    let traces = path.join("arc-canonical-traces");
    let trace = fs::read_dir(&traces)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(trace, b"corrupt canonical bytes").unwrap();
    assert_eq!(arc.verify_canonical_traces(), Err(DiskFault::Corrupt));
    assert_eq!(
        arc.encode(b"input", b"second", &grid, &evidence).err(),
        Some(DiskFault::Corrupt)
    );
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn namespaced_states_and_four_masks_survive_reopen() {
    let path = path();
    let arc = DiskArc::create(&path).unwrap();
    let evidence = arc.store_root().provision(b"evidence").unwrap();
    let input = vec![vec![0, 1, 0], vec![1, 0, 1], vec![0, 1, 0]];
    let output = vec![vec![0, 1, 0], vec![1, 6, 1], vec![0, 1, 0]];
    let encoded_input = arc
        .encode(b"input", b"example-1", &input, &evidence)
        .unwrap();
    let encoded_output = arc
        .encode(b"output", b"example-1", &output, &evidence)
        .unwrap();
    assert_eq!((encoded_input.height, encoded_input.width), (3, 3));
    assert!(!encoded_input.address.same(&encoded_output.address));
    let replayed_input = arc
        .encode(b"input", b"example-1-replay-input", &input, &evidence)
        .unwrap();
    assert!(
        encoded_input
            .address
            .child(b"canonical-state")
            .unwrap()
            .unwrap()
            .follow()
            .unwrap()
            .same(
                &replayed_input
                    .address
                    .child(b"canonical-state")
                    .unwrap()
                    .unwrap()
                    .follow()
                    .unwrap()
            )
    );
    let executed = arc
        .execute_prediction(b"example-1-execution", &output, &evidence)
        .unwrap();
    assert_eq!(executed, output);
    assert_eq!(
        arc.execute_prediction(b"example-1-execution", &output, &evidence),
        Err(DiskFault::AlreadyComplete)
    );
    arc.store_root()
        .provision(b"arc")
        .unwrap()
        .provision(b"executions")
        .unwrap()
        .provision(b"interrupted-execution")
        .unwrap()
        .provision(b"partial-evidence")
        .unwrap();
    assert_eq!(
        arc.execute_prediction(b"interrupted-execution", &output, &evidence),
        Err(DiskFault::Partial)
    );
    let mask = arc
        .derive_masks(b"example-1", &input, &output, &evidence)
        .unwrap();
    let executed_mask = arc
        .derive_masks(b"example-1-executed", &input, &executed, &evidence)
        .unwrap();
    assert!(arc.masks_equal(&mask, &executed_mask).unwrap());
    let replay = arc
        .derive_masks(b"example-1-replay", &input, &output, &evidence)
        .unwrap();
    assert!(arc.masks_equal(&mask, &replay).unwrap());
    let unchanged = arc
        .derive_masks(b"example-1-wrong", &input, &input, &evidence)
        .unwrap();
    assert!(!arc.masks_equal(&mask, &unchanged).unwrap());
    for (kind, component) in [
        (b"individual-position".as_slice(), &mask.individual_position),
        (b"neighbourhood-position", &mask.neighbourhood_position),
        (b"individual-color", &mask.individual_color),
        (b"neighbourhood-color", &mask.neighbourhood_color),
    ] {
        assert!(component.is_terminal().unwrap());
        assert!(
            mask.address
                .child(kind)
                .unwrap()
                .unwrap()
                .follow()
                .unwrap()
                .same(component)
        );
    }
    drop(arc);

    let reopened = DiskArc::open(&path).unwrap();
    assert!(reopened.verify_canonical_traces().unwrap() >= 2);
    assert!(
        reopened
            .store_root()
            .locate([b"arc".as_slice(), b"states", b"input", b"example-1"])
            .unwrap()
            .is_some()
    );
    assert!(
        reopened
            .store_root()
            .locate([
                b"arc".as_slice(),
                b"masks",
                b"example-1",
                b"individual-color"
            ])
            .unwrap()
            .is_some()
    );
    assert!(
        reopened
            .store_root()
            .locate([
                b"arc".as_slice(),
                b"executions",
                b"example-1-execution",
                b"output-trace"
            ])
            .unwrap()
            .is_some()
    );
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn puzzle_prefixes_isolate_and_retain_closed_learning_in_one_trie() {
    let path = path();
    let first_grid = vec![vec![0, 1], vec![1, 0]];
    let second_grid = vec![vec![2, 0], vec![0, 2]];

    let first = DiskArc::provision_puzzle(&path, b"puzzle-a").unwrap();
    let first_evidence = first.store_root().provision(b"first-evidence").unwrap();
    first
        .encode(b"input", b"shared-episode", &first_grid, &first_evidence)
        .unwrap();
    first
        .execute_prediction(b"shared-run", &first_grid, &first_evidence)
        .unwrap();
    first.close_puzzle().unwrap();
    assert_eq!(
        first
            .encode(b"input", b"late-write", &first_grid, &first_evidence)
            .err(),
        Some(DiskFault::AlreadyComplete)
    );
    drop(first);

    let second = DiskArc::provision_puzzle(&path, b"puzzle-b").unwrap();
    let second_evidence = second.store_root().provision(b"second-evidence").unwrap();
    second
        .encode(b"input", b"shared-episode", &second_grid, &second_evidence)
        .unwrap();
    second
        .execute_prediction(b"shared-run", &second_grid, &second_evidence)
        .unwrap();
    second.close_puzzle().unwrap();
    drop(second);

    let first = DiskArc::open_puzzle(&path, b"puzzle-a").unwrap();
    let root = first.store_root();
    let first_state = root
        .locate([
            b"arc".as_slice(),
            b"puzzles",
            b"puzzle-a",
            b"states",
            b"input",
            b"shared-episode",
        ])
        .unwrap()
        .unwrap();
    let second_state = root
        .locate([
            b"arc".as_slice(),
            b"puzzles",
            b"puzzle-b",
            b"states",
            b"input",
            b"shared-episode",
        ])
        .unwrap()
        .unwrap();
    assert!(!first_state.same(&second_state));
    assert!(
        root.locate([b"arc".as_slice(), b"puzzles", b"puzzle-a", b"complete"])
            .unwrap()
            .is_some()
    );
    assert!(
        root.locate([b"arc".as_slice(), b"puzzles", b"puzzle-b", b"complete"])
            .unwrap()
            .is_some()
    );
    assert!(first.verify_canonical_traces().unwrap() >= 2);
    fs::remove_dir_all(path).unwrap();
}
