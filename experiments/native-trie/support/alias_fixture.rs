//! Explicit shown associations, not a runtime encoding/decoding table. Used only for teaching,
//! supplying benchmark query addresses, and independent output validation.
use native_trie_seed::{Address, representations::Representations};
pub const LESSONS: &[(&[&[u8]], &[u8])] = &[
    (
        &[b"bits", &[0], &[1], &[0], &[0], &[0], &[0], &[1], &[1]],
        b"C",
    ),
    (&[b"utf8", &[0x43]], b"C"),
    (&[b"scalar", b"U+0043"], b"C"),
    (&[b"utf8", &[0xC3], &[0xA9]], b"e-acute"),
    (&[b"scalar", b"U+00E9"], b"e-acute"),
    (&[b"latin1", &[0xE9]], b"e-acute"),
    (&[b"utf8", &[0xC3], &[0xB1]], b"n-tilde"),
    (&[b"scalar", b"U+00F1"], b"n-tilde"),
    (&[b"utf8", &[0xE2], &[0x82], &[0xAC]], b"euro"),
    (&[b"scalar", b"U+20AC"], b"euro"),
    (&[b"utf8", &[0xF0], &[0x9F], &[0x98], &[0x80]], b"grin"),
    (&[b"scalar", b"U+1F600"], b"grin"),
];
pub fn teach() -> Representations {
    let mut model = Representations::new();
    for (i, (path, id)) in LESSONS.iter().enumerate() {
        let character = model.character(id);
        let source = model.source(&i.to_be_bytes());
        // Original supplied tokens, already typed by the representation selector; no conversion.
        path.iter()
            .fold(source.provision(b"supplied-path"), |at, key| {
                at.provision(key)
            });
        model.teach(path, &character, &source).unwrap();
    }
    model
}
pub fn target(model: &Representations, path: &[&[u8]]) -> Address {
    match model.lookup(path.iter().copied()).unwrap() {
        native_trie_seed::representations::Resolution::Complete { character, .. } => character,
        _ => panic!("expected taught complete address"),
    }
}
