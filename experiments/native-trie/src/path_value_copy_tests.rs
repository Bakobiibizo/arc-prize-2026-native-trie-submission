use crate::{
    Address,
    arc_grids::{Grid, Grids, Side},
    region_witness::RegionWitness,
    sparse_rewrite::{GroupProgram, SparseRewrite},
};
type GridData = Vec<Vec<Option<u8>>>;
fn grid(s: &Grids, id: &[u8], a: &GridData, side: Side) -> Grid {
    let source = s
        .source(
            id,
            b"declared path-reference/copy fixture; not learner inference",
        )
        .unwrap();
    s.admit(id, side, a, &source).unwrap()
}
// Declared fixture: slanted outline, teacher shifts non-bottom contour right,
// retaining bottom and right endpoint. This is NOT an inference routine.
fn shapes(
    height: usize,
    width: usize,
    items: &[(u8, usize, usize, usize, usize)],
) -> (GridData, GridData) {
    let mut a = vec![vec![Some(7); width]; height];
    let mut b = a.clone();
    for &(v, y, x, h, w) in items {
        let right = x + w + h - 3;
        for dx in 0..w {
            a[y][x + dx] = Some(v);
            b[y][x + dx + 1] = Some(v);
        }
        for dy in 1..h - 1 {
            a[y + dy][x + dy - 1] = Some(v);
            a[y + dy][x + w - 1 + dy] = Some(v);
            b[y + dy][x + dy] = Some(v);
            b[y + dy][(x + w + dy).min(right)] = Some(v);
        }
        for xx in x + h - 2..=right {
            a[y + h - 1][xx] = Some(v);
            b[y + h - 1][xx] = Some(v);
        }
    }
    (a, b)
}
fn teach(s: &Grids, m: &mut SparseRewrite, bad: bool) -> (Option<GroupProgram>, Vec<Address>) {
    let mut lessons = Vec::new();
    let mut source_values = Vec::new();
    for (i, (v, h, w)) in [(9, 3, 3), (6, 4, 4)].into_iter().enumerate() {
        let (a, mut b) = shapes(8, 11, &[(v, 1, 1, h, w)]);
        if bad && i == 1 {
            b[1][w + 1] = Some(7);
        }
        let input = grid(s, format!("t{i}").as_bytes(), &a, Side::TrainingInput);
        if i == 0 {
            source_values.push(
                input
                    .cell(0, 0)
                    .unwrap()
                    .child(b"value")
                    .unwrap()
                    .follow()
                    .unwrap(),
            );
        }
        source_values.push(
            input
                .cell(1, 1)
                .unwrap()
                .child(b"value")
                .unwrap()
                .follow()
                .unwrap(),
        );
        let output = grid(s, format!("o{i}").as_bytes(), &b, Side::TrainingOutput);
        lessons.push(m.derive_groups(&i.to_be_bytes(), &input, &output).unwrap());
    }
    (m.agree_path_value_actions(&lessons).unwrap(), source_values)
}
fn rules(p: &GroupProgram) -> Vec<Address> {
    (0..p.groups())
        .map(|i| {
            p.root
                .child(b"groups")
                .unwrap()
                .child(&i.to_be_bytes())
                .unwrap()
                .follow()
                .unwrap()
        })
        .collect()
}
#[test]
fn learned_value_paths_copy_new_palettes_shapes_and_immutable_outputs() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (p, values) = teach(&s, &mut m, false);
    let p = p.unwrap();
    assert!(p.root.child(b"path-copy-program").is_some());
    assert_eq!(p.groups(), 2);
    let body = rules(&p);
    let variable = body
        .iter()
        .find(|r| r.child(b"positive-source-bindings").is_some())
        .unwrap();
    for colour in &values[1..] {
        assert!(
            variable
                .child(b"positive-source-bindings")
                .unwrap()
                .child(&colour.identity_key())
                .unwrap()
                .follow()
                .unwrap()
                .same(colour)
        );
    }
    assert!(
        variable
            .child(b"excluded-source-colours")
            .unwrap()
            .child(&values[0].identity_key())
            .unwrap()
            .follow()
            .unwrap()
            .same(&values[0])
    );
    let (a, b) = shapes(12, 14, &[(4, 1, 1, 4, 4), (8, 7, 7, 3, 3)]);
    let q = grid(&s, b"q", &a, Side::QueryInput);
    let old = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_eq!(old.inspect().unwrap(), b);
    assert!(m.membership_proofs > 0 && m.membership_copies > 0);
    let mut unknown = a.clone();
    unknown[11][13] = None;
    let q = grid(&s, b"unknown", &unknown, Side::QueryInput);
    let unresolved = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_eq!(unresolved.inspect().unwrap(), unknown);
    let last = variable
        .child(b"step-word")
        .unwrap()
        .child(&1usize.to_be_bytes())
        .unwrap();
    let east = last.follow().unwrap();
    let west = m
        .root
        .child(b"path-copy-port-keys")
        .unwrap()
        .child(b"west")
        .unwrap();
    last.point_to(&west);
    let step_query = grid(&s, b"step-change", &a, Side::QueryInput);
    let altered = m.apply_groups(&mut s, &step_query, &p).unwrap();
    assert_ne!(altered.inspect().unwrap(), b);
    last.point_to(&east);
    let restored = m.apply_groups(&mut s, &step_query, &p).unwrap();
    assert_eq!(restored.inspect().unwrap(), b);
    let absent = m
        .root
        .child(b"native-path-memberships")
        .unwrap()
        .child(b"set-membership-tags")
        .unwrap()
        .child(b"absent")
        .unwrap();
    for r in &body {
        r.child(b"expected-membership").unwrap().point_to(&absent);
    }
    let q = grid(&s, b"mutated", &a, Side::QueryInput);
    let new = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_ne!(new.inspect().unwrap(), b);
    let present = m
        .root
        .child(b"native-path-memberships")
        .unwrap()
        .child(b"set-membership-tags")
        .unwrap()
        .child(b"present")
        .unwrap();
    for r in &body {
        r.child(b"expected-membership").unwrap().point_to(&present);
    }
    let restored = m.apply_groups(&mut s, &step_query, &p).unwrap();
    assert_eq!(restored.inspect().unwrap(), b);
    variable.provision(b"expected-action-source").terminate();
    let role_changed = m.apply_groups(&mut s, &step_query, &p).unwrap();
    assert_ne!(role_changed.inspect().unwrap(), b);
    assert_eq!(old.inspect().unwrap(), b);
    assert!(m.workspace_bytes() < 286_720_000);
    assert_eq!(RegionWitness::dangling_links(&m.root), 0);
}
#[test]
fn mandatory_unchanged_counterexample_rejects_path_family() {
    let s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let (p, _) = teach(&s, &mut m, true);
    assert!(p.is_none_or(|p| p.root.child(b"path-copy-program").is_none()));
    assert!(m.root.child(b"path-copy-assemblies").is_some());
}
#[test]
fn learned_different_action_port_copies_actual_query_input() {
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut lessons = Vec::new();
    for (i, v) in [9, 6].into_iter().enumerate() {
        let mut a = vec![vec![Some(7); 7]; 5];
        a[1][3] = Some(v);
        let mut b = a.clone();
        b[1..5].clone_from_slice(&a[..4]);
        let input = grid(&s, format!("t{i}").as_bytes(), &a, Side::TrainingInput);
        let output = grid(&s, format!("o{i}").as_bytes(), &b, Side::TrainingOutput);
        lessons.push(m.derive_groups(&i.to_be_bytes(), &input, &output).unwrap());
    }
    let p = m.agree_path_value_actions(&lessons).unwrap().unwrap();
    assert!(p.root.child(b"path-copy-program").is_some());
    assert_eq!(p.groups(), 1);
    let mut a = vec![vec![Some(7); 9]; 7];
    a[2][4] = Some(4);
    a[2][5] = Some(8);
    let mut b = a.clone();
    b[1..7].clone_from_slice(&a[..6]);
    let q = grid(&s, b"new-palette", &a, Side::QueryInput);
    let output = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_eq!(output.inspect().unwrap(), b);
    let rule = rules(&p).remove(0);
    assert_eq!(
        rule.child(b"action-port")
            .unwrap()
            .follow()
            .unwrap()
            .token(),
        Some(b"north".as_slice())
    );
    let west = m
        .root
        .child(b"path-copy-port-keys")
        .unwrap()
        .child(b"west")
        .unwrap();
    rule.child(b"action-port").unwrap().point_to(&west);
    let changed = m.apply_groups(&mut s, &q, &p).unwrap();
    assert_ne!(changed.inspect().unwrap(), b);
    assert_eq!(output.inspect().unwrap(), b);
    assert_eq!(RegionWitness::dangling_links(&m.root), 0);
    drop(changed);
    drop(rule);
    drop(lessons);
    drop(p);
    drop(m);
    drop(q);
    drop(s);
    assert_eq!(output.inspect().unwrap(), b);
    assert_eq!(RegionWitness::dangling_links(&output.anchor), 0);
}
