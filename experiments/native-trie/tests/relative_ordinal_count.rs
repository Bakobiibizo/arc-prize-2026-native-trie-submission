use native_trie_seed::{
    Address, Trie,
    arc_grids::{Grids, Side},
    complete_count_witness::CompleteCountWitness,
    ordinal_relation::OrdinalInput,
    records,
    reference_set_count::SetCountWitness,
    region_witness::{RegionError, RegionWitness},
    relative_ordinal_count::{OrdinalCandidate, RelativeOrdinalCount},
};
use std::rc::Rc;
fn count(root: &Address, id: &[u8], n: usize, unknown: bool) -> Rc<OrdinalInput> {
    let set = root.provision(b"inputs").provision(id);
    for i in 0..n {
        let value = set.provision(b"values").provision(&i.to_be_bytes());
        records::record(&set, b"reference", &value, &value).unwrap();
    }
    let missing = root.provision(b"set-count-tags").provision(b"unavailable");
    let slot = set.provision(b"availability");
    slot.point_to(if unknown { &missing } else { &set });
    let mut w = SetCountWitness::prepare(
        set,
        b"reference",
        &[(slot, missing.clone())],
        root,
        id,
        Vec::new(),
    )
    .unwrap();
    w.execute(w.budget).unwrap();
    OrdinalInput::from_set(w.record().unwrap(), missing)
}
fn ordinal(root: &Address, n: usize) -> Address {
    let mut a = RegionWitness::cardinality_zero(root);
    for _ in 0..n {
        a = a.follow().unwrap();
    }
    a
}
fn guard(root: &Address, reserve: usize) -> Result<(), RegionError> {
    if RegionWitness::storage_charge(root) + reserve > 286720000 {
        Err(RegionError::Limit)
    } else {
        Ok(())
    }
}
#[test]
fn counts_distinct_larger_references_keeps_ties_and_repeated_occurrences() {
    let root = Trie::default().root();
    let subject = count(&root, b"s", 2, false);
    let a = count(&root, b"a", 5, false);
    let b = count(&root, b"b", 5, false);
    let c = count(&root, b"c", 1, false);
    let r0 = root.provision(b"r0");
    let r1 = root.provision(b"r1");
    let cs = vec![
        OrdinalCandidate {
            reference: r0.clone(),
            count: a.clone(),
        },
        OrdinalCandidate {
            reference: r1,
            count: b,
        },
        OrdinalCandidate {
            reference: root.provision(b"r2"),
            count: c,
        },
        OrdinalCandidate {
            reference: root.provision(b"equal"),
            count: subject.clone(),
        },
        OrdinalCandidate {
            reference: r0,
            count: a,
        },
    ];
    let zero = ordinal(&root, 0);
    let mut f = RelativeOrdinalCount::new(&root, &zero);
    let p = f
        .prepare(subject, &cs, &[], b"relative", |n| guard(&root, n))
        .unwrap();
    assert!(p.count().value.same(&ordinal(&root, 2)));
    assert_eq!(f.bank_count(), 3);
    assert!(
        p.evidence
            .child(b"occurrences")
            .unwrap()
            .child(&4usize.to_be_bytes())
            .is_some()
    );
    assert_eq!(RegionWitness::dangling_links(&p.count().proof), 0);
    println!(
        "relative banks={} comparison_copies={} count_copies={} final_proxy={}",
        f.bank_count(),
        f.comparison_copies,
        f.count_copies,
        RegionWitness::storage_charge(&root)
    );
}
#[test]
fn unavailable_candidate_subject_or_universe_never_becomes_zero() {
    for which in 0..3 {
        let root = Trie::default().root();
        let subject = count(&root, b"s", 1, which == 0);
        let candidate = count(&root, b"c", 2, which == 1);
        let unknown = count(&root, b"u", 0, true);
        let cs = vec![OrdinalCandidate {
            reference: root.provision(b"r"),
            count: candidate,
        }];
        let universe = if which == 2 {
            vec![(
                unknown.proof.child(b"value").unwrap(),
                unknown.unavailable.clone(),
            )]
        } else {
            Vec::new()
        };
        let mut f = RelativeOrdinalCount::new(&root, &ordinal(&root, 0));
        let p = f
            .prepare(subject, &cs, &universe, b"rank", |n| guard(&root, n))
            .unwrap();
        assert!(
            p.count().value.same(
                &root
                    .child(b"set-count-tags")
                    .unwrap()
                    .child(b"unavailable")
                    .unwrap()
            )
        );
    }
}
#[test]
fn empty_declared_universe_is_zero_only_with_known_subject() {
    let root = Trie::default().root();
    let subject = count(&root, b"s", 1, false);
    let mut f = RelativeOrdinalCount::new(&root, &ordinal(&root, 0));
    let p = f
        .prepare(subject, &[], &[], b"zero", |n| guard(&root, n))
        .unwrap();
    assert!(p.count().value.same(&ordinal(&root, 0)));
}
#[test]
fn native_region_counts_retain_relative_order_under_absolute_size_palette_geometry_changes() {
    for (case, sizes, colour, bg) in [
        (0, [8usize, 5, 4, 2], 9u8, 6u8),
        (1, [14usize, 10, 9, 6], 4u8, 0u8),
    ] {
        let s = Grids::new();
        let root = Trie::default().root();
        let mut pixels = vec![vec![Some(bg); 24]; 20];
        for (j, size) in sizes.iter().enumerate() {
            for x in 0..*size {
                pixels[2 + j * 4][2 + x] = Some(colour);
            }
        }
        let src = s
            .source(
                b"fixture",
                b"independent absolute-size/palette/geometry native-region count fixture",
            )
            .unwrap();
        let grid = s
            .admit(b"input", Side::TrainingInput, &pixels, &src)
            .unwrap();
        let mut cs = Vec::new();
        for j in 0..4usize {
            let mut w = RegionWitness::prepare_counted(
                &s,
                &grid,
                &grid.cell(2 + j * 4, 2).unwrap(),
                &root,
                &j.to_be_bytes(),
                &root.provision(b"cache"),
            )
            .unwrap();
            w.execute(w.budget).unwrap();
            let region = w.record().unwrap();
            let reference = region.proof.clone();
            let mut c = CompleteCountWitness::prepare(region, &root, &j.to_be_bytes()).unwrap();
            c.execute(c.budget).unwrap();
            let value = OrdinalInput::from_complete(
                c.record().unwrap(),
                root.provision(b"complete-count-tags")
                    .provision(b"unavailable"),
            );
            cs.push(OrdinalCandidate {
                reference,
                count: value,
            });
        }
        let mut f = RelativeOrdinalCount::new(&root, &ordinal(&root, 0));
        for j in 0..4usize {
            let p = f
                .prepare(cs[j].count.clone(), &cs, &[], &j.to_be_bytes(), |n| {
                    guard(&root, n)
                })
                .unwrap();
            assert!(p.count().value.same(&ordinal(&root, j)));
        }
        println!(
            "native-regions case={case} banks={} comparison_copies={} count_copies={} final_proxy={}",
            f.bank_count(),
            f.comparison_copies,
            f.count_copies,
            RegionWitness::storage_charge(&root)
        );
    }
}
#[test]
fn relative_value_as_operand_retains_all_original_native_grid_owners() {
    let input = {
        let s = Grids::new();
        let root = Trie::default().root();
        let src = s
            .source(b"s", b"retained relative operand lifetime fixture")
            .unwrap();
        let grid = s
            .admit(
                b"i",
                Side::TrainingInput,
                &[
                    vec![Some(6); 5],
                    vec![Some(4), Some(6), Some(6), Some(4), Some(4)],
                    vec![Some(6); 5],
                ],
                &src,
            )
            .unwrap();
        let mut cs = Vec::new();
        for (j, x) in [0usize, 3].iter().enumerate() {
            let mut w = RegionWitness::prepare_counted(
                &s,
                &grid,
                &grid.cell(1, *x).unwrap(),
                &root,
                &j.to_be_bytes(),
                &root.provision(b"cache"),
            )
            .unwrap();
            w.execute(w.budget).unwrap();
            let r = w.record().unwrap();
            let reference = r.proof.clone();
            let mut w = CompleteCountWitness::prepare(r, &root, &j.to_be_bytes()).unwrap();
            w.execute(w.budget).unwrap();
            cs.push(OrdinalCandidate {
                reference,
                count: OrdinalInput::from_complete(
                    w.record().unwrap(),
                    root.provision(b"complete-count-tags")
                        .provision(b"unavailable"),
                ),
            });
        }
        let mut f = RelativeOrdinalCount::new(&root, &ordinal(&root, 0));
        let p = f
            .prepare(cs[0].count.clone(), &cs, &[], b"rank", |n| guard(&root, n))
            .unwrap();
        assert!(p.count().value.same(&ordinal(&root, 1)));
        OrdinalInput::from_relative(
            p,
            root.provision(b"set-count-tags").provision(b"unavailable"),
        )
    };
    assert_eq!(RegionWitness::dangling_links(&input.proof), 0);
    assert_eq!(
        input.current().unwrap().token(),
        Some(b"successor".as_slice())
    );
}
#[test]
fn native_unknown_frontier_count_propagates_unavailable() {
    let s = Grids::new();
    let root = Trie::default().root();
    let src = s.source(b"s", b"unknown frontier fixture").unwrap();
    let grid = s
        .admit(
            b"i",
            Side::TrainingInput,
            &[vec![Some(4), None], vec![Some(4), Some(6)]],
            &src,
        )
        .unwrap();
    let mut w = RegionWitness::prepare_counted(
        &s,
        &grid,
        &grid.cell(0, 0).unwrap(),
        &root,
        b"r",
        &root.provision(b"cache"),
    )
    .unwrap();
    w.execute(w.budget).unwrap();
    let region = w.record().unwrap();
    let reference = region.proof.clone();
    let mut w = CompleteCountWitness::prepare(region, &root, b"c").unwrap();
    w.execute(w.budget).unwrap();
    let input = OrdinalInput::from_complete(
        w.record().unwrap(),
        root.provision(b"complete-count-tags")
            .provision(b"unavailable"),
    );
    let cs = vec![OrdinalCandidate {
        reference,
        count: input.clone(),
    }];
    let mut f = RelativeOrdinalCount::new(&root, &ordinal(&root, 0));
    let p = f
        .prepare(input, &cs, &[], b"relative", |n| guard(&root, n))
        .unwrap();
    assert!(
        p.count().value.same(
            &root
                .child(b"set-count-tags")
                .unwrap()
                .child(b"unavailable")
                .unwrap()
        )
    );
}
