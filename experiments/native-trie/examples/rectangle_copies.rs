//! Supplied geometry/order/background/anchors: native proof/copy prototype,
//! NOT learned ARC programme or query-derived demonstrations.
use native_trie_seed::{
    arc_grids::{Grids, Side},
    graph_export,
    sparse_rewrite::SparseRewrite,
};
use std::rc::Rc;
fn main() {
    let mut roots = Vec::new();
    let mut outputs = Vec::new();
    for (case, bg, items, h, w) in [
        (0, 7, vec![(5, 1, 2, 2, 3), (4, 5, 5, 2, 2)], 8, 9),
        (
            1,
            0,
            vec![(9, 1, 1, 3, 4), (6, 5, 5, 2, 2), (8, 8, 8, 2, 3)],
            12,
            14,
        ),
    ] {
        let mut s = Grids::new();
        let mut a = vec![vec![Some(bg); w]; h];
        for &(v, y, x, hh, ww) in &items {
            for row in a.iter_mut().skip(y).take(hh) {
                for cell in row.iter_mut().skip(x).take(ww) {
                    *cell = Some(v);
                }
            }
        }
        let source = s
            .source(
                b"fixture",
                b"declared supplied ordered rectangles; no learner/query gold",
            )
            .unwrap();
        let input = s.admit(b"input", Side::QueryInput, &a, &source).unwrap();
        let mut m = SparseRewrite::new(&s);
        let mut rects = Vec::new();
        for &(_, y, x, _, _) in &items {
            rects.push(Rc::new(
                m.rectangle_source(&input, &input.cell(y, x).unwrap())
                    .unwrap(),
            ));
        }
        let bg = input
            .cell(0, 0)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap();
        let program = m.rectangle_copy_program(&rects, &bg, b"NW", b"SE").unwrap();
        let mut copy = m.prepare_rectangle_copy(&input, &program, rects).unwrap();
        let copies = copy.execute(copy.budget).unwrap();
        let output = copy.record(&mut s).unwrap();
        let data = output.inspect().unwrap();
        let wire = data
            .iter()
            .map(|row| {
                format!(
                    "[{}]",
                    row.iter()
                        .map(|v| v.map_or_else(|| "null".into(), |n| n.to_string()))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "{{\"kind\":\"supplied-rectangle-prototype\",\"case\":{case},\"native_copy_writes\":{copies},\"workspace_bytes\":{},\"grid\":[{wire}]}}",
            m.workspace_bytes()
        );
        roots.extend([m.root.clone(), input.anchor.clone(), output.anchor.clone()]);
        outputs.push(output);
    }
    graph_export::write(&roots, &mut std::io::stderr()).unwrap();
    drop(outputs);
}
