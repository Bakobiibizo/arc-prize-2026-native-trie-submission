use native_trie_seed::{
    arc_grids::{GridError, Grids, Side},
    arc_rows::{RowError, RowProgram},
};
#[test]
fn foreign_sources_and_grids_cannot_leave_dangling_lineage() {
    let mut a = Grids::new();
    let mut b = Grids::new();
    let sa = a.source(b"source", b"a").unwrap();
    let sb = b.source(b"source", b"b").unwrap();
    let cells = vec![vec![Some(1)], vec![Some(2)]];
    assert!(matches!(
        a.admit(b"input", Side::QueryInput, &cells, &sb),
        Err(GridError::ForeignSpace)
    ));
    assert!(matches!(
        RowProgram::teach(&a, b"swap", &[1, 0], &sb),
        Err(RowError::Grid(GridError::ForeignSpace))
    ));
    let ga = a.admit(b"input", Side::QueryInput, &cells, &sa).unwrap();
    let gb = b.admit(b"input", Side::QueryInput, &cells, &sb).unwrap();
    let mut program = RowProgram::teach(&a, b"swap", &[1, 0], &sa).unwrap();
    assert!(matches!(
        program.bind(&gb),
        Err(RowError::Grid(GridError::ForeignSpace))
    ));
    program.bind(&ga).unwrap();
    program.execute(2).unwrap();
    assert!(matches!(
        program.record(&mut b),
        Err(RowError::Grid(GridError::ForeignSpace))
    ));
    assert!(program.record(&mut a).is_ok());
}
#[test]
fn lifecycle_requires_bound_complete_unconsumed_result_and_valid_lesson() {
    let mut space = Grids::new();
    let source = space.source(b"source", b"fixture").unwrap();
    for slots in [vec![], vec![2], vec![0; 31]] {
        assert!(matches!(
            RowProgram::teach(&space, b"swap", &slots, &source),
            Err(RowError::InvalidLesson)
        ));
    }
    let mut program = RowProgram::teach(&space, b"swap", &[1, 0], &source).unwrap();
    assert!(matches!(
        RowProgram::teach(&space, b"swap", &[0, 1], &source),
        Err(RowError::DuplicateScope)
    ));
    assert!(matches!(program.execute(2), Err(RowError::NotReady)));
    assert!(matches!(
        program.record(&mut space),
        Err(RowError::NotReady)
    ));
    let input = space
        .admit(
            b"input",
            Side::QueryInput,
            &[vec![Some(1)], vec![Some(2)]],
            &source,
        )
        .unwrap();
    program.bind(&input).unwrap();
    assert!(matches!(program.execute(1), Err(RowError::Limit)));
    assert!(matches!(
        program.record(&mut space),
        Err(RowError::NotReady)
    ));
    assert_eq!(program.execute(1).unwrap(), 2);
    let output = program.record(&mut space).unwrap().grid;
    assert!(matches!(
        program.record(&mut space),
        Err(RowError::NotReady)
    ));
    assert!(matches!(program.execute(1), Err(RowError::NotReady)));
    drop(program);
    drop(space);
    assert_eq!(
        output.inspect().unwrap(),
        vec![vec![Some(2)], vec![Some(1)]]
    );
    assert!(
        output
            .anchor
            .child(b"construction")
            .unwrap()
            .follow()
            .is_ok()
    );
}
