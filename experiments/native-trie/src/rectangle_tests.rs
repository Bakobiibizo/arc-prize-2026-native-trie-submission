use crate::{
    arc_grids::{Grid, Grids, Side},
    next_field,
    region_witness::RegionWitness,
    source_field,
    sparse_rewrite::{RectangleProjection, SparseRewrite},
    target_field,
};
use std::rc::Rc;
type Pixels = Vec<Vec<Option<u8>>>;
fn grid(s: &Grids, id: &[u8], a: &Pixels) -> Grid {
    let raw = s
        .source(id, b"declared rectangle-copy prototype fixture")
        .unwrap();
    s.admit(id, Side::QueryInput, a, &raw).unwrap()
}
fn fixture(s: &Grids, m: &mut SparseRewrite) -> (Grid, Vec<Rc<RectangleProjection>>, Pixels) {
    let mut a = vec![vec![Some(7); 9]; 8];
    for row in a.iter_mut().take(3).skip(1) {
        for c in row.iter_mut().take(5).skip(2) {
            *c = Some(5);
        }
    }
    for row in a.iter_mut().take(7).skip(5) {
        for c in row.iter_mut().take(7).skip(5) {
            *c = Some(4);
        }
    }
    let input = grid(s, b"input", &a);
    let p = Rc::new(
        m.rectangle_source(&input, &input.cell(1, 2).unwrap())
            .unwrap(),
    );
    let q = Rc::new(
        m.rectangle_source(&input, &input.cell(5, 5).unwrap())
            .unwrap(),
    );
    assert_eq!(p.status.token(), Some(b"rectangle".as_slice()));
    assert_eq!(q.status.token(), Some(b"rectangle".as_slice()));
    let mut want = vec![vec![Some(7); 9]; 8];
    for row in want.iter_mut().take(2) {
        for c in row.iter_mut().take(3) {
            *c = Some(5);
        }
    }
    for row in want.iter_mut().take(3).skip(1) {
        for c in row.iter_mut().take(4).skip(2) {
            *c = Some(4);
        }
    }
    (input, vec![p, q], want)
}
#[test]
fn native_ordered_overlap_copies_original_slots_and_survives_callers() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (input, sources, want) = fixture(&s, &mut m);
    let bg = input
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let program = m
        .rectangle_copy_program(&sources, &bg, b"NW", b"SE")
        .unwrap();
    let mut w = m
        .prepare_rectangle_copy(&input, &program, sources.clone())
        .unwrap();
    let tape = w.root.clone();
    assert_eq!(w.execute(w.budget).unwrap(), 82);
    let output = w.record(&mut s).unwrap();
    assert_eq!(output.inspect().unwrap(), want);
    let event = tape
        .child(b"observed-steps")
        .unwrap()
        .child(&78u64.to_be_bytes())
        .unwrap();
    assert_eq!(
        event
            .child(b"target-value-before")
            .unwrap()
            .follow()
            .unwrap()
            .token(),
        Some(&[5][..])
    );
    assert_eq!(
        event
            .child(b"target-value-after")
            .unwrap()
            .follow()
            .unwrap()
            .token(),
        Some(&[4][..])
    );
    let nw = m
        .root
        .child(b"rectangle-corner-keys")
        .unwrap()
        .child(b"NW")
        .unwrap();
    program.child(b"next-corner").unwrap().point_to(&nw);
    let mut changed = m
        .prepare_rectangle_copy(&input, &program, sources.clone())
        .unwrap();
    changed.execute(changed.budget).unwrap();
    let changed = changed.record(&mut s).unwrap();
    assert_ne!(changed.inspect().unwrap(), want);
    assert_eq!(output.inspect().unwrap(), want);
    assert!(m.workspace_bytes() < 286720000);
    drop(changed);
    drop(sources);
    drop(program);
    drop(input);
    drop(m);
    drop(s);
    assert_eq!(output.inspect().unwrap(), want);
    assert_eq!(RegionWitness::dangling_links(&output.anchor), 0);
}
#[test]
fn native_source_target_and_continuation_instructions_are_authoritative() {
    for case in 0..3 {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let (input, sources, want) = fixture(&s, &mut m);
        let bg = input
            .cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap();
        let p = m
            .rectangle_copy_program(&sources, &bg, b"NW", b"SE")
            .unwrap();
        let mut w = m.prepare_rectangle_copy(&input, &p, sources).unwrap();
        let root = w.root.clone();
        let state = root
            .child(b"execution")
            .unwrap()
            .child(b"construction")
            .unwrap()
            .child(&72usize.to_be_bytes())
            .unwrap();
        match case {
            0 => source_field(&state)
                .unwrap()
                .point_to(&input.cell(5, 5).unwrap().child(b"value").unwrap()),
            1 => {
                let target = root
                    .child(b"rows")
                    .unwrap()
                    .child(&0usize.to_be_bytes())
                    .unwrap()
                    .child(b"cells")
                    .unwrap()
                    .child(&5usize.to_be_bytes())
                    .unwrap()
                    .child(b"value")
                    .unwrap();
                target_field(&state).unwrap().point_to(&target);
            }
            _ => {
                let arg = root.provision(b"early-halt");
                arg.point_to(&root.child(b"execution").unwrap().child(b"halt").unwrap());
                next_field(&state).unwrap().point_to(&arg);
            }
        }
        if case == 2 {
            assert!(w.execute(w.budget).is_err());
            assert!(w.record(&mut s).is_err());
            assert!(root.child(b"committed").is_none());
        } else {
            w.execute(w.budget).unwrap();
            let q = w.record(&mut s).unwrap();
            assert_ne!(q.inspect().unwrap(), want);
            if case == 0 {
                assert_eq!(q.inspect().unwrap()[0][0], Some(4));
            }
        }
    }
}
#[test]
fn holes_l_shapes_and_unknown_frontiers_do_not_produce_rectangle_certificates() {
    for case in 0..3 {
        let s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let mut a = vec![vec![Some(7); 8]; 8];
        for row in a.iter_mut().take(5).skip(1) {
            for c in row.iter_mut().take(5).skip(1) {
                *c = Some(5);
            }
        }
        match case {
            0 => a[2][2] = Some(7),
            1 => {
                for row in a.iter_mut().take(4).skip(2) {
                    for c in row.iter_mut().take(5).skip(2) {
                        *c = Some(7);
                    }
                }
            }
            _ => a[0][1] = None,
        };
        let q = grid(&s, b"shape", &a);
        let p = m.rectangle_source(&q, &q.cell(1, 1).unwrap()).unwrap();
        assert_eq!(
            p.status.token(),
            Some(if case == 2 {
                b"unavailable".as_slice()
            } else {
                b"not-rectangle"
            })
        );
    }
}
#[test]
fn stale_native_coverage_foreign_owner_outside_and_partial_are_preserved_failures() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (input, sources, _) = fixture(&s, &mut m);
    let bg = input
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let mut foreign = SparseRewrite::new(&s);
    assert!(
        foreign
            .rectangle_copy_program(&sources, &bg, b"NW", b"SE")
            .is_err()
    );
    let out = m
        .rectangle_copy_program(&sources, &bg, b"SE", b"SE")
        .unwrap();
    assert!(
        m.prepare_rectangle_copy(&input, &out, sources.clone())
            .is_err()
    );
    let p = m
        .rectangle_copy_program(&sources, &bg, b"NW", b"SE")
        .unwrap();
    let mut w = m
        .prepare_rectangle_copy(&input, &p, sources.clone())
        .unwrap();
    let root = w.root.clone();
    assert!(w.execute(1).is_err());
    assert!(w.record(&mut s).is_err());
    assert!(root.child(b"committed").is_none());
    let original = sources[0]
        .proof
        .child(b"original-count-proof")
        .unwrap()
        .follow()
        .unwrap();
    let zero = m
        .root
        .child(b"native-components")
        .unwrap()
        .child(b"cardinalities")
        .unwrap()
        .child(b"zero")
        .unwrap();
    original.child(b"value").unwrap().point_to(&zero);
    assert!(m.prepare_rectangle_copy(&input, &p, sources).is_err());
}
#[test]
fn actual_native_boundary_mutation_and_copy_fault_remain_authoritative() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (input, sources, _) = fixture(&s, &mut m);
    let bg = input
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let program = m
        .rectangle_copy_program(&sources, &bg, b"NW", b"SE")
        .unwrap();
    let mut w = m
        .prepare_rectangle_copy(&input, &program, sources.clone())
        .unwrap();
    let root = w.root.clone();
    let state = root
        .child(b"execution")
        .unwrap()
        .child(b"construction")
        .unwrap()
        .child(&72usize.to_be_bytes())
        .unwrap();
    source_field(&state)
        .unwrap()
        .point_to(&root.provision(b"unset-source"));
    assert!(w.execute(w.budget).is_err());
    assert!(w.record(&mut s).is_err());
    assert!(root.child(b"committed").is_none());
    let region = sources[0]
        .proof
        .child(b"region-proof")
        .unwrap()
        .follow()
        .unwrap();
    let neighbour = input.cell(0, 2).unwrap();
    let view = RegionWitness::recorded_view(&region, &neighbour).unwrap();
    let unknown = m
        .root
        .child(b"native-components")
        .unwrap()
        .child(b"class-tags")
        .unwrap()
        .child(b"unknown")
        .unwrap();
    view.child(b"class").unwrap().point_to(&unknown);
    let changed = m
        .rectangle_source(&input, &input.cell(1, 2).unwrap())
        .unwrap();
    assert_eq!(changed.status.token(), Some(b"unavailable".as_slice()));
}
#[test]
fn changed_certified_row_bindings_and_input_values_cannot_reuse_frozen_geometry() {
    for after_prepare in [false, true] {
        let mut s = Grids::new();
        let mut m = SparseRewrite::new(&s);
        let (input, sources, _) = fixture(&s, &mut m);
        let bg = input
            .cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap();
        let program = m
            .rectangle_copy_program(&sources, &bg, b"NW", b"SE")
            .unwrap();
        if after_prepare {
            let mut w = m.prepare_rectangle_copy(&input, &program, sources).unwrap();
            let root = w.root.clone();
            let other = input
                .cell(5, 5)
                .unwrap()
                .child(b"value")
                .unwrap()
                .follow()
                .unwrap();
            input
                .cell(1, 2)
                .unwrap()
                .child(b"value")
                .unwrap()
                .point_to(&other);
            assert!(w.execute(w.budget).is_err());
            assert!(w.record(&mut s).is_err());
            assert!(root.child(b"committed").is_none());
        } else {
            let descriptor = sources[0]
                .proof
                .child(b"source-rows")
                .unwrap()
                .child(&0usize.to_be_bytes())
                .unwrap()
                .child(b"cells")
                .unwrap()
                .child(&0usize.to_be_bytes())
                .unwrap();
            let other = input.cell(5, 5).unwrap();
            descriptor.child(b"source-cell").unwrap().point_to(&other);
            descriptor
                .child(b"source-value-slot")
                .unwrap()
                .point_to(&other.child(b"value").unwrap());
            assert!(m.prepare_rectangle_copy(&input, &program, sources).is_err());
        }
    }
}
#[test]
fn maximum900_source_footprint_and_native_copy_fit_unchanged_envelope() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let a = vec![vec![Some(5); 30]; 30];
    let input = grid(&s, b"maximum", &a);
    let p = Rc::new(
        m.rectangle_source(&input, &input.cell(0, 0).unwrap())
            .unwrap(),
    );
    assert_eq!(p.status.token(), Some(b"rectangle".as_slice()));
    let background = input
        .cell(0, 0)
        .unwrap()
        .child(b"value")
        .unwrap()
        .follow()
        .unwrap();
    let sources = vec![p];
    let program = m
        .rectangle_copy_program(&sources, &background, b"NW", b"SE")
        .unwrap();
    let mut w = m.prepare_rectangle_copy(&input, &program, sources).unwrap();
    assert_eq!(w.execute(w.budget).unwrap(), 1800);
    assert_eq!(w.record(&mut s).unwrap().inspect().unwrap(), a);
    assert!(m.workspace_bytes() < 286720000);
}
