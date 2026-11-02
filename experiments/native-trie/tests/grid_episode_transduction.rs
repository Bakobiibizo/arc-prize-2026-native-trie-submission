use native_trie_seed::{
    Trie,
    disk_arc::DiskArc,
    episode_transduction::{LocalEpisodeTransducer, TransductionError},
};
use std::time::{SystemTime, UNIX_EPOCH};

fn values(trie: &Trie, grid: &[Vec<u8>]) -> Vec<native_trie_seed::Address> {
    grid.iter()
        .flatten()
        .map(|value| trie.provision(&[b"colors", &[*value]]))
        .collect()
}
fn inspect(values: &[native_trie_seed::Address], width: usize) -> Vec<Vec<u8>> {
    values
        .chunks(width)
        .map(|row| row.iter().map(|value| value.token().unwrap()[0]).collect())
        .collect()
}
fn motif(height: usize, width: usize, centers: &[(usize, usize)], result: bool) -> Vec<Vec<u8>> {
    let mut grid = vec![vec![0; width]; height];
    for &(cy, cx) in centers {
        for row in grid.iter_mut().take(cy + 2).skip(cy - 1) {
            for cell in row.iter_mut().take(cx + 2).skip(cx - 1) {
                *cell = 1;
            }
        }
        grid[cy][cx] = if result { 6 } else { 0 };
    }
    grid
}

#[test]
fn singleton_relative_stencil_transfers_to_translated_anchor() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let mut train_input = vec![vec![0; 5]; 5];
    train_input[2][2] = 2;
    let mut train_output = train_input.clone();
    for (y, x) in [(1, 1), (1, 3), (3, 1), (3, 3)] {
        train_output[y][x] = 4;
    }
    learner
        .teach_grid(
            5,
            5,
            &values(&colors, &train_input),
            &values(&colors, &train_output),
        )
        .unwrap();
    let mut query = vec![vec![0; 6]; 6];
    query[3][3] = 2;
    let mut expected = query.clone();
    for (y, x) in [(2, 2), (2, 4), (4, 2), (4, 4)] {
        expected[y][x] = 4;
    }
    let path = std::env::temp_dir().join(format!(
        "persistent-arc-execution-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let disk = DiskArc::create(&path).unwrap();
    let evidence = disk.store_root().provision(b"query-evidence").unwrap();
    learner
        .validate_grid_persistent(
            &disk,
            b"singleton-training-validation",
            &evidence,
            (5, 5),
            &values(&colors, &train_input),
            &values(&colors, &train_output),
        )
        .unwrap();
    assert_eq!(
        learner.validate_grid_persistent(
            &disk,
            b"singleton-wrong-validation",
            &evidence,
            (5, 5),
            &values(&colors, &train_input),
            &values(&colors, &train_input),
        ),
        Err(TransductionError::ValidationMismatch)
    );
    let result = learner
        .transduce_grid_persistent(
            &disk,
            b"singleton-query",
            &evidence,
            6,
            6,
            &values(&colors, &query),
        )
        .unwrap();
    assert_eq!(result, expected);
    drop(disk);
    let reopened = DiskArc::open(&path).unwrap();
    for path in [
        vec![
            b"arc".as_slice(),
            b"executions",
            b"singleton-query",
            b"complete",
        ],
        vec![b"arc".as_slice(), b"states", b"input", b"singleton-query"],
        vec![b"arc".as_slice(), b"states", b"output", b"singleton-query"],
        vec![b"arc".as_slice(), b"masks", b"singleton-query"],
    ] {
        assert!(reopened.store_root().locate(path).unwrap().is_some());
    }
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn enclosed_background_region_fill_transfers_across_hole_sizes() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let train_input = vec![vec![3, 3, 3], vec![3, 0, 3], vec![3, 3, 3]];
    let train_output = vec![vec![3, 3, 3], vec![3, 4, 3], vec![3, 3, 3]];
    learner
        .teach_grid(
            3,
            3,
            &values(&colors, &train_input),
            &values(&colors, &train_output),
        )
        .unwrap();
    let query = vec![
        vec![3, 3, 3, 3],
        vec![3, 0, 0, 3],
        vec![3, 0, 0, 3],
        vec![3, 3, 3, 3],
    ];
    let expected = vec![
        vec![3, 3, 3, 3],
        vec![3, 4, 4, 3],
        vec![3, 4, 4, 3],
        vec![3, 3, 3, 3],
    ];
    let result = learner
        .transduce_grid(4, 4, &values(&colors, &query))
        .unwrap();
    assert_eq!(inspect(&result, 4), expected);
}

#[test]
fn component_group_cue_erases_marker_and_recolors_different_target_shape() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let train_input = vec![
        vec![8, 8, 0, 0, 0, 0],
        vec![8, 8, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![1, 0, 0, 0, 0, 0],
        vec![0, 1, 0, 0, 0, 0],
    ];
    let train_output = vec![
        vec![7, 7, 0, 0, 0, 0],
        vec![7, 7, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
    ];
    learner
        .teach_grid(
            6,
            6,
            &values(&colors, &train_input),
            &values(&colors, &train_output),
        )
        .unwrap();

    let query = vec![
        vec![8, 8, 8, 0, 0, 0],
        vec![8, 8, 8, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 1, 0, 0],
        vec![0, 0, 0, 0, 1, 0],
        vec![0, 0, 0, 0, 0, 0],
    ];
    let expected = vec![
        vec![7, 7, 7, 0, 0, 0],
        vec![7, 7, 7, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0],
    ];
    let result = learner
        .transduce_grid(6, 6, &values(&colors, &query))
        .unwrap();
    assert_eq!(inspect(&result, 6), expected);
}

#[test]
fn ambiguous_context_attempts_every_construction_before_abstaining() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let input = motif(3, 3, &[(1, 1)], false);
    let output_six = motif(3, 3, &[(1, 1)], true);
    let mut output_seven = output_six.clone();
    output_seven[1][1] = 7;
    learner
        .teach_grid(
            3,
            3,
            &values(&colors, &input),
            &values(&colors, &output_six),
        )
        .unwrap();
    learner
        .teach_grid(
            3,
            3,
            &values(&colors, &input),
            &values(&colors, &output_seven),
        )
        .unwrap();
    let before = learner.applications;
    assert!(matches!(
        learner.transduce_grid(3, 3, &values(&colors, &input)),
        Err(TransductionError::Ambiguous)
    ));
    assert!(learner.applications >= before + 2);
}

#[test]
fn eight_direction_context_induces_validates_and_applies_two_dimensional_construction() {
    let colors = Trie::default();
    let mut learner = LocalEpisodeTransducer::default();
    let train_input = values(&colors, &motif(3, 3, &[(1, 1)], false));
    let train_output = values(&colors, &motif(3, 3, &[(1, 1)], true));
    assert_eq!(
        learner
            .teach_grid(3, 3, &train_input, &train_output)
            .unwrap(),
        1
    );

    let validation_input = values(&colors, &motif(5, 5, &[(2, 2)], false));
    let validation_output = values(&colors, &motif(5, 5, &[(2, 2)], true));
    learner
        .validate_grid(5, 5, &validation_input, &validation_output)
        .unwrap();

    let query = values(&colors, &motif(7, 7, &[(2, 2), (4, 4)], false));
    let expected = motif(7, 7, &[(2, 2), (4, 4)], true);
    let result = learner.transduce_grid(7, 7, &query).unwrap();
    assert_eq!(inspect(&result, 7), expected);
}
