//! Lossless post-computation reachable native graph observations. Diagnostic IDs only;
//! NOT a portable model loader, semantic reconstruction or inference index.
use crate::{Address, Fault};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};
fn id(
    nodes: &mut Vec<Address>,
    ids: &mut BTreeMap<[u8; std::mem::size_of::<usize>()], usize>,
    at: &Address,
) -> usize {
    if let Some(i) = ids.get(&at.identity_key()) {
        *i
    } else {
        let i = nodes.len();
        ids.insert(at.identity_key(), i);
        nodes.push(at.clone());
        i
    }
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for b in bytes {
        write!(out, "{b:02x}").unwrap();
    }
    out
}
pub fn write(roots: &[Address], out: &mut impl Write) -> io::Result<usize> {
    let mut nodes = vec![];
    let mut ids = BTreeMap::new();
    let roots = roots
        .iter()
        .map(|r| id(&mut nodes, &mut ids, r).to_string())
        .collect::<Vec<_>>();
    writeln!(
        out,
        "{{\"kind\":\"native-graph-roots\",\"roots\":[{}],\"scope\":\"diagnostic opaque tokens/IDs; not portable identity or model input\"}}",
        roots.join(",")
    )?;
    let mut index = 0;
    while index < nodes.len() {
        let at = nodes[index].clone();
        let children = at
            .inspect_children()
            .iter()
            .map(|c| id(&mut nodes, &mut ids, c).to_string())
            .collect::<Vec<_>>();
        let content = if at.is_terminal() {
            "{\"end\":true}".into()
        } else {
            match at.follow() {
                Ok(next) => format!("{{\"ref\":{}}}", id(&mut nodes, &mut ids, &next)),
                Err(Fault::Unset) => "{\"unset\":true}".into(),
                Err(e) => format!("{{\"fault\":\"{e:?}\"}}"),
            }
        };
        let token = at
            .token()
            .map(|b| format!("\"{}\"", hex(b)))
            .unwrap_or("null".into());
        writeln!(
            out,
            "{{\"kind\":\"native-node\",\"id\":{index},\"token_hex\":{token},\"content\":{content},\"children\":[{}]}}",
            children.join(",")
        )?;
        index += 1;
    }
    Ok(nodes.len())
}
