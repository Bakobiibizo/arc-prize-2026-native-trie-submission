use native_trie_seed::disk_trie::DiskTrie;
use std::{
    fs,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

fn bytes(path: &std::path::Path) -> u64 {
    fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                bytes(&entry.path())
            } else {
                metadata.len()
            }
        })
        .sum()
}

fn main() {
    for count in [250usize, 1_000, 4_000] {
        let root = std::env::temp_dir().join(format!(
            "disk-trie-benchmark-{}-{}-{count}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let started = Instant::now();
        let trie = DiskTrie::create(&root).unwrap();
        for index in 0..count {
            let episode = trie
                .provision(&[b"episodes", &(index as u64).to_be_bytes()])
                .unwrap();
            let evidence = trie
                .provision(&[b"evidence", &(index as u64).to_be_bytes()])
                .unwrap();
            episode.point_to(&evidence).unwrap();
        }
        let write_micros = started.elapsed().as_micros();
        let read_started = Instant::now();
        for index in 0..count {
            let episode = trie
                .root()
                .locate([b"episodes".as_slice(), &(index as u64).to_be_bytes()])
                .unwrap()
                .unwrap();
            assert!(episode.follow().is_ok());
        }
        let read_micros = read_started.elapsed().as_micros();
        println!(
            "{{\"nodes\":{},\"write_micros\":{},\"read_micros\":{},\"disk_bytes\":{}}}",
            count * 2 + 3,
            write_micros,
            read_micros,
            bytes(&root)
        );
        drop(trie);
        fs::remove_dir_all(root).unwrap();
    }
}
