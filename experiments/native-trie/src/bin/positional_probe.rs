//! TSV import / native positional experiment / JSON recording. Query expected outputs absent.
use native_trie_seed::{
    arc_grids::{Grids, Side},
    graph_export,
    positional::{Error, Positional},
};
fn grid(s: &str) -> Vec<Vec<Option<u8>>> {
    s.split('/')
        .map(|r| r.split(',').map(|c| Some(c.parse().unwrap())).collect())
        .collect()
}
fn main() {
    let raw = std::fs::read(std::env::args().nth(1).expect("TSV path")).unwrap();
    let text = std::str::from_utf8(&raw).unwrap();
    let mut s = Grids::new();
    let mut m = Positional::default();
    let mut derived = Vec::new();
    let mut queries = Vec::new();
    let mut shape_errors = 0;
    for line in text.lines() {
        let f: Vec<_> = line.split('\t').collect();
        let source = s.source(f[1].as_bytes(), line.as_bytes()).unwrap();
        match f[0] {
            "TRAIN" => {
                let a = s
                    .admit(
                        format!("{}-in", f[1]).as_bytes(),
                        Side::TrainingInput,
                        &grid(f[2]),
                        &source,
                    )
                    .unwrap();
                let b = s
                    .admit(
                        format!("{}-out", f[1]).as_bytes(),
                        Side::TrainingOutput,
                        &grid(f[3]),
                        &source,
                    )
                    .unwrap();
                match m.derive(f[1].as_bytes(), &a, &b) {
                    Ok(d) => derived.push(d),
                    Err(Error::Shape) => shape_errors += 1,
                    Err(e) => panic!("{e:?}"),
                }
            }
            "QUERY" => {
                assert_eq!(f.len(), 3, "query expected output forbidden");
                let a = s
                    .admit(f[1].as_bytes(), Side::QueryInput, &grid(f[2]), &source)
                    .unwrap();
                queries.push((f[1].to_owned(), a));
            }
            _ => panic!("unknown record"),
        }
    }
    let agreed = if shape_errors == 0 {
        if std::env::args().any(|a| a == "--candidate-intersection") {
            m.agree_candidates(&derived).unwrap()
        } else {
            Positional::agree(&derived)
        }
    } else {
        None
    };
    println!(
        "{{\"kind\":\"derivation\",\"pairs\":{},\"shape_errors\":{},\"unresolved_correspondences\":{},\"matched\":{}}}",
        derived.len() + shape_errors,
        shape_errors,
        derived.iter().map(|d| d.unresolved).sum::<usize>(),
        agreed.is_some()
    );
    for (id, q) in queries {
        if let Some(ref e) = agreed {
            match m.begin(&q, e) {
                Ok(()) => {
                    let copies = m.execute(3 * q.width() * q.height()).unwrap();
                    let out = m.record(&mut s).unwrap();
                    println!(
                        "{{\"kind\":\"prediction\",\"id\":\"{}\",\"copies\":{},\"grid\":{:?}}}",
                        id,
                        copies,
                        out.inspect()
                            .unwrap()
                            .iter()
                            .map(|r| r.iter().map(|v| v.unwrap()).collect::<Vec<_>>())
                            .collect::<Vec<_>>()
                    );
                }
                Err(Error::Shape) => println!(
                    "{{\"kind\":\"prediction\",\"id\":\"{}\",\"status\":\"shape-outside-concrete-encoding\"}}",
                    id
                ),
                Err(e) => panic!("{e:?}"),
            }
        } else {
            println!(
                "{{\"kind\":\"prediction\",\"id\":\"{}\",\"status\":\"no-matching-complete-encoding\"}}",
                id
            )
        }
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
