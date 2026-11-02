use native_trie_seed::{
    arc_grids::{Grids, Side},
    positional::Error,
    sparse_rewrite::SparseRewrite,
};

fn admit(grids: &Grids, id: &[u8], cells: &[Vec<Option<u8>>]) -> native_trie_seed::arc_grids::Grid {
    let source = grids.source(id, b"native axis profile control").unwrap();
    grids
        .admit(id, Side::TrainingInput, cells, &source)
        .unwrap()
}

#[test]
fn actual_port_walk_proves_complete_supplied_line_and_rejects_interruption() {
    let grids = Grids::new();
    let full = admit(&grids, b"full", &[vec![Some(8); 4]]);
    let interrupted = admit(
        &grids,
        b"interrupted",
        &[vec![Some(8), Some(8), Some(3), Some(8)]],
    );
    let expected = full
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let mut learner = SparseRewrite::new(&grids);
    let proof = learner
        .prove_native_full_line(
            &full,
            &full.cell(0, 0).unwrap(),
            b"east",
            b"west",
            &expected,
            b"supplied-separator",
        )
        .unwrap();
    assert!(proof.belongs_to(&learner));
    assert!(proof.root.child(b"complete").is_some());
    assert_eq!(proof.visited_count(), 4);

    let paired = admit(
        &grids,
        b"paired-lines",
        &[vec![Some(8); 4], vec![Some(2); 4]],
    );
    let separator_value = paired
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let ground_value = paired
        .cell(1, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let separator = learner
        .prove_native_full_line(
            &paired,
            &paired.cell(0, 0).unwrap(),
            b"east",
            b"west",
            &separator_value,
            b"supplied-separator",
        )
        .unwrap();
    let ground = learner
        .prove_native_full_line(
            &paired,
            &paired.cell(1, 0).unwrap(),
            b"east",
            b"west",
            &ground_value,
            b"supplied-ground",
        )
        .unwrap();
    let bridge = learner
        .prove_native_line_bridge(&paired, &separator, &ground, b"east")
        .unwrap();
    assert!(bridge.belongs_to(&learner));
    assert!(bridge.root.child(b"complete").is_some());
    assert_eq!(bridge.pair_count(), 4);
    assert!(matches!(
        learner.prove_native_line_bridge(&paired, &separator, &ground, b"south"),
        Err(Error::NotReady)
    ));

    let activated = admit(
        &grids,
        b"activated-axis",
        &[
            vec![Some(0), Some(0), Some(0), Some(8)],
            vec![Some(4), Some(0), Some(0), Some(8)],
            vec![Some(0), Some(0), Some(0), Some(8)],
        ],
    );
    let background_value = activated
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let marker_value = activated
        .cell(1, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let separator_value = activated
        .cell(0, 3)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let vertical_separator = learner
        .prove_native_full_line(
            &activated,
            &activated.cell(0, 3).unwrap(),
            b"south",
            b"north",
            &separator_value,
            b"supplied-separator",
        )
        .unwrap();
    let activation = learner
        .prove_native_marker_activation(
            &activated,
            &activated.cell(1, 0).unwrap(),
            b"east",
            (&marker_value, &background_value, &separator_value),
            &vertical_separator,
        )
        .unwrap();
    assert!(activation.belongs_to(&learner));
    assert!(activation.root.child(b"complete").is_some());
    assert!(matches!(
        learner.prove_native_full_line(
            &interrupted,
            &interrupted.cell(0, 0).unwrap(),
            b"east",
            b"west",
            &expected,
            b"supplied-separator",
        ),
        Err(Error::Incomplete)
    ));
    assert!(matches!(
        learner.prove_native_full_line(
            &full,
            &full.cell(0, 1).unwrap(),
            b"east",
            b"west",
            &expected,
            b"supplied-separator",
        ),
        Err(Error::Incomplete)
    ));

    let unknown = admit(&grids, b"unknown", &[vec![Some(8), None, Some(8), Some(8)]]);
    assert!(matches!(
        learner.prove_native_full_line(
            &unknown,
            &unknown.cell(0, 0).unwrap(),
            b"east",
            b"west",
            &expected,
            b"supplied-separator",
        ),
        Err(Error::Incomplete)
    ));

    let foreign_grids = Grids::new();
    let foreign = admit(&foreign_grids, b"foreign", &[vec![Some(8); 4]]);
    assert!(matches!(
        learner.prove_native_full_line(
            &foreign,
            &foreign.cell(0, 0).unwrap(),
            b"east",
            b"west",
            &expected,
            b"supplied-separator",
        ),
        Err(Error::NotReady)
    ));
}
