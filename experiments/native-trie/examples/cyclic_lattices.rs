//! GIVEN cycle lengths/background/E-S/NW roles: NOT learned ARC/score10.
use native_trie_seed::{
    arc_grids::{Grids, Side},
    graph_export,
    sparse_rewrite::SparseRewrite,
};
fn main() {
    let mut roots = Vec::new();
    let mut outputs = Vec::new();
    for (case, n, bg, palette, h, w) in [
        (0, 2, 7, vec![8, 1], 8, 9),
        (1, 3, 0, vec![4, 8, 3], 7, 7),
        (2, 4, 5, vec![9, 2, 7, 1], 9, 11),
    ] {
        let mut s = Grids::new();
        let mut a = vec![vec![Some(bg); w]; h];
        if case == 1 {
            for (y, row) in [
                "0000830", "0008300", "0083000", "0830004", "8300040", "3000400", "0004000",
            ]
            .iter()
            .enumerate()
            {
                for (x, b) in row.bytes().enumerate() {
                    a[y][x] = Some(b - b'0');
                }
            }
        } else {
            for (i, v) in palette.iter().enumerate() {
                a[0][i] = Some(*v);
            }
        }
        let src = s
            .source(
                b"fixture",
                b"declared cycle prototype, background and roles supplied; sparse data only",
            )
            .unwrap();
        let input = s.admit(b"input", Side::QueryInput, &a, &src).unwrap();
        let mut m = SparseRewrite::new(&s);
        let bg = input
            .cell(h - 1, w - 1)
            .unwrap()
            .child(b"value")
            .unwrap()
            .follow()
            .unwrap();
        let p = m.cyclic_program(&bg, n).unwrap();
        for gauge in 0..=1 {
            if gauge == 1 {
                let one = p
                    .root
                    .child(b"phases")
                    .unwrap()
                    .child(&1usize.to_be_bytes())
                    .unwrap();
                p.root.child(b"start-phase").unwrap().point_to(&one);
            }
            let out = m.cyclic_lattice(&mut s, &input, &p).unwrap();
            let data = out.inspect().unwrap();
            let wire = data
                .iter()
                .map(|r| {
                    format!(
                        "[{}]",
                        r.iter()
                            .map(|v| v.map_or_else(|| "null".into(), |n| n.to_string()))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "{{\"kind\":\"supplied-cyclic-prototype\",\"case\":{case},\"gauge\":{gauge},\"given_length\":{n},\"workspace_bytes\":{},\"grid\":[{wire}]}}",
                m.workspace_bytes()
            );
            roots.extend([m.root.clone(), input.anchor.clone(), out.anchor.clone()]);
            outputs.push(out);
        }
    }
    graph_export::write(&roots, &mut std::io::stderr()).unwrap();
    drop(outputs);
}
