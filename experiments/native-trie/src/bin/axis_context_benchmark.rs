use native_trie_seed::{
    Trie,
    axis_context::{AxisConstructions, AxisEpisodes, AxisEvent},
    region_witness::RegionWitness,
};
use std::time::Instant;

fn main() {
    for size in [1_000usize, 2_000, 4_000, 8_000, 16_000] {
        let traits = Trie::default();
        let occurrences = Trie::default();
        let programs = Trie::default();
        let position_kind = traits.provision(&[b"positions"]);
        let neighbour_position = traits.provision(&[b"neighbour-position", b"east"]);
        let color = traits.provision(&[b"color", b"blank"]);
        let neighbour_color = traits.provision(&[b"color", b"blue"]);
        let identity = programs.provision(&[b"identity"]);
        let constructions = AxisConstructions {
            position: identity.clone(),
            neighbour_position: identity.clone(),
            color: programs.provision(&[b"blank-to-pink"]),
            neighbour_color: identity,
        };
        let setup = Instant::now();
        let events = (0..size)
            .map(|index| AxisEvent {
                position: position_kind.provision(&index.to_be_bytes()),
                neighbour_position: neighbour_position.clone(),
                color: color.clone(),
                neighbour_color: neighbour_color.clone(),
                focal: occurrences.provision(&[b"focal", &index.to_be_bytes()]),
                neighbour: occurrences.provision(&[b"neighbour", &index.to_be_bytes()]),
            })
            .collect::<Vec<_>>();
        let setup_ns = setup.elapsed().as_nanos();
        let mut episodes = AxisEpisodes::default();
        let teach = Instant::now();
        for (index, event) in events.iter().enumerate() {
            episodes
                .teach(
                    std::slice::from_ref(event),
                    &constructions,
                    &occurrences.provision(&[b"evidence", &index.to_be_bytes()]),
                )
                .unwrap();
        }
        let teach_ns = teach.elapsed().as_nanos();
        let resolve = Instant::now();
        for event in &events {
            episodes.resolve(std::slice::from_ref(event)).unwrap();
        }
        let resolve_ns = resolve.elapsed().as_nanos();
        println!(
            "{{\"events\":{size},\"setup_ns\":{setup_ns},\"teach_ns\":{teach_ns},\"resolve_ns\":{resolve_ns},\"episode_storage_bytes\":{}}}",
            RegionWitness::storage_charge(&episodes.root())
        );
    }
}
