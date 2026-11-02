//! Supplied-role native axis prototype. This module proves physical full lines only; it does not
//! infer semantic roles, select a task, or claim the complete profile transformation.
use super::*;

#[derive(Clone)]
pub struct NativeAxisLineProof {
    pub root: Address,
    owner: Address,
}

#[derive(Clone)]
pub struct NativeAxisBridgeProof {
    pub root: Address,
    owner: Address,
}

#[derive(Clone)]
pub struct NativeAxisActivationProof {
    pub root: Address,
    owner: Address,
}

impl SparseRewrite {
    pub fn prove_native_marker_activation(
        &mut self,
        input: &Grid,
        marker: &Address,
        forward_port: &[u8],
        values: (&Address, &Address, &Address),
        separator: &NativeAxisLineProof,
    ) -> Result<NativeAxisActivationProof, Error> {
        let (marker_value, background_value, separator_value) = values;
        if !input.owner.same(&self.space)
            || !separator.owner.same(&self.root)
            || separator.root.child(b"complete").is_none()
        {
            return Err(Error::NotReady);
        }
        let proof = self
            .root
            .provision(b"native-axis-activations")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        proof.provision(b"input").point_to(&input.anchor);
        proof.provision(b"marker").point_to(marker);
        proof.provision(b"separator-line").point_to(&separator.root);
        proof
            .provision(b"forward-port")
            .provision(forward_port)
            .terminate();
        let actual_marker = field(marker, b"value")?.follow()?;
        let (is_marker, marker_fact) = self.context_equality_fact(&actual_marker, marker_value)?;
        proof
            .provision(b"native-marker-value")
            .point_to(&marker_fact);
        if !is_marker.same(&self.yes) {
            return Err(Error::Incomplete);
        }
        let separator_visited = field(&separator.root, b"visited")?;
        let unknown = self.unknown.clone();
        let mut current = marker.clone();
        loop {
            let step = proof.provision(b"walk").provision(&current.identity_key());
            if step.child(b"visited").is_some() {
                step.provision(b"cycle").terminate();
                return Err(Error::CycleLimit);
            }
            step.provision(b"visited").terminate();
            step.provision(b"cell").point_to(&current);
            let next = field(&current, forward_port)?.follow()?;
            step.provision(b"actual-next").point_to(&next);
            let outside = self.outside.clone();
            let (is_outside, outside_fact) = self.context_equality_fact(&next, &outside)?;
            step.provision(b"native-outside").point_to(&outside_fact);
            if is_outside.same(&self.yes) {
                step.provision(b"unreachable-separator").terminate();
                return Err(Error::Incomplete);
            }
            let value = field(&next, b"value")?.follow()?;
            step.provision(b"next-value-at-bind").point_to(&value);
            let (is_unknown, unknown_fact) = self.context_equality_fact(&value, &unknown)?;
            step.provision(b"native-known-check")
                .point_to(&unknown_fact);
            if is_unknown.same(&self.yes) {
                return Err(Error::Incomplete);
            }
            let (is_separator, separator_fact) =
                self.context_equality_fact(&value, separator_value)?;
            step.provision(b"native-separator-value")
                .point_to(&separator_fact);
            if is_separator.same(&self.yes) {
                if separator_visited.child(&next.identity_key()).is_none() {
                    step.provision(b"separator-not-in-line").terminate();
                    return Err(Error::Incomplete);
                }
                step.provision(b"activated-separator").point_to(&next);
                proof.provision(b"complete").terminate();
                return Ok(NativeAxisActivationProof {
                    root: proof,
                    owner: self.root.clone(),
                });
            }
            let (is_background, background_fact) =
                self.context_equality_fact(&value, background_value)?;
            let (is_intervening_marker, intervening_marker_fact) =
                self.context_equality_fact(&value, marker_value)?;
            step.provision(b"native-background")
                .point_to(&background_fact);
            step.provision(b"native-intervening-marker")
                .point_to(&intervening_marker_fact);
            if !is_background.same(&self.yes) && !is_intervening_marker.same(&self.yes) {
                step.provision(b"unexpected-intervening-value").terminate();
                return Err(Error::Incomplete);
            }
            current = next;
        }
    }

    pub fn prove_native_line_bridge(
        &mut self,
        input: &Grid,
        separator: &NativeAxisLineProof,
        ground: &NativeAxisLineProof,
        transverse_port: &[u8],
    ) -> Result<NativeAxisBridgeProof, Error> {
        if !input.owner.same(&self.space)
            || !separator.owner.same(&self.root)
            || !ground.owner.same(&self.root)
            || separator.root.child(b"complete").is_none()
            || ground.root.child(b"complete").is_none()
            || separator
                .root
                .child(b"forward-port")
                .and_then(|ports| ports.child(transverse_port))
                .is_none()
            || ground
                .root
                .child(b"forward-port")
                .and_then(|ports| ports.child(transverse_port))
                .is_none()
        {
            return Err(Error::NotReady);
        }
        let input_anchor = input.anchor.clone();
        for line in [separator, ground] {
            let recorded_input = field(&line.root, b"input")?.follow()?;
            let (same, fact) = self.context_equality_fact(&recorded_input, &input_anchor)?;
            line.root
                .provision(b"bridge-input-validation")
                .point_to(&fact);
            if !same.same(&self.yes) {
                return Err(Error::NotReady);
            }
        }
        let bridge = self
            .root
            .provision(b"native-axis-profile-bridges")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        bridge.provision(b"input").point_to(&input.anchor);
        bridge
            .provision(b"separator-line")
            .point_to(&separator.root);
        bridge.provision(b"ground-line").point_to(&ground.root);
        bridge
            .provision(b"transverse-port")
            .provision(transverse_port)
            .terminate();
        let separator_visited = field(&separator.root, b"visited")?;
        let ground_visited = field(&ground.root, b"visited")?;
        let mut separator_cell = field(&separator.root, b"start")?.follow()?;
        let mut ground_cell = field(&ground.root, b"start")?.follow()?;
        let outside = self.outside.clone();
        loop {
            let separator_step = separator_visited
                .child(&separator_cell.identity_key())
                .ok_or(Error::Incomplete)?;
            let ground_step = ground_visited
                .child(&ground_cell.identity_key())
                .ok_or(Error::Incomplete)?;
            if separator_step.child(b"visited-complete").is_none()
                || ground_step.child(b"visited-complete").is_none()
            {
                return Err(Error::Incomplete);
            }
            let pair = bridge
                .provision(b"pairs")
                .provision(&separator_cell.identity_key())
                .provision(&ground_cell.identity_key());
            pair.provision(b"separator").point_to(&separator_cell);
            pair.provision(b"ground").point_to(&ground_cell);
            let separator_next = field(&separator_cell, transverse_port)?.follow()?;
            let ground_next = field(&ground_cell, transverse_port)?.follow()?;
            pair.provision(b"separator-actual-next")
                .point_to(&separator_next);
            pair.provision(b"ground-actual-next").point_to(&ground_next);
            let (separator_finished, separator_fact) =
                self.context_equality_fact(&separator_next, &outside)?;
            let (ground_finished, ground_fact) =
                self.context_equality_fact(&ground_next, &outside)?;
            pair.provision(b"native-separator-boundary")
                .point_to(&separator_fact);
            pair.provision(b"native-ground-boundary")
                .point_to(&ground_fact);
            let separator_done = separator_finished.same(&self.yes);
            let ground_done = ground_finished.same(&self.yes);
            if separator_done != ground_done {
                pair.provision(b"inconsistent-boundary").terminate();
                return Err(Error::Incomplete);
            }
            pair.provision(b"paired-complete").terminate();
            if separator_done {
                bridge.provision(b"complete").terminate();
                return Ok(NativeAxisBridgeProof {
                    root: bridge,
                    owner: self.root.clone(),
                });
            }
            separator_cell = separator_next;
            ground_cell = ground_next;
        }
    }

    pub fn prove_native_full_line(
        &mut self,
        input: &Grid,
        start: &Address,
        forward_port: &[u8],
        backward_port: &[u8],
        expected_value: &Address,
        supplied_role: &[u8],
    ) -> Result<NativeAxisLineProof, Error> {
        if !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        let proof = self
            .root
            .provision(b"native-axis-profile-line-proofs")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        proof.provision(b"input").point_to(&input.anchor);
        proof
            .provision(b"supplied-role")
            .provision(supplied_role)
            .terminate();
        proof
            .provision(b"forward-port")
            .provision(forward_port)
            .terminate();
        proof
            .provision(b"backward-port")
            .provision(backward_port)
            .terminate();
        proof.provision(b"expected-value").point_to(expected_value);
        proof.provision(b"start").point_to(start);

        let row = field(start, b"source-row")?.follow()?;
        let source_grid = field(&row, b"source-grid")?.follow()?;
        let (owned, owned_fact) = self.context_equality_fact(&source_grid, &input.anchor)?;
        proof.provision(b"native-start-owner").point_to(&owned_fact);
        if !owned.same(&self.yes) {
            proof.provision(b"foreign-start").terminate();
            return Err(Error::NotReady);
        }
        let backward = field(start, backward_port)?.follow()?;
        let outside = self.outside.clone();
        let (at_boundary, boundary_fact) = self.context_equality_fact(&backward, &outside)?;
        proof
            .provision(b"native-start-boundary")
            .point_to(&boundary_fact);
        if !at_boundary.same(&self.yes) {
            proof.provision(b"start-not-boundary").terminate();
            return Err(Error::Incomplete);
        }
        let unknown = self.unknown.clone();
        let visited = proof.provision(b"visited");
        let mut current = start.clone();
        loop {
            let key = current.identity_key();
            if visited.child(&key).is_some() {
                proof.provision(b"duplicate-visit").point_to(&current);
                return Err(Error::CycleLimit);
            }
            let step = visited.provision(&key);
            step.provision(b"cell").point_to(&current);
            let value = field(&current, b"value")?.follow()?;
            step.provision(b"value-at-bind").point_to(&value);
            let (unknown_tag, unknown_fact) = self.context_equality_fact(&value, &unknown)?;
            step.provision(b"native-known-check")
                .point_to(&unknown_fact);
            if unknown_tag.same(&self.yes) {
                step.provision(b"unknown-value").terminate();
                return Err(Error::Incomplete);
            }
            let (expected, expected_fact) = self.context_equality_fact(&value, expected_value)?;
            step.provision(b"native-expected-value")
                .point_to(&expected_fact);
            if !expected.same(&self.yes) {
                step.provision(b"unexpected-value").terminate();
                return Err(Error::Incomplete);
            }
            let next = field(&current, forward_port)?.follow()?;
            step.provision(b"actual-next").point_to(&next);
            let (finished, outside_fact) = self.context_equality_fact(&next, &outside)?;
            step.provision(b"native-opposite-boundary")
                .point_to(&outside_fact);
            step.provision(b"visited-complete").terminate();
            if finished.same(&self.yes) {
                proof.provision(b"complete").terminate();
                return Ok(NativeAxisLineProof {
                    root: proof,
                    owner: self.root.clone(),
                });
            }
            current = next;
        }
    }
}

impl NativeAxisActivationProof {
    pub fn belongs_to(&self, learner: &SparseRewrite) -> bool {
        self.owner.same(&learner.root)
    }
}

impl NativeAxisBridgeProof {
    pub fn belongs_to(&self, learner: &SparseRewrite) -> bool {
        self.owner.same(&learner.root)
    }

    pub fn pair_count(&self) -> usize {
        self.root.child(b"pairs").map_or(0, |pairs| {
            pairs
                .inspect_children()
                .into_iter()
                .map(|separator| separator.inspect_children().len())
                .sum()
        })
    }
}

impl NativeAxisLineProof {
    pub fn belongs_to(&self, learner: &SparseRewrite) -> bool {
        self.owner.same(&learner.root)
    }

    pub fn visited_count(&self) -> usize {
        self.root
            .child(b"visited")
            .map_or(0, |visited| visited.inspect_children().len())
    }
}
