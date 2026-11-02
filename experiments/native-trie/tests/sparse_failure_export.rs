//! Expected resource failure must retain actual partial native graph, not just panic text.
#[test]
fn workspace_failure_exports_partial_computation() {
    let path = std::env::temp_dir().join(format!("sparse-failure-{}.tsv", std::process::id()));
    let wire = |v: &str| vec![vec![v; 30].join(","); 30].join("/");
    std::fs::write(
        &path,
        format!("TRAIN\ttrain0\t{}\t{}\n", wire("0"), wire("4")),
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sparse_probe"))
        .arg(&path)
        .arg("--input-relations")
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("\"kind\":\"failure\"") && stdout.contains("\"error\":\"WorkspaceLimit\"")
    );
    assert!(!stdout.contains("\"kind\":\"prediction\""));
    let graph = String::from_utf8(out.stderr).unwrap();
    assert!(graph.starts_with("{\"kind\":\"native-graph-roots\""));
    assert!(graph.contains("\"kind\":\"native-node\""));
    assert!(!graph.contains("panicked"));
}

#[test]
fn generalized_worker_emits_exactly_one_primary_and_one_attempt_2() {
    fn fixture(palette: &[u8], height: usize, width: usize) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        let mut input = vec![vec![7; width]; height];
        input[0][..palette.len()].copy_from_slice(palette);
        let output = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| palette[(x + y) % palette.len()])
                    .collect()
            })
            .collect();
        (input, output)
    }
    fn wire(grid: &[Vec<u8>]) -> String {
        grid.iter()
            .map(|row| row.iter().map(u8::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join("/")
    }

    let path = std::env::temp_dir().join(format!("sparse-attempts-{}.tsv", std::process::id()));
    let mut records = Vec::new();
    for (index, palette) in [&[2, 8][..], &[1, 3, 6][..]].into_iter().enumerate() {
        let (input, output) = fixture(palette, 6 + index, 8 + index);
        records.push(format!(
            "TRAIN\ttrain{index}\t{}\t{}",
            wire(&input),
            wire(&output)
        ));
    }
    let (query, _) = fixture(&[5, 0, 8], 9, 11);
    records.push(format!("QUERY\tquery0\t{}", wire(&query)));
    std::fs::write(&path, records.join("\n") + "\n").unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sparse_probe"))
        .arg(&path)
        .arg("--cyclic-lattice-actions")
        .arg("--skip-graph-export")
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.matches("\"kind\":\"prediction\"").count(), 1);
    assert_eq!(stdout.matches("\"kind\":\"attempt-2\"").count(), 1);
    assert!(stdout.contains("\"status\":\"duplicate-attempt-1\""));
    assert!(stdout.contains("\"reason\":\"no-secondary-replay-valid-candidate\""));
}

#[test]
fn generalized_worker_scans_secondaries_after_primary_is_inapplicable() {
    fn fixture(palette: &[u8], height: usize, width: usize) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        let mut input = vec![vec![7; width]; height];
        input[0][..palette.len()].copy_from_slice(palette);
        let output = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| palette[(x + y) % palette.len()])
                    .collect()
            })
            .collect();
        (input, output)
    }
    fn wire(grid: &[Vec<u8>]) -> String {
        grid.iter()
            .map(|row| row.iter().map(u8::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join("/")
    }

    let path = std::env::temp_dir().join(format!(
        "sparse-primary-inapplicable-{}.tsv",
        std::process::id()
    ));
    let mut records = Vec::new();
    for (index, palette) in [&[2, 8][..], &[1, 3, 6][..]].into_iter().enumerate() {
        let (input, output) = fixture(palette, 6 + index, 8 + index);
        records.push(format!(
            "TRAIN\ttrain{index}\t{}\t{}",
            wire(&input),
            wire(&output)
        ));
    }
    let query = vec![vec![7; 11]; 9];
    records.push(format!("QUERY\tquery0\t{}", wire(&query)));
    std::fs::write(&path, records.join("\n") + "\n").unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sparse_probe"))
        .arg(&path)
        .arg("--cyclic-lattice-actions")
        .arg("--skip-graph-export")
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(
        "\"kind\":\"prediction\",\"id\":\"query0\",\"status\":\"no-matching-valid-relational-encoding\""
    ));
    assert_eq!(stdout.matches("\"kind\":\"attempt-2\"").count(), 1);
    assert!(stdout.contains("\"reason\":\"no-secondary-replay-valid-candidate\""));
    assert!(stdout.contains("\"all_demonstrations_replayed\":false"));
    assert!(!stdout.contains("\"reason\":\"primary-inapplicable\""));
}

#[test]
fn generalized_worker_emits_distinct_replay_valid_broad_attempt_2() {
    type Role = (u8, (usize, usize), (usize, usize));

    fn fixture(height: usize, width: usize, roles: &[Role]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        let mut input = vec![vec![0; width]; height];
        let mut output = input.clone();
        for &(color, first, second) in roles {
            input[first.0][first.1] = color;
            input[second.0][second.1] = color;
            if first.0 == second.0 {
                for cell in &mut output[first.0][first.1.min(second.1)..=first.1.max(second.1)] {
                    if *cell == 0 {
                        *cell = color;
                    }
                }
            } else {
                for row in output
                    .iter_mut()
                    .take(first.0.max(second.0) + 1)
                    .skip(first.0.min(second.0))
                {
                    if row[first.1] == 0 {
                        row[first.1] = color;
                    }
                }
            }
        }
        for &(color, first, second) in roles {
            output[first.0][first.1] = color;
            output[second.0][second.1] = color;
        }
        (input, output)
    }

    fn wire(grid: &[Vec<u8>]) -> String {
        grid.iter()
            .map(|row| row.iter().map(u8::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join("/")
    }

    let path =
        std::env::temp_dir().join(format!("sparse-broad-attempts-{}.tsv", std::process::id()));
    let mut records = Vec::new();
    for (index, (input, output)) in [
        fixture(12, 12, &[(3, (2, 1), (2, 8)), (7, (4, 5), (10, 5))]),
        fixture(15, 10, &[(2, (1, 2), (12, 2)), (9, (8, 0), (8, 9))]),
    ]
    .into_iter()
    .enumerate()
    {
        records.push(format!(
            "TRAIN\ttrain{index}\t{}\t{}",
            wire(&input),
            wire(&output)
        ));
    }
    let (query, _) = fixture(11, 11, &[(8, (1, 3), (9, 3)), (4, (5, 1), (5, 9))]);
    records.push(format!("QUERY\tquery0\t{}", wire(&query)));
    std::fs::write(&path, records.join("\n") + "\n").unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sparse_probe"))
        .arg(&path)
        .arg("--cyclic-lattice-actions")
        .arg("--skip-graph-export")
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.matches("\"kind\":\"prediction\"").count(), 1);
    assert_eq!(stdout.matches("\"kind\":\"attempt-2\"").count(), 1);
    assert!(stdout.contains("\"status\":\"distinct-replay-valid-candidate\""));
    assert!(stdout.contains("\"candidate_origin\":\"broad-replay-valid\""));
    assert!(stdout.contains("\"candidate_index\":1"));
    assert!(stdout.contains("\"candidate_address\":\"ordered-replay-valid-candidates/1\""));
    assert!(stdout.contains("\"all_demonstrations_replayed\":true"));
    assert!(stdout.contains("\"replay_valid\":true,\"independent\":true"));
}
