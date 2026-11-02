use native_trie_seed::{
    Fault,
    arc_grids::{Grid, GridError, Grids, Side},
    arc_rows::{RowError, RowProgram},
    source_field,
};
fn observe(space: &Grids, id: &[u8], cells: Vec<Vec<Option<u8>>>) -> Grid {
    let source = space
        .source(
            id,
            b"test fixture source; values explicitly supplied by the test",
        )
        .unwrap();
    space.admit(id, Side::QueryInput, &cells, &source).unwrap()
}
fn lesson(space: &Grids, scope: &[u8], slots: &[usize]) -> RowProgram {
    let source = space
        .source(
            scope,
            b"explicit symbolic row-reference lesson; not puzzle evidence",
        )
        .unwrap();
    RowProgram::teach(space, scope, slots, &source).unwrap()
}
#[test]
fn typed_values_original_sources_and_outside_are_distinct() {
    let space = Grids::new();
    let grid = observe(
        &space,
        b"input",
        vec![vec![Some(0), None], vec![Some(7), Some(0)]],
    );
    assert_eq!(
        grid.inspect().unwrap(),
        vec![vec![Some(0), None], vec![Some(7), Some(0)]]
    );
    let zero = grid
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    assert!(!zero.same(&space.outside()));
    assert!(!zero.same(&space.unresolved()));
    assert!(!space.outside().same(&space.unresolved()));
    assert!(
        grid.cell(0, 1)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap()
            .same(&space.unresolved())
    );
    assert!(
        grid.cell(0, 0)
            .unwrap()
            .child(b"north")
            .unwrap()
            .follow()
            .unwrap()
            .same(&space.outside())
    );
    assert!(
        grid.cell(0, 0)
            .unwrap()
            .child(b"south")
            .unwrap()
            .follow()
            .unwrap()
            .same(&grid.cell(1, 0).unwrap())
    );
    assert!(
        grid.observed_source()
            .unwrap()
            .unwrap()
            .child(b"raw-bytes")
            .unwrap()
            .child(b"test fixture source; values explicitly supplied by the test")
            .is_some()
    );
}
#[test]
fn swap_and_repeat_use_exactly_the_taught_copy_count_at_unseen_widths() {
    let mut space = Grids::new();
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    let mut repeat = lesson(&space, b"repeat", &[0, 1, 0, 1]);
    for width in [1, 4, 17, 30] {
        let a = vec![Some(8); width];
        let b = vec![Some(6); width];
        let input = observe(&space, &width.to_be_bytes(), vec![a.clone(), b.clone()]);
        let output = swap.apply(&mut space, &input, 2).unwrap();
        assert_eq!(output.cycles, 2);
        assert_eq!(output.grid.inspect().unwrap(), vec![b.clone(), a.clone()]);
        let composed = repeat.apply(&mut space, &output.grid, 4).unwrap();
        assert_eq!(composed.cycles, 4);
        assert_eq!(
            composed.grid.inspect().unwrap(),
            vec![b.clone(), a.clone(), b, a]
        );
        assert_eq!(
            input.inspect().unwrap(),
            vec![vec![Some(8); width], vec![Some(6); width]]
        );
    }
}
#[test]
fn output_is_readdressed_with_fresh_topology_not_fabricated_source() {
    let mut space = Grids::new();
    let input = observe(
        &space,
        b"input",
        vec![vec![Some(1), Some(2)], vec![Some(3), Some(4)]],
    );
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    let output = swap.apply(&mut space, &input, 2).unwrap().grid;
    assert!(output.observed_source().unwrap().is_none());
    assert!(
        output
            .anchor
            .child(b"input")
            .unwrap()
            .follow()
            .unwrap()
            .same(&input.anchor)
    );
    assert!(!output.cell(0, 0).unwrap().same(&input.cell(1, 0).unwrap()));
    assert!(
        output
            .cell(0, 0)
            .unwrap()
            .child(b"derived-from")
            .unwrap()
            .follow()
            .unwrap()
            .same(&input.cell(1, 0).unwrap())
    );
    assert!(
        output
            .cell(0, 0)
            .unwrap()
            .child(b"south")
            .unwrap()
            .follow()
            .unwrap()
            .same(&output.cell(1, 0).unwrap())
    );
    assert!(
        output
            .cell(0, 0)
            .unwrap()
            .child(b"north")
            .unwrap()
            .follow()
            .unwrap()
            .same(&space.outside())
    );
    let roundtrip = swap.apply(&mut space, &output, 2).unwrap().grid;
    assert_eq!(roundtrip.inspect().unwrap(), input.inspect().unwrap());
}
#[test]
fn exhausted_budget_does_not_publish_or_reuse_a_stale_output() {
    let mut space = Grids::new();
    let input = observe(&space, b"a", vec![vec![Some(1)], vec![Some(2)]]);
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    let first = swap.apply(&mut space, &input, 2).unwrap().grid;
    let second = observe(&space, b"b", vec![vec![Some(8)], vec![Some(9)]]);
    assert!(matches!(
        swap.apply(&mut space, &second, 1),
        Err(RowError::Limit)
    ));
    assert_eq!(first.inspect().unwrap(), vec![vec![Some(2)], vec![Some(1)]]);
    assert!(matches!(
        swap.apply(&mut space, &second, 0),
        Err(RowError::Limit)
    ));
    let next = swap.apply(&mut space, &second, 2).unwrap().grid;
    assert_eq!(next.inspect().unwrap(), vec![vec![Some(9)], vec![Some(8)]]);
}
#[test]
fn actual_stored_instruction_mutation_controls_output_no_host_swap() {
    let mut space = Grids::new();
    let input = observe(&space, b"input", vec![vec![Some(1)], vec![Some(2)]]);
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    let first = swap
        .construction
        .child(b"instructions")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap();
    let binding0 = swap
        .construction
        .child(b"bindings")
        .unwrap()
        .child(b"0")
        .unwrap();
    source_field(&first).unwrap().point_to(&binding0);
    let output = swap.apply(&mut space, &input, 2).unwrap().grid;
    assert_eq!(
        output.inspect().unwrap(),
        vec![vec![Some(1)], vec![Some(1)]]
    );
}
#[test]
fn missing_instruction_data_faults_without_fallback() {
    let mut space = Grids::new();
    let input = observe(&space, b"input", vec![vec![Some(1)], vec![Some(2)]]);
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    let first = swap
        .construction
        .child(b"instructions")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap();
    source_field(&first).unwrap().point_to(&space.outside());
    assert!(matches!(
        swap.apply(&mut space, &input, 2),
        Err(RowError::Machine(Fault::Unset))
    ));
}
#[test]
fn invalid_inputs_duplicates_and_wrong_arity_are_explicit() {
    let mut space = Grids::new();
    let source = space.source(b"source", b"raw").unwrap();
    for invalid in [
        vec![],
        vec![vec![]],
        vec![vec![Some(1)], vec![Some(2), Some(3)]],
        vec![vec![Some(0); 31]],
    ] {
        assert!(matches!(
            space.admit(b"invalid", Side::TrainingInput, &invalid, &source),
            Err(GridError::InvalidShape)
        ));
    }
    assert!(matches!(
        space.admit(b"invalid", Side::TrainingInput, &[vec![Some(10)]], &source),
        Err(GridError::InvalidColor)
    ));
    let grid = space
        .admit(b"invalid", Side::TrainingInput, &[vec![Some(1)]], &source)
        .unwrap();
    assert!(matches!(
        space.admit(b"invalid", Side::TrainingInput, &[vec![Some(2)]], &source),
        Err(GridError::DuplicateOccurrence)
    ));
    let output = space
        .admit(b"invalid", Side::TrainingOutput, &[vec![Some(9)]], &source)
        .unwrap();
    assert_eq!(output.inspect().unwrap(), vec![vec![Some(9)]]);
    assert_eq!(grid.inspect().unwrap(), vec![vec![Some(1)]]);
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    assert!(matches!(
        swap.apply(&mut space, &grid, 2),
        Err(RowError::Grid(GridError::InvalidShape))
    ));
}
#[test]
fn unfinished_row_is_not_published_even_when_continuation_halts() {
    let mut space = Grids::new();
    let input = observe(&space, b"input", vec![vec![Some(1)], vec![Some(2)]]);
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    let first = swap
        .construction
        .child(b"instructions")
        .unwrap()
        .child(&0usize.to_be_bytes())
        .unwrap();
    first
        .child(b"continuation")
        .unwrap()
        .point_to(&swap.construction.child(b"halt").unwrap());
    assert!(matches!(
        swap.apply(&mut space, &input, 2),
        Err(RowError::Grid(GridError::Incomplete))
    ));
}
#[test]
fn unknown_is_preserved_and_unrelated_curricula_do_not_change_execution() {
    let mut space = Grids::new();
    let input = observe(
        &space,
        b"input",
        vec![vec![None, Some(0)], vec![Some(9), None]],
    );
    let mut swap = lesson(&space, b"swap", &[1, 0]);
    let before = swap.apply(&mut space, &input, 2).unwrap();
    for i in 0..100usize {
        let mut scope = b"unrelated".to_vec();
        scope.extend(i.to_be_bytes());
        let _ = lesson(&space, &scope, &[0, 0]);
    }
    let after = swap.apply(&mut space, &input, 2).unwrap();
    assert_eq!(before.cycles, after.cycles);
    assert_eq!(
        before.grid.inspect().unwrap(),
        after.grid.inspect().unwrap()
    );
    assert_eq!(
        after.grid.inspect().unwrap(),
        vec![vec![Some(9), None], vec![None, Some(0)]]
    );
}
