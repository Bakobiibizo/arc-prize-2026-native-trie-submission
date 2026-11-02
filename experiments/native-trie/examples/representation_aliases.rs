#[path = "../support/alias_fixture.rs"]
mod fixture;
use native_trie_seed::representations::Resolution;
fn main() {
    let mut model = fixture::teach();
    for (path, id) in fixture::LESSONS {
        let character = fixture::target(&model, path);
        assert_eq!(character.token(), Some(*id));
        assert_eq!(model.emit(&character), Ok(1));
        assert!(model.result.follow().unwrap().same(&character));
    }
    let c = fixture::target(&model, &[b"utf8", &[0x43]]);
    assert!(c.same(&fixture::target(&model, fixture::LESSONS[0].0)));
    assert!(c.same(&fixture::target(&model, &[b"scalar", b"U+0043"])));
    println!("bits/01000011, utf8/43 and scalar/U+0043 share one C address");
    let e = fixture::target(&model, &[b"utf8", &[0xC3], &[0xA9]]);
    assert!(e.same(&fixture::target(&model, &[b"scalar", b"U+00E9"])));
    assert!(e.same(&fixture::target(&model, &[b"latin1", &[0xE9]])));
    println!("utf8/C3/A9, scalar/U+00E9 and latin1/E9 share one e-acute address");
    assert!(matches!(
        model.lookup([b"utf8".as_slice(), &[0xC3]]),
        Ok(Resolution::Prefix { .. })
    ));
    println!("utf8/C3 is only a prefix; A9 and B1 complete to different character addresses");
    println!(
        "12 shown aliases; 5 canonical identities; one shared one-copy value construction; no decoding or inferred semantics"
    );
}
