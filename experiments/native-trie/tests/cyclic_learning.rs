use native_trie_seed::{
    arc_grids::{Grid, Grids, Side},
    sparse_rewrite::{GroupProgram, SparseRewrite},
};
type Pixels = Vec<Vec<Option<u8>>>;
fn grid(s: &Grids, id: &str, p: &Pixels, side: Side) -> Grid {
    let src = s
        .source(id.as_bytes(), b"all-demo cyclic grammar control fixture")
        .unwrap();
    s.admit(id.as_bytes(), side, p, &src).unwrap()
}
fn fixture(palette: &[u8], h: usize, w: usize) -> (Pixels, Pixels) {
    let mut a = vec![vec![Some(7); w]; h];
    for (i, v) in palette.iter().enumerate() {
        a[0][i] = Some(*v);
    }
    let b = (0..h)
        .map(|y| {
            (0..w)
                .map(|x| Some(palette[(x + y) % palette.len()]))
                .collect()
        })
        .collect();
    (a, b)
}
fn teach(s: &Grids, m: &mut SparseRewrite, negative: u8) -> Option<GroupProgram> {
    let mut lessons = Vec::new();
    for (i, palette) in [vec![2, 8], vec![1, 3, 6], vec![4, 9, 2, 1]]
        .iter()
        .enumerate()
    {
        let (a, mut b) = fixture(palette, 6 + i, 8 + i);
        if negative == 1 && i == 2 {
            b[1][1] = a[1][1];
        }
        if negative == 2 {
            for row in &mut b {
                row.fill(Some(palette[0]));
            }
        }
        let input = grid(s, &format!("i{i}"), &a, Side::TrainingInput);
        let output = grid(s, &format!("o{i}"), &b, Side::TrainingOutput);
        lessons.push(m.derive_groups(&i.to_be_bytes(), &input, &output).unwrap());
    }
    m.learn_cyclic_actions(&lessons).unwrap()
}
#[test]
fn learns_input_derived_cycle_sizes_two_three_four_and_new_palette() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = teach(&s, &mut m, 0).unwrap();
    assert!(p.root.child(b"cyclic-lattice-program").is_some());
    assert_eq!(p.groups(), 1);
    let (a, b) = fixture(&[5, 0, 8, 2], 9, 11);
    let input = grid(&s, "q", &a, Side::QueryInput);
    assert!(m.source_pair_applicable(&input, &p).unwrap());
    let out = m.apply_groups(&mut s, &input, &p).unwrap();
    assert_eq!(out.inspect().unwrap(), b);
    let mut unknown = a.clone();
    unknown[8][10] = None;
    let q = grid(&s, "unknown", &unknown, Side::QueryInput);
    assert!(!m.source_pair_applicable(&q, &p).unwrap());
    let bg = vec![vec![Some(7); 11]; 9];
    let q = grid(&s, "all-bg", &bg, Side::QueryInput);
    assert!(!m.source_pair_applicable(&q, &p).unwrap());
    let mut conflict = a.clone();
    conflict[1][0] = Some(5);
    let q = grid(&s, "conflict", &conflict, Side::QueryInput);
    assert!(!m.source_pair_applicable(&q, &p).unwrap());
    let foreign_s = Grids::new();
    let mut foreign = SparseRewrite::new(&foreign_s);
    assert!(foreign.source_pair_applicable(&input, &p).is_err());
    drop(input);
    drop(p);
    drop(m);
    drop(s);
    assert_eq!(out.inspect().unwrap(), b);
}
#[test]
fn all_unchanged_and_functional_but_wrong_teacher_labels_are_mandatory() {
    for bad in [1, 2] {
        let s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        assert!(
            teach(&s, &mut m, bad)
                .is_none_or(|p| p.root.child(b"cyclic-lattice-program").is_none())
        );
    }
}
#[test]
fn generalized_candidates_address_primary_replay_provenance() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut lessons = Vec::new();
    for (i, palette) in [vec![2, 8], vec![1, 3, 6]].iter().enumerate() {
        let (input_pixels, output_pixels) = fixture(palette, 6 + i, 8 + i);
        let input = grid(
            &s,
            &format!("candidate-i{i}"),
            &input_pixels,
            Side::TrainingInput,
        );
        let output = grid(
            &s,
            &format!("candidate-o{i}"),
            &output_pixels,
            Side::TrainingOutput,
        );
        lessons.push(m.derive_groups(&i.to_be_bytes(), &input, &output).unwrap());
    }
    let candidates = m.agree_generalized_candidates(&lessons).unwrap();
    assert_eq!(candidates.candidates().len(), 1);
    let first = candidates.primary().unwrap();
    assert!(
        candidates
            .root
            .child(b"selected")
            .unwrap()
            .follow()
            .unwrap()
            .same(&first.program().root)
    );
    for (index, candidate) in candidates.candidates().iter().enumerate() {
        let addressed = candidates
            .root
            .child(b"ordered-replay-valid-candidates")
            .unwrap()
            .child(&index.to_be_bytes())
            .unwrap();
        assert!(
            addressed
                .child(b"program")
                .unwrap()
                .follow()
                .unwrap()
                .same(&candidate.program().root)
        );
        assert!(
            addressed
                .child(b"all-demonstrations-replayed")
                .unwrap()
                .is_terminal()
        );
        assert!(
            addressed
                .child(b"provenance")
                .unwrap()
                .child(candidate.origin().as_str().as_bytes())
                .unwrap()
                .is_terminal()
        );
    }
    assert!(candidates.secondary().is_none());
}

#[test]
fn actual_stored_background_binding_controls_new_invocation() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let p = teach(&s, &mut m, 0).unwrap();
    let (a, b) = fixture(&[5, 0, 8], 9, 11);
    let q = grid(&s, "q", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_eq!(old.inspect().unwrap(), b);
    let fg = q
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    p.root.child(b"background-value").unwrap().point_to(&fg);
    let changed = m.apply_groups(&mut s, &q, &p);
    assert!(changed.is_err() || changed.unwrap().inspect().unwrap() != b);
    assert_eq!(old.inspect().unwrap(), b);
}
