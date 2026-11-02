use crate::{
    arc_grids::{Grids, Side},
    region_witness::RegionWitness,
    sparse_rewrite::SparseRewrite,
};
#[test]
fn ignores_uncommitted_cache_and_retains_closed_native_facts() {
    let s = Grids::new();
    let src = s
        .source(
            b"cache-control",
            b"enclosed single zero surrounded by ones; uncommitted cache is not truth",
        )
        .unwrap();
    let a = vec![
        vec![Some(1), Some(1), Some(1)],
        vec![Some(1), Some(0), Some(1)],
        vec![Some(1), Some(1), Some(1)],
    ];
    let mut b = a.clone();
    b[1][1] = Some(4);
    let i = s.admit(b"i", Side::TrainingInput, &a, &src).unwrap();
    let o = s.admit(b"o", Side::TrainingOutput, &b, &src).unwrap();
    let mut m = SparseRewrite::new(&s);
    let owner = m.root.provision(b"native-components");
    let fake = owner.provision(b"uncommitted-control");
    let escape = owner.provision(b"relation-tags").provision(b"escape");
    fake.provision(b"result").point_to(&escape);
    let cache = owner
        .provision(b"cache")
        .provision(&i.anchor.identity_key());
    cache
        .provision(&i.cell(1, 1).unwrap().identity_key())
        .point_to(&fake);
    let d = m.derive_component_relations(b"d", &i, &o).unwrap();
    let rule = d.rule.unwrap();
    let bounded = owner
        .child(b"relation-tags")
        .unwrap()
        .child(b"bounded")
        .unwrap();
    assert!(
        rule.root
            .child(b"expected-kind")
            .unwrap()
            .follow()
            .unwrap()
            .same(&bounded)
    );
    let (proof, value) = RegionWitness::cached(&cache, &i.cell(1, 1).unwrap())
        .unwrap()
        .unwrap();
    assert!(!proof.same(&fake));
    assert!(value.same(&bounded));
    assert!(proof.child(b"activation-retired").unwrap().is_terminal());
    assert!(RegionWitness::recorded_view(&proof, &i.cell(1, 1).unwrap()).is_some());
    assert_eq!(RegionWitness::dangling_links(&m.root), 0);
}
