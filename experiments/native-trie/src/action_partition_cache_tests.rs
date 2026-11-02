use super::*;
use crate::arc_grids::Side;
#[test]
fn knownness_fact_is_native_and_fingerprinted_by_producer_value_and_sentinel() {
    let s = Grids::new();
    let source = s
        .source(b"known", b"knownness cache fingerprint fixture")
        .unwrap();
    let input = s
        .admit(b"i", Side::TrainingInput, &[vec![Some(0)]], &source)
        .unwrap();
    let mut m = SparseRewrite::new(&s);
    let (producer, value) = m
        .component_value(&input, &input.cell(0, 0).unwrap())
        .unwrap();
    let absent = m
        .root
        .child(b"native-components")
        .unwrap()
        .child(b"relation-tags")
        .unwrap()
        .child(b"unresolved")
        .unwrap();
    let before = m.comparison_copies;
    let (tag, fact) = m.property_knownness(&producer, &value, &absent).unwrap();
    assert!(tag.same(&m.yes));
    assert_eq!(m.comparison_copies, before + 6);
    let (again, same) = m.property_knownness(&producer, &value, &absent).unwrap();
    assert!(again.same(&tag) && same.same(&fact));
    assert_eq!(m.comparison_copies, before + 6);
    let (changed, different) = m.property_knownness(&producer, &value, &value).unwrap();
    assert!(changed.same(&m.no) && !different.same(&fact));
    assert_eq!(m.comparison_copies, before + 12);
    producer.child(b"committed").unwrap().point_to(&m.no);
    assert!(m.property_knownness(&producer, &value, &absent).is_err());
}
#[test]
fn cached_native_projection_checks_input_and_count_producer_identity() {
    let s = Grids::new();
    let source = s
        .source(
            b"cache",
            b"two independent identical native input occurrences",
        )
        .unwrap();
    let pixels = vec![vec![Some(0); 2]; 2];
    let a = s
        .admit(b"a", Side::TrainingInput, &pixels, &source)
        .unwrap();
    let b = s
        .admit(b"b", Side::TrainingInput, &pixels, &source)
        .unwrap();
    let mut m = SparseRewrite::new(&s);
    let (region, _) = m.component_value(&a, &a.cell(0, 0).unwrap()).unwrap();
    let owner = m.root.child(b"native-components").unwrap();
    assert!(RegionWitness::projection_from_committed(&owner, &b, &region).is_err());
    let live = RegionWitness::projection_from_committed(&owner, &a, &region).unwrap();
    assert!(live.proof.same(&region));
    assert!(live.copies > 0);
    let (pa, va) = m.complete_count_value(&a, &a.cell(0, 0).unwrap()).unwrap();
    let copies = m.count_copies;
    let (again, vagain) = m.complete_count_value(&a, &a.cell(1, 1).unwrap()).unwrap();
    assert!(pa.same(&again) && va.same(&vagain));
    assert_eq!(m.count_copies, copies);
    let (pb, _) = m.complete_count_value(&b, &b.cell(0, 0).unwrap()).unwrap();
    m.root
        .child(b"complete-count-cache")
        .unwrap()
        .child(&region.identity_key())
        .unwrap()
        .point_to(&pb);
    assert!(m.complete_count_value(&a, &a.cell(0, 0).unwrap()).is_err());
}
