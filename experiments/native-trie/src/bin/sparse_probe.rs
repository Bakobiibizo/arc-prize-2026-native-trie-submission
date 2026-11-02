//! Import, actual native sparse relational experiment, export. No query expected values.
use native_trie_seed::{
    arc_grids::{Grids, Side},
    graph_export,
    positional::Error,
    sparse_rewrite::{GeneralizedCandidate, SparseRewrite},
};
type PlainGrid = Vec<Vec<u8>>;

fn plain_grid(grid: &native_trie_seed::arc_grids::Grid) -> PlainGrid {
    grid.inspect()
        .unwrap()
        .iter()
        .map(|row| row.iter().map(|value| value.unwrap()).collect())
        .collect()
}

fn validated_plain_grid(grid: &native_trie_seed::arc_grids::Grid) -> Option<PlainGrid> {
    let inspected = grid.inspect().ok()?;
    let width = inspected.first()?.len();
    if inspected.len() > 30 || width == 0 || width > 30 {
        return None;
    }
    inspected
        .iter()
        .map(|row| {
            if row.len() != width {
                return None;
            }
            row.iter()
                .map(|value| value.filter(|color| *color <= 9))
                .collect::<Option<Vec<_>>>()
        })
        .collect()
}

fn distinct_secondary(primary: &PlainGrid, candidate: PlainGrid) -> Option<PlainGrid> {
    (candidate != *primary).then_some(candidate)
}

fn duplicate_attempt_2(
    id: &str,
    primary: &PlainGrid,
    candidate: Option<&GeneralizedCandidate>,
    replay_valid: bool,
    reason: &str,
) {
    if let Some(candidate) = candidate {
        let origin = candidate.origin().as_str();
        let index = candidate.replay_address_index().unwrap_or(usize::MAX);
        println!(
            "{{\"kind\":\"attempt-2\",\"id\":\"{id}\",\"status\":\"duplicate-attempt-1\",\"reason\":\"{reason}\",\"candidate_origin\":\"{origin}\",\"candidate_index\":{index},\"candidate_address\":\"ordered-replay-valid-candidates/{index}\",\"all_demonstrations_replayed\":{replay_valid},\"replay_valid\":{replay_valid},\"independent\":false,\"grid\":{:?}}}",
            primary
        );
    } else {
        println!(
            "{{\"kind\":\"attempt-2\",\"id\":\"{id}\",\"status\":\"duplicate-attempt-1\",\"reason\":\"{reason}\",\"candidate_origin\":\"none\",\"all_demonstrations_replayed\":false,\"replay_valid\":false,\"independent\":false,\"grid\":{:?}}}",
            primary
        );
    }
}

fn emit_attempt_2(
    machine: &mut SparseRewrite,
    grids: &mut Grids,
    query: &native_trie_seed::arc_grids::Grid,
    id: &str,
    primary: &PlainGrid,
    alternatives: &[GeneralizedCandidate],
) {
    if alternatives.is_empty() {
        duplicate_attempt_2(
            id,
            primary,
            None,
            false,
            "no-secondary-replay-valid-candidate",
        );
        return;
    }
    let mut fallback = None;
    for candidate in alternatives {
        let replay_valid = candidate.has_all_demonstration_replay_provenance();
        let result: Result<(), &str> = if !replay_valid {
            Err("secondary-replay-provenance-invalid")
        } else {
            match machine.source_pair_applicable(query, candidate.program()) {
                Ok(false) => Err("secondary-inapplicable"),
                Err(_) => Err("secondary-applicability-error"),
                Ok(true) => match machine.apply_groups(grids, query, candidate.program()) {
                    Err(_) => Err("secondary-application-error"),
                    Ok(output) => match validated_plain_grid(&output) {
                        None => Err("secondary-invalid-grid"),
                        Some(output) => match distinct_secondary(primary, output) {
                            None => Err("secondary-output-identical"),
                            Some(grid) => {
                                let origin = candidate.origin().as_str();
                                let index = candidate
                                    .replay_address_index()
                                    .expect("validated provenance has an address");
                                println!(
                                    "{{\"kind\":\"attempt-2\",\"id\":\"{id}\",\"status\":\"distinct-replay-valid-candidate\",\"reason\":\"secondary-applicable\",\"candidate_origin\":\"{origin}\",\"candidate_index\":{index},\"candidate_address\":\"ordered-replay-valid-candidates/{index}\",\"all_demonstrations_replayed\":true,\"replay_valid\":true,\"independent\":true,\"grid\":{:?}}}",
                                    grid
                                );
                                return;
                            }
                        },
                    },
                },
            }
        };
        if fallback.is_none() {
            fallback = Some((candidate, replay_valid, result.unwrap_err()));
        }
    }
    let (candidate, replay_valid, reason) =
        fallback.expect("nonempty alternatives recorded a fallback");
    duplicate_attempt_2(id, primary, Some(candidate), replay_valid, reason);
}
fn grid(s: &str) -> Vec<Vec<Option<u8>>> {
    s.split('/')
        .map(|r| r.split(',').map(|c| Some(c.parse().unwrap())).collect())
        .collect()
}
fn failure(m: &SparseRewrite, phase: &str, id: &str, error: &Error) -> ! {
    println!(
        "{{\"kind\":\"failure\",\"phase\":\"{phase}\",\"id\":\"{id}\",\"error\":\"{error:?}\",\"workspace_bytes\":{},\"comparison_copies\":{},\"emission_copies\":{}}}",
        m.workspace_bytes(),
        m.comparison_copies,
        m.emission_copies
    );
    graph_export::write(
        &[
            m.root.clone(),
            m.machine.states.root(),
            m.machine.output.root(),
        ],
        &mut std::io::stderr(),
    )
    .unwrap();
    std::process::exit(1)
}
fn main() {
    let benchmark = std::env::args().any(|a| a == "--benchmark-timings");
    let skip_graph_export = std::env::args().any(|a| a == "--skip-graph-export");
    let total_started = std::time::Instant::now();
    let cyclic = std::env::args().any(|a| a == "--cyclic-lattice-actions");
    let ordered = cyclic || std::env::args().any(|a| a == "--ordered-object-actions");
    let path_copy = ordered || std::env::args().any(|a| a == "--path-value-actions");
    let source_pairs = path_copy || std::env::args().any(|a| a == "--source-pair-actions");
    let relative = source_pairs || std::env::args().any(|a| a == "--relative-actions");
    assert!(
        !(relative && std::env::args().any(|a| a == "--context-composition")),
        "choose one input representation"
    );
    let composition = relative || std::env::args().any(|a| a == "--context-composition");
    let contexts = composition || std::env::args().any(|a| a == "--component-contexts");
    assert!(
        !(contexts && std::env::args().any(|a| a == "--binding-generalization")),
        "choose one input representation"
    );
    let generalize = contexts || std::env::args().any(|a| a == "--binding-generalization");
    assert!(
        !(generalize && std::env::args().any(|a| a == "--action-partitions")),
        "choose one input representation"
    );
    let partitions = generalize || std::env::args().any(|a| a == "--action-partitions");
    assert!(
        !(partitions && std::env::args().any(|a| a == "--source-groups")),
        "choose one input representation"
    );
    let groups = partitions || std::env::args().any(|a| a == "--source-groups");
    let component = std::env::args().any(|a| a == "--component-relations");
    assert!(
        !groups || (!component && !std::env::args().any(|a| a == "--input-relations")),
        "choose one input representation"
    );
    assert!(
        !(component && std::env::args().any(|a| a == "--input-relations")),
        "choose one input representation"
    );
    let raw = std::fs::read(std::env::args().nth(1).expect("TSV")).unwrap();
    let mut s = Grids::new();
    let mut m = SparseRewrite::new(&s);
    let mut ds = Vec::new();
    let mut grouped = Vec::new();
    let mut training_grids = Vec::new();
    let mut queries = Vec::new();
    let mut shape_errors = 0;
    for line in std::str::from_utf8(&raw).unwrap().lines() {
        let f: Vec<_> = line.split('\t').collect();
        let src = s.source(f[1].as_bytes(), line.as_bytes()).unwrap();
        match f[0] {
            "TRAIN" => {
                let i = s
                    .admit(
                        format!("{}-in", f[1]).as_bytes(),
                        Side::TrainingInput,
                        &grid(f[2]),
                        &src,
                    )
                    .unwrap();
                let o = s
                    .admit(
                        format!("{}-out", f[1]).as_bytes(),
                        Side::TrainingOutput,
                        &grid(f[3]),
                        &src,
                    )
                    .unwrap();
                if groups {
                    training_grids.push((i.clone(), o.clone()));
                    match m.derive_groups(f[1].as_bytes(), &i, &o) {
                        Ok(d) => {
                            println!(
                                "{{\"kind\":\"group-lesson\",\"id\":\"{}\",\"cells\":{},\"changed\":{},\"groups\":{}}}",
                                f[1],
                                i.width() * i.height(),
                                d.changed,
                                d.groups
                            );
                            grouped.push(d);
                        }
                        Err(Error::Shape) => shape_errors += 1,
                        Err(e) => failure(&m, "teach", f[1], &e),
                    }
                    continue;
                }
                let derived = if component {
                    m.derive_component_relations(f[1].as_bytes(), &i, &o)
                } else if std::env::args().any(|a| a == "--input-relations") {
                    m.derive_input_relations(f[1].as_bytes(), &i, &o)
                } else {
                    m.derive(f[1].as_bytes(), &i, &o)
                };
                match derived {
                    Ok(d) => {
                        if d.evidence
                            .child(b"negative-validation-short-circuited")
                            .is_some()
                        {
                            println!(
                                "{{\"kind\":\"negative-coverage\",\"id\":\"{}\",\"checked\":{},\"unchecked_after_counterexample\":{}}}",
                                f[1], d.negative_checks, d.negative_unchecked
                            );
                        }
                        println!(
                            "{{\"kind\":\"lesson\",\"id\":\"{}\",\"cells\":{},\"changed\":{},\"positive_tests\":{},\"negative_checks\":{},\"clauses\":{},\"valid\":{}}}",
                            f[1],
                            i.width() * i.height(),
                            d.changed,
                            d.positive_tests,
                            d.negative_checks,
                            d.rule.as_ref().map_or(0, |r| r.clauses()),
                            d.rule.is_some()
                        );
                        ds.push(d)
                    }
                    Err(Error::Shape) => shape_errors += 1,
                    Err(e) => failure(&m, "teach", f[1], &e),
                }
            }
            "QUERY" => {
                assert_eq!(f.len(), 3);
                queries.push((
                    f[1].to_owned(),
                    s.admit(f[1].as_bytes(), Side::QueryInput, &grid(f[2]), &src)
                        .unwrap(),
                ));
            }
            _ => panic!("record"),
        }
    }
    let training_ns = total_started.elapsed().as_nanos();
    if groups {
        let agreement_started = std::time::Instant::now();
        let mut alternatives: Vec<GeneralizedCandidate> = Vec::new();
        let program = if shape_errors == 0 {
            if cyclic {
                match m.agree_generalized_candidates(&grouped) {
                    Ok(candidates) => {
                        alternatives.extend(candidates.candidates().iter().skip(1).cloned());
                        candidates
                            .primary()
                            .map(|candidate| candidate.program().clone())
                    }
                    Err(e) => failure(&m, "agree", "all-lessons", &e),
                }
            } else {
                match if ordered {
                    m.agree_ordered_objects(&grouped)
                } else if path_copy {
                    m.agree_path_value_actions(&grouped)
                } else if source_pairs {
                    m.agree_source_pair_actions(&grouped)
                } else if relative {
                    m.agree_relative_actions(&grouped)
                } else if composition {
                    m.agree_context_composition(&grouped)
                } else if contexts {
                    m.agree_context_actions(&grouped)
                } else if partitions {
                    m.agree_partitions(&grouped)
                } else {
                    m.agree_groups(&grouped)
                } {
                    Ok(Some(p)) if generalize && !contexts => {
                        match m.generalize_source_bindings(&grouped, p) {
                            Ok(p) => Some(p),
                            Err(e) => failure(&m, "generalize", "all-lessons", &e),
                        }
                    }
                    Ok(p) => p,
                    Err(e) => failure(&m, "agree", "all-lessons", &e),
                }
            }
        } else if cyclic {
            match m.agree_canonical_whole_grid_transforms(&training_grids) {
                Ok(programs) => {
                    alternatives.extend(
                        programs
                            .into_iter()
                            .map(GeneralizedCandidate::broad_replay_valid),
                    );
                    None
                }
                Err(e) => failure(&m, "agree-whole-grid", "all-lessons", &e),
            }
        } else {
            None
        };
        if cyclic && shape_errors == 0 {
            match m.agree_canonical_whole_grid_transforms(&training_grids) {
                Ok(programs) => alternatives.extend(
                    programs
                        .into_iter()
                        .map(GeneralizedCandidate::broad_replay_valid),
                ),
                Err(e) => failure(&m, "agree-whole-grid", "all-lessons", &e),
            }
        }
        let agreement_ns = agreement_started.elapsed().as_nanos();
        println!(
            "{{\"kind\":\"derivation\",\"pairs\":{},\"matched\":{},\"groups\":{}}}",
            grouped.len() + shape_errors,
            program.is_some(),
            program.as_ref().map_or(0, |p| p.groups())
        );
        if source_pairs {
            let family = program.as_ref().map_or("none", |p| {
                if p.root.child(b"cyclic-lattice-program").is_some() {
                    "cyclic-lattice"
                } else if p.root.child(b"ordered-object-program").is_some() {
                    "ordered-object"
                } else if p.root.child(b"path-copy-program").is_some() {
                    "path-copy"
                } else if p.root.child(b"source-pair-program").is_some() {
                    "source-pair"
                } else {
                    "prior"
                }
            });
            println!("{{\"kind\":\"programme-family\",\"family\":\"{family}\"}}");
        }
        let query_started = std::time::Instant::now();
        for (id, q) in queries {
            if let Some(ref p) = program {
                match m.source_pair_applicable(&q, p) {
                    Ok(true) => {}
                    Ok(false) => {
                        println!(
                            "{{\"kind\":\"prediction\",\"id\":\"{id}\",\"status\":\"no-matching-valid-relational-encoding\"}}"
                        );
                        if cyclic {
                            let fallback = plain_grid(&q);
                            emit_attempt_2(&mut m, &mut s, &q, &id, &fallback, &alternatives);
                        }
                        continue;
                    }
                    Err(e) => {
                        if cyclic {
                            let fallback = plain_grid(&q);
                            emit_attempt_2(&mut m, &mut s, &q, &id, &fallback, &alternatives);
                        }
                        failure(&m, "query-bind", &id, &e)
                    }
                }
                let c = m.comparison_copies;
                let e = m.emission_copies;
                let out = match m.apply_groups(&mut s, &q, p) {
                    Ok(o) => o,
                    Err(e) => {
                        if cyclic {
                            let fallback = plain_grid(&q);
                            emit_attempt_2(&mut m, &mut s, &q, &id, &fallback, &alternatives);
                        }
                        failure(&m, "query", &id, &e)
                    }
                };
                let primary = plain_grid(&out);
                println!(
                    "{{\"kind\":\"prediction\",\"id\":\"{}\",\"comparison_copies\":{},\"emission_copies\":{},\"grid\":{:?}}}",
                    id,
                    m.comparison_copies - c,
                    m.emission_copies - e,
                    primary
                );
                if cyclic {
                    emit_attempt_2(&mut m, &mut s, &q, &id, &primary, &alternatives);
                }
            } else {
                println!(
                    "{{\"kind\":\"prediction\",\"id\":\"{}\",\"status\":\"no-matching-valid-relational-encoding\"}}",
                    id
                );
                if cyclic {
                    let fallback = plain_grid(&q);
                    emit_attempt_2(&mut m, &mut s, &q, &id, &fallback, &alternatives);
                }
            }
        }
        let query_ns = query_started.elapsed().as_nanos();
        println!(
            "{{\"kind\":\"component-cost\",\"native_copies\":{},\"proofs\":{},\"cache_hits\":{},\"workspace_bytes\":{}}}",
            m.region_copies,
            m.region_proofs,
            m.region_cache_hits,
            m.workspace_bytes()
        );
        if path_copy {
            println!(
                "{{\"kind\":\"membership-cost\",\"native_copies\":{},\"proofs\":{}}}",
                m.membership_copies, m.membership_proofs
            );
        }
        if relative {
            let (banks, comparisons, counts) = m.relative_statistics();
            println!(
                "{{\"kind\":\"relative-cost\",\"banks\":{banks},\"comparison_copies\":{comparisons},\"count_copies\":{counts}}}"
            );
        }
        if contexts {
            println!(
                "{{\"kind\":\"reference-set-cost\",\"native_copies\":{},\"proofs\":{}}}",
                m.set_copies, m.set_proofs
            );
        }
        if partitions {
            println!(
                "{{\"kind\":\"count-property-cost\",\"native_copies\":{},\"proofs\":{}}}",
                m.count_copies, m.count_proofs
            );
        }
        if benchmark {
            println!(
                "{{\"kind\":\"benchmark-timing\",\"training_ns\":{training_ns},\"agreement_ns\":{agreement_ns},\"query_ns\":{query_ns},\"pre_export_total_ns\":{}}}",
                total_started.elapsed().as_nanos()
            );
        }
        if !skip_graph_export {
            graph_export::write(
                &[
                    m.root.clone(),
                    m.machine.states.root(),
                    m.machine.output.root(),
                ],
                &mut std::io::stderr(),
            )
            .unwrap();
        }
        return;
    }
    let rule = if shape_errors == 0 {
        SparseRewrite::agree(&ds)
    } else {
        None
    };
    println!(
        "{{\"kind\":\"derivation\",\"pairs\":{},\"matched\":{}}}",
        ds.len() + shape_errors,
        rule.is_some()
    );
    for (id, q) in queries {
        if let Some(ref r) = rule {
            let comparisons = m.comparison_copies;
            let emissions = m.emission_copies;
            let out = match m.apply(&mut s, &q, r) {
                Ok(out) => out,
                Err(e) => failure(&m, "query", &id, &e),
            };
            println!(
                "{{\"kind\":\"prediction\",\"id\":\"{}\",\"comparison_copies\":{},\"emission_copies\":{},\"grid\":{:?}}}",
                id,
                m.comparison_copies - comparisons,
                m.emission_copies - emissions,
                out.inspect()
                    .unwrap()
                    .iter()
                    .map(|r| r.iter().map(|v| v.unwrap()).collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            );
        } else {
            println!(
                "{{\"kind\":\"prediction\",\"id\":\"{}\",\"status\":\"no-matching-valid-relational-encoding\"}}",
                id
            );
        }
    }
    if component {
        println!(
            "{{\"kind\":\"component-cost\",\"native_copies\":{},\"proofs\":{},\"cache_hits\":{},\"workspace_bytes\":{}}}",
            m.region_copies,
            m.region_proofs,
            m.region_cache_hits,
            m.workspace_bytes()
        );
    }
    graph_export::write(
        &[
            m.root.clone(),
            m.machine.states.root(),
            m.machine.output.root(),
        ],
        &mut std::io::stderr(),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_or_absent_secondary_is_suppressed() {
        let primary = vec![vec![1, 2], vec![3, 4]];
        assert!(distinct_secondary(&primary, primary.clone()).is_none());
        assert_eq!(
            distinct_secondary(&primary, vec![vec![4, 3], vec![2, 1]]),
            Some(vec![vec![4, 3], vec![2, 1]])
        );
    }
}
