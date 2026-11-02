use native_trie_seed::{Trie, axis_context::AxisEvent, paired_states::PairedStates};

fn event(
    traits: &Trie,
    occurrences: &Trie,
    position: &[u8],
    neighbour_position: &[u8],
    color: &[u8],
    neighbour_color: &[u8],
) -> AxisEvent {
    AxisEvent {
        position: traits.provision(&[b"position", position]),
        neighbour_position: traits.provision(&[b"neighbour-position", neighbour_position]),
        color: traits.provision(&[b"color", color]),
        neighbour_color: traits.provision(&[b"color", neighbour_color]),
        focal: occurrences.provision(&[b"focal"]),
        neighbour: occurrences.provision(&[b"neighbour"]),
    }
}

#[test]
fn literal_input_and_output_graphs_derive_four_axis_delta_from_trie_paths() {
    let traits = Trie::default();
    let occurrences = Trie::default();
    let evidence = occurrences.provision(&[b"demonstration"]);
    let input = [event(
        &traits,
        &occurrences,
        b"middle",
        b"west",
        b"blank",
        b"blue",
    )];
    let output = [event(
        &traits,
        &occurrences,
        b"middle",
        b"west",
        b"pink",
        b"blue",
    )];
    let mut states = PairedStates::default();
    let encoded_input = states.encode(b"input", &input, &evidence).unwrap();
    let encoded_output = states.encode(b"output", &output, &evidence).unwrap();
    assert_eq!(encoded_input.namespace.token(), Some(b"input".as_slice()));
    assert_eq!(encoded_output.namespace.token(), Some(b"output".as_slice()));
    let deltas = PairedStates::compare(&encoded_input, &encoded_output).unwrap();
    assert_eq!(deltas.len(), 1);
    assert!(!deltas[0].position_changed);
    assert!(!deltas[0].neighbour_position_changed);
    assert!(deltas[0].color_changed);
    assert!(!deltas[0].neighbour_color_changed);
    assert!(deltas[0].input.color.same(&input[0].color));
    assert!(deltas[0].output.color.same(&output[0].color));
}
