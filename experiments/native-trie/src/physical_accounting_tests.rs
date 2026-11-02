use crate::{Address, Trie, region_witness::RegionWitness};
#[test]
fn physical_accounting_preserves_original_charge_and_shared_path_multiplicity() {
    let root = Trie::default().root();
    for i in 0..2000usize {
        let mut at = root.provision(&i.to_be_bytes());
        for n in 0..6 {
            at = at.provision(&vec![n; n as usize + 1]);
        }
    }
    let shared = Address::prepared_node(b"shared");
    shared.provision(b"token-bytes").terminate();
    root.provision(b"left").publish_prepared_child(&shared);
    root.provision(b"right").publish_prepared_child(&shared);
    fn original(root: &Address) -> (usize, usize) {
        let mut stack = vec![root.clone()];
        let (mut nodes, mut bytes) = (0, 0);
        while let Some(a) = stack.pop() {
            nodes += 1;
            bytes += 512 + a.token().map_or(0, |t| t.len());
            stack.extend(a.inspect_children());
        }
        (nodes, bytes)
    }
    let (nodes, bytes) = original(&root);
    assert_eq!(RegionWitness::owned_nodes(&root), nodes);
    assert_eq!(RegionWitness::storage_charge(&root), bytes);
    assert_eq!(nodes, 14007);
    let now = std::time::Instant::now();
    for _ in 0..16 {
        assert_eq!(original(&root).1, bytes)
    }
    let old = now.elapsed();
    let now = std::time::Instant::now();
    for _ in 0..16 {
        assert_eq!(RegionWitness::storage_charge(&root), bytes)
    }
    println!(
        "same-charge nodes={nodes} bytes={bytes} original_us={} direct_children_us={}",
        old.as_micros(),
        now.elapsed().as_micros()
    );
}
