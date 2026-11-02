use super::*;
use crate::{
    Trie,
    arc_grids::{Grid, Grids, Side},
    region_witness::RegionWitness,
};
fn setup() -> (Address, Address, Grid, Grid, CompleteCountProjection) {
    let s = Grids::new();
    let owner = Trie::default().root();
    let native = owner.provision(b"native-components");
    let source = s
        .source(b"s", b"checked count-certificate retention fixture")
        .unwrap();
    let pixels = [vec![Some(9), Some(9)], vec![Some(6), Some(6)]];
    let a = s
        .admit(b"a", Side::TrainingInput, &pixels, &source)
        .unwrap();
    let b = s
        .admit(b"b", Side::TrainingInput, &pixels, &source)
        .unwrap();
    let mut r = RegionWitness::prepare_counted(
        &s,
        &a,
        &a.cell(0, 0).unwrap(),
        &native,
        b"r",
        &native.provision(b"cache"),
    )
    .unwrap();
    r.execute(r.budget).unwrap();
    let mut c = CompleteCountWitness::prepare(r.record().unwrap(), &owner, b"c").unwrap();
    c.execute(c.budget).unwrap();
    (owner, native, a, b, c.record().unwrap())
}
#[test]
fn retention_checks_actual_input_and_producer_owner() {
    let (owner, native, a, b, p) = setup();
    assert!(CompleteCountWitness::projection_from_committed(&owner, &native, &a, &p.proof).is_ok());
    assert!(
        CompleteCountWitness::projection_from_committed(&owner, &native, &b, &p.proof).is_err()
    );
    assert!(
        CompleteCountWitness::projection_from_committed(
            &Trie::default().root(),
            &native,
            &a,
            &p.proof
        )
        .is_err()
    );
    p.proof.child(b"committed").unwrap().point_to(&p.proof);
    assert!(
        CompleteCountWitness::projection_from_committed(&owner, &native, &a, &p.proof).is_err()
    );
}
#[test]
fn retained_certificate_preserves_original_grid_after_all_callers_drop() {
    let retained = {
        let (owner, native, a, _, p) = setup();
        CompleteCountWitness::projection_from_committed(&owner, &native, &a, &p.proof).unwrap()
    };
    assert_eq!(RegionWitness::dangling_links(&retained.proof), 0);
    assert!(retained.proof.child(b"committed").unwrap().is_terminal());
}
