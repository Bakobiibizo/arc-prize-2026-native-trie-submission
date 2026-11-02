use native_trie_seed::{
    arc_grids::{Grids, Side},
    sparse_rewrite::SparseRewrite,
};
#[test]
fn outside_and_unknown_neighbours_never_become_copy_sources() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut ds = Vec::new();
    for (id, colour) in [(b"a".as_slice(), 1), (b"b".as_slice(), 2)] {
        let src = s.source(id, b"boundary fixture").unwrap();
        let input = s
            .admit(
                id,
                Side::TrainingInput,
                &[vec![Some(colour), Some(0), Some(colour)]],
                &src,
            )
            .unwrap();
        let output = s
            .admit(id, Side::TrainingOutput, &[vec![Some(colour); 3]], &src)
            .unwrap();
        ds.push(m.derive(id, &input, &output).unwrap());
    }
    let r = SparseRewrite::agree(&ds).unwrap();
    for (i, raw) in [
        vec![vec![Some(0)]],
        vec![vec![None, Some(0), Some(3)]],
        vec![vec![Some(3), Some(0), None]],
    ]
    .iter()
    .enumerate()
    {
        let id = i.to_be_bytes();
        let src = s.source(&id, b"unknown query").unwrap();
        let q = s.admit(&id, Side::QueryInput, raw, &src).unwrap();
        assert_eq!(m.apply(&mut s, &q, &r).unwrap().inspect().unwrap(), *raw);
    }
}
