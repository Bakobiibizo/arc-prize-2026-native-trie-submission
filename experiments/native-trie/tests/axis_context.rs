use native_trie_seed::{
    Trie,
    axis_context::{
        AxisConstructions, AxisEpisodes, AxisEvent, ConstructionDags, DagInput, EpisodeError,
    },
    records::{self, Selection},
};

fn event(
    traits: &Trie,
    occurrences: &Trie,
    position: &[u8],
    neighbour_position: &[u8],
    color: &[u8],
    neighbour_color: &[u8],
    occurrence: &[u8],
) -> AxisEvent {
    AxisEvent {
        position: traits.provision(&[b"position", position]),
        neighbour_position: traits.provision(&[b"neighbour-position", neighbour_position]),
        color: traits.provision(&[b"color", color]),
        neighbour_color: traits.provision(&[b"color", neighbour_color]),
        focal: occurrences.provision(&[occurrence, b"focal"]),
        neighbour: occurrences.provision(&[occurrence, b"neighbour"]),
    }
}

#[test]
fn four_axis_episode_is_independent_of_neighbour_discovery_order() {
    let traits = Trie::default();
    let occurrences = Trie::default();
    let west = event(
        &traits,
        &occurrences,
        b"middle",
        b"west",
        b"blank",
        b"blue",
        b"w",
    );
    let east = event(
        &traits,
        &occurrences,
        b"middle",
        b"east",
        b"blank",
        b"blue",
        b"e",
    );
    let programs = Trie::default();
    let constructions = AxisConstructions {
        position: programs.provision(&[b"identity-position"]),
        neighbour_position: programs.provision(&[b"identity-neighbour-position"]),
        color: programs.provision(&[b"blank-to-pink"]),
        neighbour_color: programs.provision(&[b"identity-neighbour-color"]),
    };
    let evidence = programs.provision(&[b"demonstration"]);
    let mut episodes = AxisEpisodes::default();
    let taught = episodes
        .teach(&[west.clone(), east.clone()], &constructions, &evidence)
        .unwrap();
    let resolved = episodes.resolve(&[east, west]).unwrap();
    assert!(resolved.endpoint.same(&taught));
    assert!(resolved.constructions.color.same(&constructions.color));
    assert!(
        resolved
            .constructions
            .position
            .same(&constructions.position)
    );
}

#[test]
fn different_neighbour_axis_does_not_fall_back_to_horizontal_context() {
    let traits = Trie::default();
    let occurrences = Trie::default();
    let west = event(
        &traits,
        &occurrences,
        b"middle",
        b"west",
        b"blank",
        b"blue",
        b"w",
    );
    let east = event(
        &traits,
        &occurrences,
        b"middle",
        b"east",
        b"blank",
        b"blue",
        b"e",
    );
    let north = event(
        &traits,
        &occurrences,
        b"middle",
        b"north",
        b"blank",
        b"blue",
        b"n",
    );
    let south = event(
        &traits,
        &occurrences,
        b"middle",
        b"south",
        b"blank",
        b"blue",
        b"s",
    );
    let programs = Trie::default();
    let identity = programs.provision(&[b"identity"]);
    let constructions = AxisConstructions {
        position: identity.clone(),
        neighbour_position: identity.clone(),
        color: programs.provision(&[b"blank-to-pink"]),
        neighbour_color: identity,
    };
    let evidence = programs.provision(&[b"demonstration"]);
    let mut episodes = AxisEpisodes::default();
    episodes
        .teach(&[west, east], &constructions, &evidence)
        .unwrap();
    assert!(matches!(
        episodes.resolve(&[north, south]),
        Err(EpisodeError::Missing)
    ));
}

#[test]
fn conflicting_axis_constructions_remain_ambiguous() {
    let traits = Trie::default();
    let occurrences = Trie::default();
    let cue = event(
        &traits,
        &occurrences,
        b"middle",
        b"west",
        b"blank",
        b"blue",
        b"w",
    );
    let programs = Trie::default();
    let identity = programs.provision(&[b"identity"]);
    let first = AxisConstructions {
        position: identity.clone(),
        neighbour_position: identity.clone(),
        color: programs.provision(&[b"pink"]),
        neighbour_color: identity.clone(),
    };
    let second = AxisConstructions {
        color: programs.provision(&[b"red"]),
        ..first.clone()
    };
    let mut episodes = AxisEpisodes::default();
    episodes
        .teach(
            std::slice::from_ref(&cue),
            &first,
            &programs.provision(&[b"e1"]),
        )
        .unwrap();
    episodes
        .teach(
            std::slice::from_ref(&cue),
            &second,
            &programs.provision(&[b"e2"]),
        )
        .unwrap();
    assert!(matches!(
        episodes.resolve(&[cue]),
        Err(EpisodeError::Ambiguous)
    ));
}

#[test]
fn identical_dag_structure_consolidates_implementations_and_evidence() {
    let vocabulary = Trie::default();
    let operation = vocabulary.provision(&[b"operation", b"copy-reference"]);
    let role = vocabulary.provision(&[b"role", b"middle-result"]);
    let dependency = vocabulary.provision(&[b"construction", b"input-reference"]);
    let implementation = vocabulary.provision(&[b"machine-entry"]);
    let e1 = vocabulary.provision(&[b"evidence", b"one"]);
    let e2 = vocabulary.provision(&[b"evidence", b"two"]);
    let mut dags = ConstructionDags::default();
    let inputs = [DagInput {
        role,
        construction: dependency,
    }];
    let first = dags
        .intern(&operation, &inputs, &implementation, &e1)
        .unwrap();
    let second = dags
        .intern(&operation, &inputs, &implementation, &e2)
        .unwrap();
    assert!(first.same(&second));
    assert!(dags.implementation(&first).unwrap().same(&implementation));
    let candidate = match records::resolve(&first, b"implementation").unwrap() {
        Selection::Unique { candidate, .. } => candidate,
        _ => panic!("one implementation expected"),
    };
    assert_eq!(records::inspect_evidence(&candidate).unwrap().len(), 2);
}
