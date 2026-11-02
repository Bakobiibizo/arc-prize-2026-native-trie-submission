//! Explicit supplied character mappings and bounded input events, never a runtime tokenizer.
use native_trie_seed::{
    representations::{Representations, Resolution},
    words::CharacterEvent,
};
pub const CHARACTERS: &[(&[u8], &[u8], &[u8])] = &[
    (b"c", &[0x63], b"U+0063"),
    (b"a", &[0x61], b"U+0061"),
    (b"r", &[0x72], b"U+0072"),
    (b"t", &[0x74], b"U+0074"),
    (b"s", &[0x73], b"U+0073"),
    (b"f", &[0x66], b"U+0066"),
    (b"e-acute", &[0xC3, 0xA9], b"U+00E9"),
];
pub fn setup() -> Representations {
    let mut model = Representations::new();
    for (id, bytes, scalar) in CHARACTERS {
        let value = model.character(id);
        let source = model.source(id);
        let path: Vec<&[u8]> = std::iter::once(b"utf8".as_slice())
            .chain(bytes.iter().map(std::slice::from_ref))
            .collect();
        model.teach(&path, &value, &source).unwrap();
        model.teach(&[b"scalar", scalar], &value, &source).unwrap();
    }
    model
}
pub fn events(model: &Representations, indices: &[usize], scalar: bool) -> Vec<CharacterEvent> {
    indices
        .iter()
        .map(|i| {
            let (_, bytes, scalar_token) = CHARACTERS[*i];
            let path: Vec<&[u8]> = if scalar {
                vec![b"scalar", scalar_token]
            } else {
                std::iter::once(b"utf8".as_slice())
                    .chain(bytes.iter().map(std::slice::from_ref))
                    .collect()
            };
            match model.lookup(path).unwrap() {
                Resolution::Complete {
                    character,
                    candidate,
                    ..
                } => CharacterEvent {
                    character,
                    origin: candidate,
                },
                _ => panic!("fixture alias missing"),
            }
        })
        .collect()
}
