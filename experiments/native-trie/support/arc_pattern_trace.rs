//! Allocating diagnostic export ONLY, after model execution/inspection. Never imported by model.
use native_trie_seed::{Address, arc_grids::Grid, records};
fn coord(at: &Address) -> Option<usize> {
    Some(usize::from_be_bytes(at.token()?.try_into().ok()?))
}
pub(crate) fn origin(cell: &Address) -> String {
    let found = || -> Option<(String, usize, usize)> {
        let row = cell.child(b"source-row")?.follow().ok()?;
        let grid = row.child(b"source-grid")?.follow().ok()?;
        Some((
            String::from_utf8(grid.token()?.to_vec()).ok()?,
            coord(&row)?,
            coord(cell)?,
        ))
    };
    match found() {
        Some((id, y, x)) => format!("{{\"episode\":\"{id}\",\"y\":{y},\"x\":{x}}}"),
        None => "null".into(),
    }
}
pub(crate) fn color(at: &Address) -> String {
    match at.token() {
        Some([c]) if *c <= 9 => c.to_string(),
        _ => "null".into(),
    }
}
pub fn trace(id: &str, result: &Grid) {
    for y in 0..result.height() {
        for x in 0..result.width() {
            let cell = result.cell(y, x).unwrap();
            let work = cell.child(b"derived-from").unwrap().follow().unwrap();
            let input = work.child(b"input-cell").unwrap().follow().unwrap();
            let mut views = Vec::new();
            for axis in [b"horizontal".as_slice(), b"vertical"] {
                let view = work.child(b"contexts").unwrap().child(axis).unwrap();
                let status = view.child(b"status").unwrap().follow().unwrap();
                let mut candidates = Vec::new();
                if let Ok(anchor) = view.child(b"lookup").unwrap().follow() {
                    for candidate in records::inspect_candidates(&anchor, b"output") {
                        let target = candidate.follow().unwrap();
                        let mut sources = Vec::new();
                        for entry in records::inspect_evidence(&candidate).unwrap() {
                            let evidence = entry.follow().unwrap();
                            let before = evidence.child(b"input-cell").unwrap().follow().unwrap();
                            let after = evidence.child(b"output-cell").unwrap().follow().unwrap();
                            sources.push(format!(
                                "{{\"input\":{},\"output\":{},\"before\":{},\"after\":{}}}",
                                origin(&before),
                                origin(&after),
                                color(&before.child(b"value").unwrap().follow().unwrap()),
                                color(&after.child(b"value").unwrap().follow().unwrap())
                            ));
                        }
                        candidates.push(format!(
                            "{{\"target\":{},\"evidence\":[{}]}}",
                            color(&target),
                            sources.join(",")
                        ));
                    }
                }
                views.push(format!(
                    "\"{}\":{{\"status\":\"{}\",\"candidates\":[{}]}}",
                    std::str::from_utf8(axis).unwrap(),
                    std::str::from_utf8(status.token().unwrap()).unwrap(),
                    candidates.join(",")
                ));
            }
            eprintln!(
                "{{\"query\":\"{id}\",\"y\":{y},\"x\":{x},\"source\":{},\"emitted\":{},\"contexts\":{{{}}}}}",
                origin(&input),
                color(&work.child(b"value").unwrap().follow().unwrap()),
                views.join(",")
            );
        }
    }
}
