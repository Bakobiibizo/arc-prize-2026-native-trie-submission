//! Diagnostic observation only: formatting is intentionally outside any performance/allocation claim.
use native_trie_seed::{Outcome, constructions::Builder, source_field, target_field};
fn main() {
    let mut b = Builder::new();
    let bob = b.literal(b"bob");
    let alice = b.literal(b"alice");
    let c0 = b.join(bob, alice);
    let first = c0.first.clone();
    let end = c0.end.clone();
    let context = c0.context.clone();
    let suffix = b.literal(b"!");
    let root = b.join(c0, suffix);
    let mut eval = b.finish(root);
    println!("input: mark (join bob alice)");
    println!(
        "before: first_unset={}, end_terminal={}",
        first.follow().is_err(),
        end.is_terminal()
    );
    for step in 0..150 {
        let state = eval.machine.current().unwrap();
        if state.is_terminal() {
            assert_eq!(eval.machine.cycle(), Ok(Outcome::Halted));
            println!("{step}: explicit terminal");
            println!(
                "D0={}",
                String::from_utf8(eval.inspect_view(&first, Some(&end), 32).unwrap()).unwrap()
            );
            println!(
                "result={}",
                String::from_utf8(eval.inspect(32).unwrap()).unwrap()
            );
            println!(
                "parent_saved={}, V0_first_saved={}, V0_end_saved={}, cycles={}",
                context.child(b"parent").unwrap().follow().is_ok(),
                context.child(b"bound-first").unwrap().follow().is_ok(),
                context.child(b"bound-end").unwrap().follow().is_ok(),
                eval.machine.cycles
            );
            return;
        }
        let source = source_field(&state).unwrap().follow().unwrap();
        let target = target_field(&state).unwrap().follow().unwrap();
        println!(
            "{step}: state_token={:?}; source_token={:?}; target_token={:?}",
            state.token(),
            source.token(),
            target.token()
        );
        assert_eq!(eval.machine.cycle(), Ok(Outcome::Advanced));
    }
    panic!("diagnostic cycle bound exhausted");
}
