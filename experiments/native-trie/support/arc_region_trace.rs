//! Post-inference graph export. IDs/linear dedup are diagnostic only, not model state.
use crate::pattern_trace::{color, origin};
use native_trie_seed::{Address, arc_grids::Grid, records, region_witness::RegionWitness};
fn id(items: &mut Vec<Address>, a: &Address) -> usize {
    if let Some(i) = items.iter().position(|b| a.same(b)) {
        i
    } else {
        items.push(a.clone());
        items.len() - 1
    }
}
pub fn trace(query: &str, result: &Grid) {
    let mut proofs = Vec::new();
    let mut associations = Vec::new();
    for y in 0..result.height() {
        for x in 0..result.width() {
            let work = result
                .cell(y, x)
                .unwrap()
                .child(b"derived-from")
                .unwrap()
                .follow()
                .unwrap();
            let proof = work.child(b"region-proof").unwrap().follow().unwrap();
            let p = id(&mut proofs, &proof);
            let a = work
                .child(b"lookup")
                .unwrap()
                .follow()
                .ok()
                .map(|a| id(&mut associations, &a).to_string())
                .unwrap_or_else(|| "null".into());
            let value = work.child(b"value").unwrap().follow().unwrap();
            eprintln!(
                "{{\"kind\":\"cell\",\"query\":\"{query}\",\"y\":{y},\"x\":{x},\"proof\":{p},\"association\":{a},\"emitted\":{}}}",
                color(&value)
            );
        }
    }
    for (i, association) in associations.iter().enumerate() {
        let mut targets = Vec::new();
        for candidate in records::inspect_candidates(association, b"output") {
            let target = candidate.follow().unwrap();
            let mut evidence = Vec::new();
            for e in records::inspect_evidence(&candidate).unwrap() {
                let source = e.follow().unwrap();
                let before = source.child(b"input-cell").unwrap().follow().unwrap();
                let after = source.child(b"output-cell").unwrap().follow().unwrap();
                let p = source
                    .child(b"source-proof")
                    .and_then(|p| p.follow().ok())
                    .map(|p| id(&mut proofs, &p).to_string())
                    .unwrap_or_else(|| "null".into());
                evidence.push(format!(
                    "{{\"before\":{},\"after\":{},\"proof\":{p}}}",
                    origin(&before),
                    origin(&after)
                ));
            }
            targets.push(format!(
                "{{\"value\":{},\"evidence\":[{}]}}",
                color(&target),
                evidence.join(",")
            ));
        }
        eprintln!(
            "{{\"kind\":\"association\",\"query\":\"{query}\",\"id\":{i},\"targets\":[{}]}}",
            targets.join(",")
        );
    }
    for (i, proof) in proofs.iter().enumerate() {
        let result = proof.child(b"result").unwrap().follow().unwrap();
        let seed = proof.child(b"seed").unwrap().follow().unwrap();
        let exhausted = proof.child(b"frontier-exhausted").unwrap().follow().is_ok();
        let mut facts = Vec::new();
        for fact in RegionWitness::inspect_facts(proof).unwrap() {
            let class = fact
                .class
                .as_ref()
                .and_then(|a| a.token())
                .and_then(|t| std::str::from_utf8(t).ok())
                .unwrap_or("unvisited");
            let edges = fact
                .edges
                .iter()
                .map(|(neighbor, checked)| {
                    format!(
                        "{{\"neighbor\":{},\"checked\":{checked}}}",
                        origin(neighbor)
                    )
                })
                .collect::<Vec<_>>();
            facts.push(format!(
                "{{\"source\":{},\"class\":\"{class}\",\"expanded\":{},\"edges\":[{}]}}",
                origin(&fact.source),
                fact.expanded,
                edges.join(",")
            ));
        }
        eprintln!(
            "{{\"kind\":\"proof\",\"query\":\"{query}\",\"id\":{i},\"seed\":{},\"result\":\"{}\",\"exhausted\":{exhausted},\"facts\":[{}]}}",
            origin(&seed),
            std::str::from_utf8(result.token().unwrap()).unwrap(),
            facts.join(",")
        );
    }
}
