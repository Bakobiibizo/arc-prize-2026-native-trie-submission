#[path = "../support/word_fixture.rs"]
mod fixture;
use native_trie_seed::words::{WordResolution, Words};
fn main() {
    let model = fixture::setup();
    let mut words = Words::new();
    let source = model.source(b"given-lexical-span");
    let car_utf8 = fixture::events(&model, &[0, 1, 2], false);
    let car_scalar = fixture::events(&model, &[0, 1, 2], true);
    let a = words.teach(&car_utf8, &source).unwrap();
    let b = words.teach(&car_scalar, &source).unwrap();
    assert!(a.candidate.same(&b.candidate));
    println!(
        "car via UTF-8 aliases and scalar aliases: same native word endpoint, distinct provenance episodes"
    );
    words
        .teach(&fixture::events(&model, &[0, 1, 3], false), &source)
        .unwrap();
    words
        .teach(&fixture::events(&model, &[0, 1, 2, 4], false), &source)
        .unwrap();
    words
        .teach(&fixture::events(&model, &[0, 1, 5, 6], false), &source)
        .unwrap();
    let chars: Vec<_> = car_utf8.iter().map(|e| e.character.clone()).collect();
    let prefix = words.locate(&chars[..2]).unwrap().unwrap();
    assert!(matches!(words.resolve(&prefix), Ok(WordResolution::Prefix)));
    let car = words.locate(&chars).unwrap().unwrap();
    assert!(matches!(
        words.resolve(&car),
        Ok(WordResolution::Complete { .. })
    ));
    assert!(!car.is_terminal());
    println!(
        "car/cat/cars/cafe-acute share native character prefixes; car remains complete with a longer child path"
    );
    println!("no word string concatenation, splitting, tokenization, decoder or VM copy execution");
}
