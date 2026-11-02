//! Generic longest recorded-prefix selection over actual trie edges. No range/encoding logic.
use crate::{
    Address,
    records::{self, RecordError, Selection},
};
pub struct PrefixMatch {
    pub anchor: Address,
    pub candidate: Address,
    pub construction: Address,
    pub depth: usize,
}
pub enum PrefixSelection {
    Unknown,
    Match(PrefixMatch),
    Ambiguous {
        anchor: Address,
        depth: usize,
        alternatives: usize,
    },
}
pub fn select<'a>(
    root: &Address,
    tokens: impl IntoIterator<Item = &'a [u8]>,
) -> Result<PrefixSelection, RecordError> {
    let mut at = root.clone();
    let mut best = match records::resolve(root, b"construction")? {
        Selection::Unknown => PrefixSelection::Unknown,
        Selection::Unique { candidate, target } => PrefixSelection::Match(PrefixMatch {
            anchor: root.clone(),
            candidate,
            construction: target,
            depth: 0,
        }),
        Selection::Ambiguous { alternatives } => PrefixSelection::Ambiguous {
            anchor: root.clone(),
            depth: 0,
            alternatives,
        },
    };
    for (depth, token) in tokens.into_iter().enumerate() {
        let Some(next) = at.child(token) else {
            break;
        };
        at = next;
        match records::resolve(&at, b"construction")? {
            Selection::Unknown => {}
            Selection::Unique { candidate, target } => {
                best = PrefixSelection::Match(PrefixMatch {
                    anchor: at.clone(),
                    candidate,
                    construction: target,
                    depth: depth + 1,
                })
            }
            Selection::Ambiguous { alternatives } => {
                best = PrefixSelection::Ambiguous {
                    anchor: at.clone(),
                    depth: depth + 1,
                    alternatives,
                }
            }
        }
    }
    Ok(best)
}
