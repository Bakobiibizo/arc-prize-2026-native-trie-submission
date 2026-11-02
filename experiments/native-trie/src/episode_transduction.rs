//! End-to-end local episode induction and native application probe.
use crate::{
    Address, Fault, Machine, Outcome, Trie,
    axis_context::{
        AxisConstructions, AxisEpisodes, AxisEvent, ConstructionDags, DagInput, EpisodeError,
    },
    change_masks::{ChangeMasks, MaskError},
    disk_arc::DiskArc,
    disk_trie::{DiskAddress, DiskFault},
    install,
    paired_states::{PairedStates, StateError},
    target_field,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransductionError {
    Shape,
    NoChange,
    UnsupportedBoundaryChange,
    Episode(EpisodeError),
    State(StateError),
    Mask(MaskError),
    Machine(Fault),
    Incomplete,
    Ambiguous,
    Limit,
    ValidationMismatch,
    Persistent(DiskFault),
}
impl From<EpisodeError> for TransductionError {
    fn from(value: EpisodeError) -> Self {
        Self::Episode(value)
    }
}
impl From<StateError> for TransductionError {
    fn from(value: StateError) -> Self {
        Self::State(value)
    }
}
impl From<MaskError> for TransductionError {
    fn from(value: MaskError) -> Self {
        Self::Mask(value)
    }
}
impl From<Fault> for TransductionError {
    fn from(value: Fault) -> Self {
        Self::Machine(value)
    }
}
impl From<DiskFault> for TransductionError {
    fn from(value: DiskFault) -> Self {
        Self::Persistent(value)
    }
}

enum PendingAction {
    Write {
        index: usize,
        entry: Address,
        source: Option<Address>,
    },
    Move {
        source: usize,
        destination: usize,
        entry: Address,
    },
}

pub struct LocalEpisodeTransducer {
    pub states: PairedStates,
    pub contexts: AxisEpisodes,
    pub dags: ConstructionDags,
    pub masks: ChangeMasks,
    vocabulary: Trie,
    occurrences: Trie,
    component_rules: Trie,
    machine: Machine,
    frame_target: Address,
    frame_source: Address,
    frame_destination: Address,
    halt: Address,
    serial: usize,
    pub applications: u64,
    pub copy_cycles: u64,
}

impl Default for LocalEpisodeTransducer {
    fn default() -> Self {
        let machine = Machine::default();
        let frame = machine.states.root().provision(b"active-frame");
        let frame_target = frame.provision(b"target-slot");
        let frame_source = frame.provision(b"source-value");
        let frame_destination = frame.provision(b"destination-slot");
        let halt = machine.states.root().provision(b"episode-halt");
        halt.terminate();
        Self {
            states: PairedStates::default(),
            contexts: AxisEpisodes::default(),
            dags: ConstructionDags::default(),
            masks: ChangeMasks::default(),
            vocabulary: Trie::default(),
            occurrences: Trie::default(),
            component_rules: Trie::default(),
            machine,
            frame_target,
            frame_source,
            frame_destination,
            halt,
            serial: 0,
            applications: 0,
            copy_cycles: 0,
        }
    }
}

impl LocalEpisodeTransducer {
    fn position(&self) -> Address {
        self.vocabulary.provision(&[b"position", b"interior-focal"])
    }
    fn neighbour_position(&self, port: &[u8]) -> Address {
        self.vocabulary.provision(&[b"neighbour-position", port])
    }
    fn occurrence(&self, episode: usize, index: usize, role: &[u8]) -> Address {
        self.occurrences.provision(&[
            b"episodes",
            &episode.to_be_bytes(),
            &index.to_be_bytes(),
            role,
        ])
    }
    fn context(&self, values: &[Address], index: usize, episode: usize) -> Vec<AxisEvent> {
        [
            (b"west".as_slice(), index - 1),
            (b"east".as_slice(), index + 1),
        ]
        .into_iter()
        .map(|(port, neighbour)| AxisEvent {
            position: self.position(),
            neighbour_position: self.neighbour_position(port),
            color: values[index].clone(),
            neighbour_color: values[neighbour].clone(),
            focal: self.occurrence(episode, index, b"focal"),
            neighbour: self.occurrence(episode, neighbour, b"neighbour"),
        })
        .collect()
    }
    fn graph(&self, values: &[Address], episode: usize) -> Vec<AxisEvent> {
        (1..values.len().saturating_sub(1))
            .flat_map(|index| self.context(values, index, episode))
            .collect()
    }

    fn foreground_components(values: &[Address], height: usize, width: usize) -> Vec<Vec<usize>> {
        let mut visited = vec![false; values.len()];
        let mut components = Vec::new();
        for start in 0..values.len() {
            if visited[start] || values[start].token().is_some_and(|token| token == [0]) {
                continue;
            }
            let color = values[start].clone();
            let mut stack = vec![start];
            let mut members = Vec::new();
            visited[start] = true;
            while let Some(index) = stack.pop() {
                members.push(index);
                let y = index / width;
                let x = index % width;
                for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let Some(ny) = y.checked_add_signed(dy) else {
                        continue;
                    };
                    let Some(nx) = x.checked_add_signed(dx) else {
                        continue;
                    };
                    if ny >= height || nx >= width {
                        continue;
                    }
                    let neighbour = ny * width + nx;
                    if !visited[neighbour] && values[neighbour].same(&color) {
                        visited[neighbour] = true;
                        stack.push(neighbour);
                    }
                }
            }
            components.push(members);
        }
        components
    }

    fn background_regions(values: &[Address], height: usize, width: usize) -> Vec<Vec<usize>> {
        let mut visited = vec![false; values.len()];
        let mut regions = Vec::new();
        for start in 0..values.len() {
            if visited[start] || values[start].token().is_none_or(|token| token != [0]) {
                continue;
            }
            let mut stack = vec![start];
            let mut members = Vec::new();
            visited[start] = true;
            while let Some(index) = stack.pop() {
                members.push(index);
                let y = index / width;
                let x = index % width;
                for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let Some(ny) = y.checked_add_signed(dy) else {
                        continue;
                    };
                    let Some(nx) = x.checked_add_signed(dx) else {
                        continue;
                    };
                    if ny >= height || nx >= width {
                        continue;
                    }
                    let neighbour = ny * width + nx;
                    if !visited[neighbour]
                        && values[neighbour].token().is_some_and(|token| token == [0])
                    {
                        visited[neighbour] = true;
                        stack.push(neighbour);
                    }
                }
            }
            regions.push(members);
        }
        regions
    }

    fn component_shape_cue(&self, component: &[usize], width: usize) -> Address {
        let min_y = component.iter().map(|index| index / width).min().unwrap();
        let min_x = component.iter().map(|index| index % width).min().unwrap();
        let mut relative = component
            .iter()
            .map(|index| (index / width - min_y, index % width - min_x))
            .collect::<Vec<_>>();
        relative.sort_unstable();
        let mut at = self
            .component_rules
            .root()
            .provision(b"component-shape-cues");
        for (y, x) in relative {
            at = at
                .provision(&(y as u64).to_be_bytes())
                .provision(&(x as u64).to_be_bytes());
        }
        at
    }

    fn shifted_runs(
        values: &[Address],
        height: usize,
        width: usize,
        dy: isize,
        dx: isize,
        background: &Address,
    ) -> Vec<Address> {
        let mut result = vec![background.clone(); values.len()];
        if dx != 0 {
            for y in 0..height {
                let mut x = 0;
                while x < width {
                    if values[y * width + x]
                        .token()
                        .is_some_and(|token| token == [0])
                    {
                        x += 1;
                        continue;
                    }
                    let start = x;
                    while x + 1 < width
                        && values[y * width + x + 1]
                            .token()
                            .is_none_or(|token| token != [0])
                    {
                        x += 1;
                    }
                    let end = x;
                    let movable = start.checked_add_signed(dx).is_some_and(|v| v < width)
                        && end.checked_add_signed(dx).is_some_and(|v| v < width);
                    for source_x in start..=end {
                        let target_x = if movable {
                            source_x.checked_add_signed(dx).unwrap()
                        } else {
                            source_x
                        };
                        result[y * width + target_x] = values[y * width + source_x].clone();
                    }
                    x += 1;
                }
            }
        } else if dy != 0 {
            for x in 0..width {
                let mut y = 0;
                while y < height {
                    if values[y * width + x]
                        .token()
                        .is_some_and(|token| token == [0])
                    {
                        y += 1;
                        continue;
                    }
                    let start = y;
                    while y + 1 < height
                        && values[(y + 1) * width + x]
                            .token()
                            .is_none_or(|token| token != [0])
                    {
                        y += 1;
                    }
                    let end = y;
                    let movable = start.checked_add_signed(dy).is_some_and(|v| v < height)
                        && end.checked_add_signed(dy).is_some_and(|v| v < height);
                    for source_y in start..=end {
                        let target_y = if movable {
                            source_y.checked_add_signed(dy).unwrap()
                        } else {
                            source_y
                        };
                        result[target_y * width + x] = values[source_y * width + x].clone();
                    }
                    y += 1;
                }
            }
        }
        result
    }

    fn normalized_members(members: &[usize], width: usize) -> (usize, usize, Vec<(usize, usize)>) {
        let min_y = members.iter().map(|index| index / width).min().unwrap();
        let min_x = members.iter().map(|index| index % width).min().unwrap();
        let mut relative = members
            .iter()
            .map(|index| (index / width - min_y, index % width - min_x))
            .collect::<Vec<_>>();
        relative.sort_unstable();
        (min_y, min_x, relative)
    }

    fn color_groups(values: &[Address], height: usize, width: usize) -> Vec<(Address, Vec<usize>)> {
        let mut groups = Vec::<(Address, Vec<usize>)>::new();
        for component in Self::foreground_components(values, height, width) {
            let color = values[component[0]].clone();
            if let Some((_, members)) = groups.iter_mut().find(|(known, _)| known.same(&color)) {
                members.extend(component);
            } else {
                groups.push((color, component));
            }
        }
        groups
    }

    fn record_shape_transductions(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
        output: &[Address],
        evidence: &Address,
    ) -> Result<(), TransductionError> {
        let histogram = |values: &[Address]| {
            let mut counts = std::collections::BTreeMap::new();
            for value in values {
                *counts.entry(value.identity_key()).or_insert(0usize) += 1;
            }
            counts
        };
        if histogram(input) != histogram(output) {
            return Ok(());
        }
        let Some(clear_value) = input
            .iter()
            .find(|value| value.token().is_some_and(|token| token == [0]))
            .cloned()
        else {
            return Ok(());
        };
        for (color, input_members) in Self::color_groups(input, height, width) {
            let output_members = output
                .iter()
                .enumerate()
                .filter_map(|(index, value)| value.same(&color).then_some(index))
                .collect::<Vec<_>>();
            if output_members.is_empty() {
                continue;
            }
            let (input_y, input_x, input_shape) = Self::normalized_members(&input_members, width);
            let (output_y, output_x, output_shape) =
                Self::normalized_members(&output_members, width);
            let delta_y = output_y as isize - input_y as isize;
            let delta_x = output_x as isize - input_x as isize;
            let cue = self
                .component_shape_cue(&input_members, width)
                .provision(b"shape-transduction");
            if delta_y == 0 && delta_x == 0 && input_shape == output_shape {
                cue.provision(b"identity-evidence")
                    .provision(&evidence.identity_key())
                    .point_to(evidence);
                if cue.child(b"plan").is_some() {
                    cue.provision(b"ambiguous").terminate();
                }
                continue;
            }
            if cue.child(b"identity-evidence").is_some() {
                cue.provision(b"ambiguous").terminate();
                continue;
            }
            let mut dag = self
                .component_rules
                .root()
                .provision(b"shape-transduction-dags")
                .provision(&(delta_y as i64).to_be_bytes())
                .provision(&(delta_x as i64).to_be_bytes());
            for (y, x) in &output_shape {
                dag = dag
                    .provision(&(*y as u64).to_be_bytes())
                    .provision(&(*x as u64).to_be_bytes());
            }
            dag.provision(b"complete").terminate();
            dag.provision(b"delta-y")
                .provision(&(delta_y as i64).to_be_bytes())
                .terminate();
            dag.provision(b"delta-x")
                .provision(&(delta_x as i64).to_be_bytes())
                .terminate();
            for (index, (y, x)) in output_shape.iter().enumerate() {
                let member = dag.provision(b"members").provision(&index.to_be_bytes());
                member
                    .provision(b"y")
                    .provision(&(*y as u64).to_be_bytes())
                    .terminate();
                member
                    .provision(b"x")
                    .provision(&(*x as u64).to_be_bytes())
                    .terminate();
            }
            if let Some(existing) = cue.child(b"plan") {
                if !existing.follow()?.same(&dag) {
                    cue.provision(b"ambiguous").terminate();
                    continue;
                }
            } else {
                cue.provision(b"plan").point_to(&dag);
                let clear = self.program(&clear_value);
                cue.provision(b"clear-program").point_to(&clear);
                let port = self.neighbour_position(b"shape-transduction");
                let copy = self.copy_program(&port);
                cue.provision(b"copy-program").point_to(&copy);
            }
            cue.provision(b"evidence")
                .provision(&evidence.identity_key())
                .point_to(evidence);
        }
        Ok(())
    }

    fn record_translation_plan(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
        output: &[Address],
        evidence: &Address,
    ) -> Result<(), TransductionError> {
        let Some(background) = input
            .iter()
            .find(|value| value.token().is_some_and(|token| token == [0]))
            .cloned()
        else {
            return Ok(());
        };
        let mut matches = Vec::new();
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dy == 0 && dx == 0 {
                    continue;
                }
                let mut predicted = vec![background.clone(); input.len()];
                for (index, value) in input.iter().enumerate() {
                    if value.token().is_some_and(|token| token == [0]) {
                        continue;
                    }
                    let Some(y) = (index / width).checked_add_signed(dy) else {
                        continue;
                    };
                    let Some(x) = (index % width).checked_add_signed(dx) else {
                        continue;
                    };
                    if y < height && x < width {
                        predicted[y * width + x] = value.clone();
                    }
                }
                if predicted.iter().zip(output).all(|(a, b)| a.same(b)) {
                    matches.push((b"rigid".as_slice(), dy, dx));
                }
            }
        }
        let changed = input.iter().zip(output).filter(|(a, b)| !a.same(b)).count();
        if matches.is_empty() && changed > 2 {
            for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                let predicted = Self::shifted_runs(input, height, width, dy, dx, &background);
                if predicted.iter().zip(output).all(|(a, b)| a.same(b)) {
                    matches.push((b"runs".as_slice(), dy, dx));
                }
            }
        }
        if matches.len() != 1 {
            return Ok(());
        }
        let (mode, dy, dx) = matches[0];
        let plan = self
            .component_rules
            .root()
            .provision(b"component-translation-plan");
        if let Some(existing_dy) = plan.child(b"dy") {
            let mode_matches = plan
                .child(b"mode")
                .and_then(|node| node.child(mode))
                .is_some();
            let dy_matches = existing_dy.child(&(dy as i64).to_be_bytes()).is_some();
            let dx_matches = plan
                .child(b"dx")
                .and_then(|node| node.child(&(dx as i64).to_be_bytes()))
                .is_some();
            if !mode_matches || !dy_matches || !dx_matches {
                plan.provision(b"ambiguous").terminate();
                return Ok(());
            }
        } else {
            plan.provision(b"mode").provision(mode).terminate();
            plan.provision(b"dy")
                .provision(&(dy as i64).to_be_bytes())
                .terminate();
            plan.provision(b"dx")
                .provision(&(dx as i64).to_be_bytes())
                .terminate();
            plan.provision(b"background").point_to(&background);
            let clear = self.program(&background);
            plan.provision(b"clear-program").point_to(&clear);
            let copy = self.copy_program(&self.neighbour_position(b"component-translation"));
            plan.provision(b"copy-program").point_to(&copy);
        }
        plan.provision(b"evidence")
            .provision(&evidence.identity_key())
            .point_to(evidence);
        Ok(())
    }

    fn record_component_rule(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
        output: &[Address],
        evidence: &Address,
    ) -> Result<(), TransductionError> {
        let mut groups = Vec::<(Address, Vec<usize>)>::new();
        for component in Self::foreground_components(input, height, width) {
            let color = input[component[0]].clone();
            if let Some((_, members)) = groups.iter_mut().find(|(known, _)| known.same(&color)) {
                members.extend(component);
            } else {
                groups.push((color, component));
            }
        }
        if groups.len() < 2 {
            return Ok(());
        }
        let minimum_size = groups
            .iter()
            .map(|(_, members)| members.len())
            .min()
            .unwrap();
        if groups
            .iter()
            .filter(|(_, members)| members.len() == minimum_size)
            .count()
            != 1
        {
            return Ok(());
        }
        for selected_index in 0..groups.len() {
            if groups[selected_index].1.len() != minimum_size {
                continue;
            }
            let selected = &groups[selected_index].1;
            let selected_result = output[selected[0]].clone();
            if selected
                .iter()
                .any(|index| !output[*index].same(&selected_result))
            {
                continue;
            }
            let others = groups
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != selected_index)
                .flat_map(|(_, (_, members))| members.iter().copied())
                .collect::<Vec<_>>();
            let other_result = output[others[0]].clone();
            if others
                .iter()
                .any(|index| !output[*index].same(&other_result))
                || selected_result.same(&other_result)
            {
                continue;
            }
            let dag = self
                .component_rules
                .root()
                .provision(b"component-plan-dags")
                .provision(&selected_result.identity_key())
                .provision(&other_result.identity_key());
            if dag.child(b"selected-result").is_none() {
                dag.provision(b"selected-result").point_to(&selected_result);
                dag.provision(b"other-result").point_to(&other_result);
                let selected_program = self.program(&selected_result);
                dag.provision(b"selected-program")
                    .point_to(&selected_program);
                let other_program = self.program(&other_result);
                dag.provision(b"other-program").point_to(&other_program);
                dag.provision(b"write-selected-then-write-other")
                    .terminate();
            }
            let cue = self.component_shape_cue(selected, width);
            let rule = cue.provision(b"component-group-plan");
            if let Some(existing) = rule.child(b"plan") {
                if !existing.follow()?.same(&dag) {
                    rule.provision(b"ambiguous").terminate();
                    continue;
                }
            } else {
                rule.provision(b"plan").point_to(&dag);
            }
            rule.provision(b"evidence")
                .provision(&evidence.identity_key())
                .point_to(evidence);
        }
        Ok(())
    }

    fn record_enclosed_fill(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
        output: &[Address],
        evidence: &Address,
    ) -> Result<(), TransductionError> {
        let enclosed = Self::background_regions(input, height, width)
            .into_iter()
            .filter(|region| {
                region.iter().all(|index| {
                    let y = *index / width;
                    let x = *index % width;
                    y > 0 && y + 1 < height && x > 0 && x + 1 < width
                })
            })
            .collect::<Vec<_>>();
        if enclosed.is_empty() {
            return Ok(());
        }
        let result = output[enclosed[0][0]].clone();
        if result.token().is_some_and(|token| token == [0])
            || enclosed
                .iter()
                .flatten()
                .any(|index| !output[*index].same(&result))
        {
            return Ok(());
        }
        let cue = self
            .component_rules
            .root()
            .provision(b"enclosed-background-fill");
        if let Some(existing) = cue.child(b"result") {
            if !existing.follow()?.same(&result) {
                cue.provision(b"ambiguous").terminate();
                return Ok(());
            }
        } else {
            cue.provision(b"result").point_to(&result);
            let program = self.program(&result);
            cue.provision(b"program").point_to(&program);
        }
        cue.provision(b"evidence")
            .provision(&evidence.identity_key())
            .point_to(evidence);
        Ok(())
    }

    fn record_singleton_stencils(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
        output: &[Address],
        evidence: &Address,
    ) -> Result<(), TransductionError> {
        if input.iter().zip(output).any(|(before, after)| {
            !before.same(after)
                && (before.token().is_none_or(|token| token != [0])
                    || after.token().is_some_and(|token| token == [0]))
        }) {
            return Ok(());
        }
        for component in Self::foreground_components(input, height, width) {
            if component.len() != 1 {
                continue;
            }
            let index = component[0];
            let y = index / width;
            let x = index % width;
            let mut stencil = Vec::new();
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    if dy == 0 && dx == 0 {
                        continue;
                    }
                    let Some(ny) = y.checked_add_signed(dy) else {
                        continue;
                    };
                    let Some(nx) = x.checked_add_signed(dx) else {
                        continue;
                    };
                    if ny >= height || nx >= width {
                        continue;
                    }
                    let target = ny * width + nx;
                    if !input[target].same(&output[target]) {
                        stencil.push((dy, dx, output[target].clone()));
                    }
                }
            }
            if stencil.is_empty() {
                continue;
            }
            stencil.sort_by_key(|(dy, dx, _)| (*dy, *dx));
            let mut dag = self
                .component_rules
                .root()
                .provision(b"relative-stencil-dags");
            for (dy, dx, result) in &stencil {
                dag = dag
                    .provision(&(*dy as i64).to_be_bytes())
                    .provision(&(*dx as i64).to_be_bytes())
                    .provision(&result.identity_key());
                dag.provision(b"result").point_to(result);
                let program = self.program(result);
                dag.provision(b"program").point_to(&program);
            }
            dag.provision(b"complete").terminate();
            for (operation_index, (dy, dx, result)) in stencil.iter().enumerate() {
                let operation = dag
                    .provision(b"operations")
                    .provision(&operation_index.to_be_bytes());
                operation
                    .provision(b"dy")
                    .provision(&(*dy as i64).to_be_bytes())
                    .terminate();
                operation
                    .provision(b"dx")
                    .provision(&(*dx as i64).to_be_bytes())
                    .terminate();
                operation.provision(b"result").point_to(result);
                let program = self.program(result);
                operation.provision(b"program").point_to(&program);
            }
            let cue = self
                .component_rules
                .root()
                .provision(b"singleton-stencil-cues")
                .provision(&input[index].identity_key());
            cue.provision(b"color").point_to(&input[index]);
            if let Some(existing) = cue.child(b"stencil") {
                if !existing.follow()?.same(&dag) {
                    cue.provision(b"ambiguous").terminate();
                    continue;
                }
            } else {
                cue.provision(b"stencil").point_to(&dag);
            }
            cue.provision(b"evidence")
                .provision(&evidence.identity_key())
                .point_to(evidence);
        }
        Ok(())
    }

    fn component_positions(&self, values: &[Address], height: usize, width: usize) -> Vec<Address> {
        let mut positions = vec![self.position(); values.len()];
        let mut visited = vec![false; values.len()];
        let mut components = Vec::new();
        for start in 0..values.len() {
            if visited[start] {
                continue;
            }
            let color = values[start].clone();
            let background = color.token().is_some_and(|token| token == [0]);
            let mut stack = vec![start];
            let mut members = Vec::new();
            visited[start] = true;
            while let Some(index) = stack.pop() {
                members.push(index);
                let y = index / width;
                let x = index % width;
                for (dy, dx) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let Some(ny) = y.checked_add_signed(dy) else {
                        continue;
                    };
                    let Some(nx) = x.checked_add_signed(dx) else {
                        continue;
                    };
                    if ny >= height || nx >= width {
                        continue;
                    }
                    let neighbour = ny * width + nx;
                    if !visited[neighbour] && values[neighbour].same(&color) {
                        visited[neighbour] = true;
                        stack.push(neighbour);
                    }
                }
            }
            let min_y = members.iter().map(|index| index / width).min().unwrap();
            let max_y = members.iter().map(|index| index / width).max().unwrap();
            let min_x = members.iter().map(|index| index % width).min().unwrap();
            let max_x = members.iter().map(|index| index % width).max().unwrap();
            let touches_edge =
                min_y == 0 || min_x == 0 || max_y + 1 == height || max_x + 1 == width;
            components.push((
                color,
                background,
                members,
                min_y,
                max_y,
                min_x,
                max_x,
                touches_edge,
            ));
        }
        for (color, background, members, min_y, max_y, min_x, max_x, touches_edge) in &components {
            for index in members.iter().copied() {
                let y = index / width;
                let x = index % width;
                let degree = [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .filter(|(dy, dx)| {
                        y.checked_add_signed(*dy)
                            .zip(x.checked_add_signed(*dx))
                            .is_some_and(|(ny, nx)| {
                                ny < height && nx < width && values[ny * width + nx].same(color)
                            })
                    })
                    .count();
                positions[index] = if *background {
                    let edge_flags = [
                        u8::from(y == 0),
                        u8::from(y + 1 == height),
                        u8::from(x == 0),
                        u8::from(x + 1 == width),
                    ];
                    let inside_foreground_box = components.iter().any(
                        |(
                            _,
                            other_background,
                            _,
                            other_min_y,
                            other_max_y,
                            other_min_x,
                            other_max_x,
                            _,
                        )| {
                            !*other_background
                                && y >= *other_min_y
                                && y <= *other_max_y
                                && x >= *other_min_x
                                && x <= *other_max_x
                        },
                    );
                    let non_background =
                        |value: &Address| value.token().is_none_or(|token| token != [0]);
                    let horizontal_bracketed = (0..x)
                        .rev()
                        .any(|nx| non_background(&values[y * width + nx]))
                        && (x + 1..width).any(|nx| non_background(&values[y * width + nx]));
                    let vertical_bracketed = (0..y)
                        .rev()
                        .any(|ny| non_background(&values[ny * width + x]))
                        && (y + 1..height).any(|ny| non_background(&values[ny * width + x]));
                    let relations = [
                        u8::from(!*touches_edge),
                        u8::from(inside_foreground_box),
                        u8::from(horizontal_bracketed),
                        u8::from(vertical_bracketed),
                    ];
                    self.vocabulary.provision(&[
                        b"component-position",
                        b"background-region-cell",
                        &edge_flags,
                        &relations,
                        &(degree as u64).to_be_bytes(),
                    ])
                } else {
                    self.vocabulary.provision(&[
                        b"component-position",
                        b"foreground-component-cell",
                        &(members.len() as u64).to_be_bytes(),
                        &((*max_y - *min_y + 1) as u64).to_be_bytes(),
                        &((*max_x - *min_x + 1) as u64).to_be_bytes(),
                        &((y - *min_y) as u64).to_be_bytes(),
                        &((x - *min_x) as u64).to_be_bytes(),
                        &(degree as u64).to_be_bytes(),
                    ])
                };
            }
        }
        positions
    }

    fn grid_context_at(
        &self,
        values: &[Address],
        positions: &[Address],
        height: usize,
        width: usize,
        index: usize,
        episode: usize,
    ) -> Vec<AxisEvent> {
        let y = index / width;
        let x = index % width;
        [
            (b"north-west".as_slice(), -1isize, -1isize),
            (b"north".as_slice(), -1, 0),
            (b"north-east".as_slice(), -1, 1),
            (b"west".as_slice(), 0, -1),
            (b"east".as_slice(), 0, 1),
            (b"south-west".as_slice(), 1, -1),
            (b"south".as_slice(), 1, 0),
            (b"south-east".as_slice(), 1, 1),
        ]
        .into_iter()
        .filter_map(|(port, dy, dx)| {
            let ny = y.checked_add_signed(dy)?;
            let nx = x.checked_add_signed(dx)?;
            (ny < height && nx < width).then_some((port, ny * width + nx))
        })
        .map(|(port, neighbour)| AxisEvent {
            position: positions[index].clone(),
            neighbour_position: self.neighbour_position(port),
            color: values[index].clone(),
            neighbour_color: values[neighbour].clone(),
            focal: self.occurrence(episode, index, b"focal"),
            neighbour: self.occurrence(episode, neighbour, b"neighbour"),
        })
        .collect()
    }

    fn relative_port(&self, width: usize, source: usize, destination: usize) -> Option<Address> {
        let sy = source / width;
        let sx = source % width;
        let dy = destination / width;
        let dx = destination % width;
        let name = match (dy as isize - sy as isize, dx as isize - sx as isize) {
            (-1, -1) => b"north-west".as_slice(),
            (-1, 0) => b"north".as_slice(),
            (-1, 1) => b"north-east".as_slice(),
            (0, -1) => b"west".as_slice(),
            (0, 1) => b"east".as_slice(),
            (1, -1) => b"south-west".as_slice(),
            (1, 0) => b"south".as_slice(),
            (1, 1) => b"south-east".as_slice(),
            _ => return None,
        };
        Some(self.neighbour_position(name))
    }

    fn grid_context(
        &self,
        values: &[Address],
        height: usize,
        width: usize,
        index: usize,
        episode: usize,
    ) -> Vec<AxisEvent> {
        let positions = self.component_positions(values, height, width);
        self.grid_context_at(values, &positions, height, width, index, episode)
    }

    fn grid_graph(
        &self,
        values: &[Address],
        height: usize,
        width: usize,
        episode: usize,
    ) -> Vec<AxisEvent> {
        let positions = self.component_positions(values, height, width);
        (0..values.len())
            .flat_map(|index| {
                self.grid_context_at(values, &positions, height, width, index, episode)
            })
            .collect()
    }

    fn program(&mut self, result: &Address) -> Address {
        let root = self
            .machine
            .states
            .root()
            .provision(b"episode-programs")
            .provision(&result.identity_key());
        if let Some(entry) = root.child(b"entry") {
            return entry.follow().unwrap();
        }
        let result_slot = root.provision(b"result-value");
        result_slot.point_to(result);
        let operation = root.provision(b"write-result");
        let placeholder = root.provision(b"unbound-target");
        let operation_next = root.provision(b"operation-next");
        operation_next.point_to(&self.halt);
        install(&operation, &result_slot, &placeholder, &operation_next);
        let binder = root.provision(b"bind-target");
        let binder_next = root.provision(b"binder-next");
        binder_next.point_to(&operation);
        install(
            &binder,
            &self.frame_target,
            &target_field(&operation).unwrap(),
            &binder_next,
        );
        root.provision(b"entry").point_to(&binder);
        binder
    }

    fn copy_program(&mut self, port: &Address) -> Address {
        let root = self
            .machine
            .states
            .root()
            .provision(b"episode-copy-programs")
            .provision(&port.identity_key());
        if let Some(entry) = root.child(b"entry") {
            return entry.follow().unwrap();
        }
        let operation = root.provision(b"copy-neighbour-value");
        let placeholder = root.provision(b"unbound-target");
        let operation_next = root.provision(b"operation-next");
        operation_next.point_to(&self.halt);
        install(
            &operation,
            &self.frame_source,
            &placeholder,
            &operation_next,
        );
        let binder = root.provision(b"bind-target");
        let binder_next = root.provision(b"binder-next");
        binder_next.point_to(&operation);
        install(
            &binder,
            &self.frame_target,
            &target_field(&operation).unwrap(),
            &binder_next,
        );
        binder.provision(b"source-port").point_to(port);
        root.provision(b"entry").point_to(&binder);
        binder
    }

    fn move_program(&mut self, port: &Address, background: &Address) -> Address {
        let root = self
            .machine
            .states
            .root()
            .provision(b"episode-move-programs")
            .provision(&port.identity_key())
            .provision(&background.identity_key());
        if let Some(entry) = root.child(b"entry") {
            return entry.follow().unwrap();
        }
        let background_slot = root.provision(b"background-value");
        background_slot.point_to(background);
        let write_destination = root.provision(b"write-destination");
        let destination_placeholder = root.provision(b"unbound-destination");
        let destination_next = root.provision(b"destination-next");
        destination_next.point_to(&self.halt);
        install(
            &write_destination,
            &self.frame_source,
            &destination_placeholder,
            &destination_next,
        );
        let bind_destination = root.provision(b"bind-destination");
        let bind_destination_next = root.provision(b"bind-destination-next");
        bind_destination_next.point_to(&write_destination);
        install(
            &bind_destination,
            &self.frame_destination,
            &target_field(&write_destination).unwrap(),
            &bind_destination_next,
        );
        let clear_source = root.provision(b"clear-source");
        let source_placeholder = root.provision(b"unbound-source");
        let clear_next = root.provision(b"clear-next");
        clear_next.point_to(&bind_destination);
        install(
            &clear_source,
            &background_slot,
            &source_placeholder,
            &clear_next,
        );
        let bind_source = root.provision(b"bind-source");
        let bind_source_next = root.provision(b"bind-source-next");
        bind_source_next.point_to(&clear_source);
        install(
            &bind_source,
            &self.frame_target,
            &target_field(&clear_source).unwrap(),
            &bind_source_next,
        );
        bind_source.provision(b"move-port").point_to(port);
        root.provision(b"entry").point_to(&bind_source);
        bind_source
    }

    fn identity_dag(
        &mut self,
        axis: &[u8],
        evidence: &Address,
    ) -> Result<Address, TransductionError> {
        let operation = self
            .vocabulary
            .provision(&[b"operations", b"identity", axis]);
        let implementation = self
            .vocabulary
            .provision(&[b"implementations", b"identity"]);
        Ok(self
            .dags
            .intern(&operation, &[], &implementation, evidence)?)
    }

    fn color_dag(
        &mut self,
        result: &Address,
        evidence: &Address,
    ) -> Result<(Address, Address), TransductionError> {
        let literal_operation = self
            .vocabulary
            .provision(&[b"operations", b"literal-reference"]);
        let literal_implementation = self
            .vocabulary
            .provision(&[b"literal-implementations"])
            .provision(&result.identity_key());
        literal_implementation.provision(b"value").point_to(result);
        let literal =
            self.dags
                .intern(&literal_operation, &[], &literal_implementation, evidence)?;
        let role = self
            .vocabulary
            .provision(&[b"roles", b"focal-color-result"]);
        let replace = self
            .vocabulary
            .provision(&[b"operations", b"replace-reference"]);
        let entry = self.program(result);
        let dag = self.dags.intern(
            &replace,
            &[DagInput {
                role,
                construction: literal,
            }],
            &entry,
            evidence,
        )?;
        Ok((dag, entry))
    }

    fn copied_color_dag(
        &mut self,
        port: &Address,
        evidence: &Address,
    ) -> Result<(Address, Address), TransductionError> {
        let source = self
            .vocabulary
            .provision(&[b"operations", b"bound-neighbour-reference"])
            .provision(&port.identity_key());
        source.provision(b"port").point_to(port);
        let source_implementation = self
            .vocabulary
            .provision(&[b"implementations", b"bound-neighbour-reference"]);
        let source_dag = self
            .dags
            .intern(&source, &[], &source_implementation, evidence)?;
        let role = self
            .vocabulary
            .provision(&[b"roles", b"focal-color-result"]);
        let copy = self
            .vocabulary
            .provision(&[b"operations", b"copy-neighbour-color"]);
        let entry = self.copy_program(port);
        let dag = self.dags.intern(
            &copy,
            &[DagInput {
                role,
                construction: source_dag,
            }],
            &entry,
            evidence,
        )?;
        Ok((dag, entry))
    }

    fn record_change(
        &mut self,
        context: &[AxisEvent],
        result: &Address,
        evidence: &Address,
    ) -> Result<(), TransductionError> {
        let position = self.identity_dag(b"position", evidence)?;
        let neighbour_position = self.identity_dag(b"neighbour-position", evidence)?;
        let neighbour_color = self.identity_dag(b"neighbour-color", evidence)?;
        let matching = context
            .iter()
            .filter(|event| event.neighbour_color.same(result))
            .collect::<Vec<_>>();
        let (color, entry) = if let [source] = matching.as_slice() {
            self.copied_color_dag(&source.neighbour_position, evidence)?
        } else {
            self.color_dag(result, evidence)?
        };
        let constructions = AxisConstructions {
            position,
            neighbour_position,
            color,
            neighbour_color,
        };
        let endpoint = self.contexts.teach(context, &constructions, evidence)?;
        self.contexts
            .record_implementation(&endpoint, &entry, evidence)?;
        Ok(())
    }

    fn record_move(
        &mut self,
        context: &[AxisEvent],
        port: &Address,
        background: &Address,
        evidence: &Address,
    ) -> Result<(), TransductionError> {
        let direction_operation = self
            .vocabulary
            .provision(&[b"operations", b"bound-neighbour-position"])
            .provision(&port.identity_key());
        direction_operation.provision(b"port").point_to(port);
        let direction_implementation = self
            .vocabulary
            .provision(&[b"implementations", b"bound-neighbour-position"]);
        let direction = self.dags.intern(
            &direction_operation,
            &[],
            &direction_implementation,
            evidence,
        )?;
        let role = self.vocabulary.provision(&[b"roles", b"position-result"]);
        let operation = self
            .vocabulary
            .provision(&[b"operations", b"redirect-position"]);
        let entry = self.move_program(port, background);
        let position = self.dags.intern(
            &operation,
            &[DagInput {
                role,
                construction: direction,
            }],
            &entry,
            evidence,
        )?;
        let constructions = AxisConstructions {
            position,
            neighbour_position: self.identity_dag(b"neighbour-position", evidence)?,
            color: self.identity_dag(b"color", evidence)?,
            neighbour_color: self.identity_dag(b"neighbour-color", evidence)?,
        };
        let endpoint = self.contexts.teach(context, &constructions, evidence)?;
        self.contexts
            .record_implementation(&endpoint, &entry, evidence)?;
        Ok(())
    }

    /// Compare paired namespaced graph tries, derive axis DAGs for each changed focal occurrence,
    /// and attach the native episode implementation to its input context.
    pub fn teach(
        &mut self,
        input: &[Address],
        output: &[Address],
    ) -> Result<usize, TransductionError> {
        if input.len() != output.len() || input.len() < 3 {
            return Err(TransductionError::Shape);
        }
        let episode = self.serial;
        self.serial += 1;
        let evidence = self
            .occurrences
            .provision(&[b"demonstrations", &episode.to_be_bytes()]);
        let input_graph = self.graph(input, episode);
        let output_graph = self.graph(output, episode);
        let encoded_input = self.states.encode(b"input", &input_graph, &evidence)?;
        let encoded_output = self.states.encode(b"output", &output_graph, &evidence)?;
        let deltas = PairedStates::compare(&encoded_input, &encoded_output)?;
        self.masks.derive(&deltas, &evidence)?;
        let mut learned = 0;
        for index in 1..input.len() - 1 {
            if input[index].same(&output[index]) {
                continue;
            }
            let offset = (index - 1) * 2;
            if !deltas[offset].color_changed || !deltas[offset + 1].color_changed {
                return Err(TransductionError::UnsupportedBoundaryChange);
            }
            let context = self.context(input, index, episode);
            self.record_change(&context, &output[index], &evidence)?;
            learned += 1;
        }
        if learned == 0 {
            return Err(TransductionError::NoChange);
        }
        Ok(learned)
    }

    pub fn teach_grid(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
        output: &[Address],
    ) -> Result<usize, TransductionError> {
        if height == 0
            || width == 0
            || height.checked_mul(width) != Some(input.len())
            || input.len() != output.len()
        {
            return Err(TransductionError::Shape);
        }
        let episode = self.serial;
        self.serial += 1;
        let evidence = self
            .occurrences
            .provision(&[b"grid-demonstrations", &episode.to_be_bytes()]);
        let encoded_input = self.states.encode(
            b"input",
            &self.grid_graph(input, height, width, episode),
            &evidence,
        )?;
        let encoded_output = self.states.encode(
            b"output",
            &self.grid_graph(output, height, width, episode),
            &evidence,
        )?;
        let deltas = PairedStates::compare(&encoded_input, &encoded_output)?;
        self.masks.derive(&deltas, &evidence)?;
        self.record_shape_transductions(height, width, input, output, &evidence)?;
        self.record_translation_plan(height, width, input, output, &evidence)?;
        self.record_component_rule(height, width, input, output, &evidence)?;
        self.record_enclosed_fill(height, width, input, output, &evidence)?;
        self.record_singleton_stencils(height, width, input, output, &evidence)?;
        let changed = (0..input.len())
            .filter(|&index| !input[index].same(&output[index]))
            .collect::<Vec<_>>();
        if let [source, destination] = changed.as_slice()
            && output[*source].same(&input[*destination])
            && output[*destination].same(&input[*source])
            && let Some(port) = self.relative_port(width, *source, *destination)
        {
            let context = self.grid_context(input, height, width, *source, episode);
            self.record_move(&context, &port, &input[*destination], &evidence)?;
            return Ok(1);
        }
        let mut learned = 0;
        for index in 0..input.len() {
            if input[index].same(&output[index]) {
                continue;
            }
            let context = self.grid_context(input, height, width, index, episode);
            self.record_change(&context, &output[index], &evidence)?;
            learned += 1;
        }
        if learned == 0 {
            return Err(TransductionError::NoChange);
        }
        Ok(learned)
    }

    fn source_binding(
        entry: &Address,
        context: &[AxisEvent],
    ) -> Result<Option<Address>, TransductionError> {
        let Some(port) = entry.child(b"source-port") else {
            return Ok(None);
        };
        let port = port.follow()?;
        let mut matches = context
            .iter()
            .filter(|event| event.neighbour_position.same(&port));
        let source = matches
            .next()
            .ok_or(TransductionError::Incomplete)?
            .neighbour_color
            .clone();
        if matches.next().is_some() {
            return Err(TransductionError::Incomplete);
        }
        Ok(Some(source))
    }

    fn move_destination(height: usize, width: usize, source: usize, port: &[u8]) -> Option<usize> {
        let (dy, dx) = match port {
            b"north-west" => (-1, -1),
            b"north" => (-1, 0),
            b"north-east" => (-1, 1),
            b"west" => (0, -1),
            b"east" => (0, 1),
            b"south-west" => (1, -1),
            b"south" => (1, 0),
            b"south-east" => (1, 1),
            _ => return None,
        };
        let y = (source / width).checked_add_signed(dy)?;
        let x = (source % width).checked_add_signed(dx)?;
        (y < height && x < width).then_some(y * width + x)
    }

    fn execute_move(
        &mut self,
        entry: &Address,
        source_value: &Address,
        source_target: &Address,
        destination_target: &Address,
    ) -> Result<(), TransductionError> {
        self.frame_source.point_to(source_value);
        self.frame_target.point_to(source_target);
        self.frame_destination.point_to(destination_target);
        self.machine.enter(entry);
        for _ in 0..4 {
            if self.machine.cycle()? != Outcome::Advanced {
                return Err(TransductionError::Incomplete);
            }
        }
        if self.machine.cycle()? != Outcome::Halted {
            return Err(TransductionError::Incomplete);
        }
        self.applications += 1;
        self.copy_cycles += self.machine.cycles;
        Ok(())
    }

    fn execute(
        &mut self,
        entry: &Address,
        target: &Address,
        source: Option<&Address>,
    ) -> Result<(), TransductionError> {
        self.frame_target.point_to(target);
        if let Some(source) = source {
            self.frame_source.point_to(source);
        }
        self.machine.enter(entry);
        match self.machine.cycle()? {
            Outcome::Advanced => {}
            Outcome::Halted => return Err(TransductionError::Incomplete),
        }
        match self.machine.cycle()? {
            Outcome::Advanced => {}
            Outcome::Halted => return Err(TransductionError::Incomplete),
        }
        if self.machine.cycle()? != Outcome::Halted {
            return Err(TransductionError::Incomplete);
        }
        self.applications += 1;
        self.copy_cycles += self.machine.cycles;
        Ok(())
    }

    /// Apply learned constructions to a held-out teaching input and require its derived structural
    /// change mask to be the same native mask as the demonstrated input/output change.
    pub fn validate(
        &mut self,
        input: &[Address],
        expected: &[Address],
    ) -> Result<(), TransductionError> {
        if input.len() != expected.len() || input.len() < 3 {
            return Err(TransductionError::Shape);
        }
        let episode = self.serial;
        self.serial += 1;
        let evidence = self
            .occurrences
            .provision(&[b"validations", &episode.to_be_bytes()]);
        let encoded_input = self
            .states
            .encode(b"input", &self.graph(input, episode), &evidence)?;
        let encoded_expected =
            self.states
                .encode(b"output", &self.graph(expected, episode), &evidence)?;
        let expected_mask = self.masks.derive(
            &PairedStates::compare(&encoded_input, &encoded_expected)?,
            &evidence,
        )?;
        let actual = self.transduce(input)?;
        let actual_evidence = evidence.provision(b"derived-result");
        let encoded_actual =
            self.states
                .encode(b"output", &self.graph(&actual, episode), &actual_evidence)?;
        let actual_mask = self.masks.derive(
            &PairedStates::compare(&encoded_input, &encoded_actual)?,
            &actual_evidence,
        )?;
        if !ChangeMasks::matches(&expected_mask, &actual_mask) {
            return Err(TransductionError::ValidationMismatch);
        }
        Ok(())
    }

    pub fn validate_grid(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
        expected: &[Address],
    ) -> Result<(), TransductionError> {
        if height == 0
            || width == 0
            || height.checked_mul(width) != Some(input.len())
            || input.len() != expected.len()
        {
            return Err(TransductionError::Shape);
        }
        let episode = self.serial;
        self.serial += 1;
        let evidence = self
            .occurrences
            .provision(&[b"grid-validations", &episode.to_be_bytes()]);
        let encoded_input = self.states.encode(
            b"input",
            &self.grid_graph(input, height, width, episode),
            &evidence,
        )?;
        let encoded_expected = self.states.encode(
            b"output",
            &self.grid_graph(expected, height, width, episode),
            &evidence,
        )?;
        let expected_mask = self.masks.derive(
            &PairedStates::compare(&encoded_input, &encoded_expected)?,
            &evidence,
        )?;
        let actual = self.transduce_grid(height, width, input)?;
        let actual_evidence = evidence.provision(b"derived-result");
        let encoded_actual = self.states.encode(
            b"output",
            &self.grid_graph(&actual, height, width, episode),
            &actual_evidence,
        )?;
        let actual_mask = self.masks.derive(
            &PairedStates::compare(&encoded_input, &encoded_actual)?,
            &actual_evidence,
        )?;
        if !ChangeMasks::matches(&expected_mask, &actual_mask) {
            return Err(TransductionError::ValidationMismatch);
        }
        Ok(())
    }

    pub fn applicable_grid(
        &self,
        height: usize,
        width: usize,
        input: &[Address],
    ) -> Result<bool, TransductionError> {
        if height == 0 || width == 0 || height.checked_mul(width) != Some(input.len()) {
            return Err(TransductionError::Shape);
        }
        for index in 0..input.len() {
            let context = self.grid_context(input, height, width, index, self.serial);
            if self
                .contexts
                .resolve(&context)
                .ok()
                .and_then(|resolved| resolved.implementation)
                .is_some()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn apply_enclosed_fill(
        &mut self,
        height: usize,
        width: usize,
        values: &[Address],
        cells: &[Address],
    ) -> Result<bool, TransductionError> {
        let Some(cue) = self
            .component_rules
            .root()
            .child(b"enclosed-background-fill")
        else {
            return Ok(false);
        };
        if cue.child(b"ambiguous").is_some() {
            return Ok(false);
        }
        let program = cue
            .child(b"program")
            .ok_or(TransductionError::Incomplete)?
            .follow()?;
        let mut applied = false;
        for region in Self::background_regions(values, height, width) {
            if region.iter().any(|index| {
                let y = *index / width;
                let x = *index % width;
                y == 0 || y + 1 == height || x == 0 || x + 1 == width
            }) {
                continue;
            }
            for index in region {
                self.execute(&program, &cells[index], None)?;
                applied = true;
            }
        }
        Ok(applied)
    }

    fn apply_singleton_stencils(
        &mut self,
        height: usize,
        width: usize,
        values: &[Address],
        cells: &[Address],
    ) -> Result<bool, TransductionError> {
        let mut pending = std::collections::BTreeMap::<usize, Address>::new();
        for component in Self::foreground_components(values, height, width) {
            if component.len() != 1 {
                continue;
            }
            let source = component[0];
            let Some(cue) = self
                .component_rules
                .root()
                .child(b"singleton-stencil-cues")
                .and_then(|root| root.child(&values[source].identity_key()))
            else {
                continue;
            };
            if cue.child(b"ambiguous").is_some() {
                continue;
            }
            let dag = cue
                .child(b"stencil")
                .ok_or(TransductionError::Incomplete)?
                .follow()?;
            let operations = dag
                .child(b"operations")
                .ok_or(TransductionError::Incomplete)?;
            for operation in operations.inspect_children() {
                let decode = |name: &[u8]| -> Result<isize, TransductionError> {
                    let token = operation
                        .child(name)
                        .and_then(|node| node.inspect_children().into_iter().next())
                        .and_then(|node| node.token().map(<[u8]>::to_vec))
                        .ok_or(TransductionError::Incomplete)?;
                    let bytes: [u8; 8] = token
                        .try_into()
                        .map_err(|_| TransductionError::Incomplete)?;
                    Ok(i64::from_be_bytes(bytes) as isize)
                };
                let dy = decode(b"dy")?;
                let dx = decode(b"dx")?;
                let y = (source / width)
                    .checked_add_signed(dy)
                    .ok_or(TransductionError::Incomplete)?;
                let x = (source % width)
                    .checked_add_signed(dx)
                    .ok_or(TransductionError::Incomplete)?;
                if y >= height || x >= width {
                    continue;
                }
                let destination = y * width + x;
                let program = operation
                    .child(b"program")
                    .ok_or(TransductionError::Incomplete)?
                    .follow()?;
                if let Some(existing) = pending.insert(destination, program.clone())
                    && !existing.same(&program)
                {
                    return Err(TransductionError::Incomplete);
                }
            }
        }
        for (destination, program) in &pending {
            self.execute(program, &cells[*destination], None)?;
        }
        Ok(!pending.is_empty())
    }

    fn apply_shape_transductions(
        &mut self,
        height: usize,
        width: usize,
        values: &[Address],
        cells: &[Address],
    ) -> Result<bool, TransductionError> {
        let mut clears = std::collections::BTreeMap::<usize, Address>::new();
        let mut writes = std::collections::BTreeMap::<usize, (Address, Address)>::new();
        for (color, members) in Self::color_groups(values, height, width) {
            let (anchor_y, anchor_x, _) = Self::normalized_members(&members, width);
            let cue = self
                .component_shape_cue(&members, width)
                .provision(b"shape-transduction");
            if cue.child(b"ambiguous").is_some() {
                continue;
            }
            let Some(plan) = cue.child(b"plan") else {
                continue;
            };
            let plan = plan.follow()?;
            let decode_signed = |name: &[u8]| -> Result<isize, TransductionError> {
                let token = plan
                    .child(name)
                    .and_then(|node| node.inspect_children().into_iter().next())
                    .and_then(|node| node.token().map(<[u8]>::to_vec))
                    .ok_or(TransductionError::Incomplete)?;
                let bytes: [u8; 8] = token
                    .try_into()
                    .map_err(|_| TransductionError::Incomplete)?;
                Ok(i64::from_be_bytes(bytes) as isize)
            };
            let delta_y = decode_signed(b"delta-y")?;
            let delta_x = decode_signed(b"delta-x")?;
            let clear = cue
                .child(b"clear-program")
                .ok_or(TransductionError::Incomplete)?
                .follow()?;
            let copy = cue
                .child(b"copy-program")
                .ok_or(TransductionError::Incomplete)?
                .follow()?;
            for member in members {
                clears.insert(member, clear.clone());
            }
            let plan_members = plan
                .child(b"members")
                .ok_or(TransductionError::Incomplete)?;
            for member in plan_members.inspect_children() {
                let decode_unsigned = |name: &[u8]| -> Result<usize, TransductionError> {
                    let token = member
                        .child(name)
                        .and_then(|node| node.inspect_children().into_iter().next())
                        .and_then(|node| node.token().map(<[u8]>::to_vec))
                        .ok_or(TransductionError::Incomplete)?;
                    let bytes: [u8; 8] = token
                        .try_into()
                        .map_err(|_| TransductionError::Incomplete)?;
                    Ok(u64::from_be_bytes(bytes) as usize)
                };
                let Some(y) = anchor_y
                    .checked_add_signed(delta_y)
                    .and_then(|base| base.checked_add(decode_unsigned(b"y").ok()?))
                else {
                    return Err(TransductionError::Incomplete);
                };
                let Some(x) = anchor_x
                    .checked_add_signed(delta_x)
                    .and_then(|base| base.checked_add(decode_unsigned(b"x").ok()?))
                else {
                    return Err(TransductionError::Incomplete);
                };
                if y >= height || x >= width {
                    return Err(TransductionError::Incomplete);
                }
                let destination = y * width + x;
                if let Some((known, _)) = writes.get(&destination)
                    && !known.same(&color)
                {
                    return Err(TransductionError::Incomplete);
                }
                writes.insert(destination, (color.clone(), copy.clone()));
            }
        }
        for (index, program) in &clears {
            self.execute(program, &cells[*index], None)?;
        }
        for (index, (value, program)) in &writes {
            self.execute(program, &cells[*index], Some(value))?;
        }
        Ok(!clears.is_empty())
    }

    fn apply_translation_plan(
        &mut self,
        height: usize,
        width: usize,
        values: &[Address],
        cells: &[Address],
    ) -> Result<bool, TransductionError> {
        let Some(plan) = self
            .component_rules
            .root()
            .child(b"component-translation-plan")
        else {
            return Ok(false);
        };
        if plan.child(b"ambiguous").is_some() {
            return Ok(false);
        }
        let decode = |name: &[u8]| -> Result<isize, TransductionError> {
            let token = plan
                .child(name)
                .and_then(|node| node.inspect_children().into_iter().next())
                .and_then(|node| node.token().map(<[u8]>::to_vec))
                .ok_or(TransductionError::Incomplete)?;
            let bytes: [u8; 8] = token
                .try_into()
                .map_err(|_| TransductionError::Incomplete)?;
            Ok(i64::from_be_bytes(bytes) as isize)
        };
        let dy = decode(b"dy")?;
        let dx = decode(b"dx")?;
        let clear = plan
            .child(b"clear-program")
            .ok_or(TransductionError::Incomplete)?
            .follow()?;
        let copy = plan
            .child(b"copy-program")
            .ok_or(TransductionError::Incomplete)?
            .follow()?;
        let mode = plan
            .child(b"mode")
            .and_then(|node| node.inspect_children().into_iter().next())
            .and_then(|node| node.token().map(<[u8]>::to_vec))
            .ok_or(TransductionError::Incomplete)?;
        let background = plan
            .child(b"background")
            .ok_or(TransductionError::Incomplete)?
            .follow()?;
        let desired = if mode == b"runs" {
            Self::shifted_runs(values, height, width, dy, dx, &background)
        } else if mode == b"rigid" {
            let mut result = vec![background.clone(); values.len()];
            for (source, value) in values.iter().enumerate() {
                if value.token().is_some_and(|token| token == [0]) {
                    continue;
                }
                let Some(y) = (source / width).checked_add_signed(dy) else {
                    continue;
                };
                let Some(x) = (source % width).checked_add_signed(dx) else {
                    continue;
                };
                if y < height && x < width {
                    result[y * width + x] = value.clone();
                }
            }
            result
        } else {
            return Err(TransductionError::Incomplete);
        };
        for (index, value) in values.iter().enumerate() {
            if value.token().is_none_or(|token| token != [0]) {
                self.execute(&clear, &cells[index], None)?;
            }
        }
        for (destination, value) in desired.iter().enumerate() {
            if value.token().is_none_or(|token| token != [0]) {
                self.execute(&copy, &cells[destination], Some(value))?;
            }
        }
        Ok(true)
    }

    fn apply_component_rule(
        &mut self,
        height: usize,
        width: usize,
        values: &[Address],
        cells: &[Address],
    ) -> Result<bool, TransductionError> {
        let components = Self::foreground_components(values, height, width);
        let mut groups = Vec::<(Address, Vec<usize>)>::new();
        for component in components {
            let color = values[component[0]].clone();
            if let Some((_, members)) = groups.iter_mut().find(|(known, _)| known.same(&color)) {
                members.extend(component);
            } else {
                groups.push((color, component));
            }
        }
        let mut selected = None;
        for (_, component) in groups {
            let cue = self.component_shape_cue(&component, width);
            let Some(rule) = cue.child(b"component-group-plan") else {
                continue;
            };
            if rule.child(b"ambiguous").is_some() {
                continue;
            }
            if selected.is_some() {
                return Err(TransductionError::Incomplete);
            }
            selected = Some((component, rule));
        }
        let Some((selected_members, rule)) = selected else {
            return Ok(false);
        };
        let plan = rule
            .child(b"plan")
            .ok_or(TransductionError::Incomplete)?
            .follow()?;
        let selected_program = plan
            .child(b"selected-program")
            .ok_or(TransductionError::Incomplete)?
            .follow()?;
        let other_program = plan
            .child(b"other-program")
            .ok_or(TransductionError::Incomplete)?
            .follow()?;
        let selected_members = selected_members
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        for (index, cell) in cells.iter().enumerate() {
            if selected_members.contains(&index) {
                self.execute(&selected_program, cell, None)?;
            } else if values[index].token().is_none_or(|token| token != [0]) {
                self.execute(&other_program, cell, None)?;
            }
        }
        Ok(true)
    }

    pub fn validate_grid_persistent(
        &mut self,
        disk: &DiskArc,
        episode: &[u8],
        evidence: &DiskAddress,
        shape: (usize, usize),
        input: &[Address],
        expected: &[Address],
    ) -> Result<(), TransductionError> {
        let (height, width) = shape;
        if height == 0
            || width == 0
            || height.checked_mul(width) != Some(input.len())
            || input.len() != expected.len()
        {
            return Err(TransductionError::Shape);
        }
        let bytes = |values: &[Address]| {
            values
                .chunks(width)
                .map(|row| {
                    row.iter()
                        .map(|value| {
                            let token = value.token().ok_or(TransductionError::Incomplete)?;
                            if token.len() != 1 {
                                return Err(TransductionError::Incomplete);
                            }
                            Ok(token[0])
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()
        };
        let input_grid = bytes(input)?;
        let expected_grid = bytes(expected)?;
        let mut expected_episode = episode.to_vec();
        expected_episode.extend_from_slice(b"-expected");
        disk.encode(b"input", &expected_episode, &input_grid, evidence)?;
        disk.encode(b"output", &expected_episode, &expected_grid, evidence)?;
        let expected_mask =
            disk.derive_masks(&expected_episode, &input_grid, &expected_grid, evidence)?;
        let actual =
            self.transduce_grid_persistent(disk, episode, evidence, height, width, input)?;
        let actual_mask = disk.derive_masks(episode, &input_grid, &actual, evidence)?;
        if !disk.masks_equal(&expected_mask, &actual_mask)? {
            return Err(TransductionError::ValidationMismatch);
        }
        Ok(())
    }

    /// Run the ARC construction path, then materialize every predicted cell through the
    /// persistent copy machine. This is the migration bridge: plan induction remains resident,
    /// while the outward grid is read back from disk-executed effects.
    pub fn transduce_grid_persistent(
        &mut self,
        disk: &DiskArc,
        episode: &[u8],
        evidence: &DiskAddress,
        height: usize,
        width: usize,
        input: &[Address],
    ) -> Result<Vec<Vec<u8>>, TransductionError> {
        if height == 0 || width == 0 || height.checked_mul(width) != Some(input.len()) {
            return Err(TransductionError::Shape);
        }
        let input_grid = input
            .chunks(width)
            .map(|row| {
                row.iter()
                    .map(|value| {
                        let token = value.token().ok_or(TransductionError::Incomplete)?;
                        if token.len() != 1 {
                            return Err(TransductionError::Incomplete);
                        }
                        Ok(token[0])
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        disk.encode(b"input", episode, &input_grid, evidence)?;
        let prediction = self.transduce_grid(height, width, input)?;
        let prediction = prediction
            .chunks(width)
            .map(|row| {
                row.iter()
                    .map(|value| {
                        let token = value.token().ok_or(TransductionError::Incomplete)?;
                        if token.len() != 1 {
                            return Err(TransductionError::Incomplete);
                        }
                        Ok(token[0])
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let output = disk.execute_prediction(episode, &prediction, evidence)?;
        disk.encode(b"output", episode, &output, evidence)?;
        disk.derive_masks(episode, &input_grid, &output, evidence)?;
        Ok(output)
    }

    fn attempt_grid_candidates(
        &mut self,
        shape: (usize, usize),
        index: usize,
        run: usize,
        snapshot: &[Address],
        context: &[AxisEvent],
        candidates: &[Address],
    ) -> Result<(), TransductionError> {
        let (height, width) = shape;
        let attempts = self
            .machine
            .output
            .root()
            .provision(b"ambiguous-grid-attempts")
            .provision(&run.to_be_bytes())
            .provision(&index.to_be_bytes());
        let mut first_error = None;
        for candidate in candidates {
            let attempt = attempts.provision(&candidate.identity_key());
            attempt.provision(b"construction").point_to(candidate);
            let result = (|| {
                if let Some(port) = candidate.child(b"move-port") {
                    let port = port.follow()?;
                    let destination = Self::move_destination(
                        height,
                        width,
                        index,
                        port.token().ok_or(TransductionError::Incomplete)?,
                    )
                    .ok_or(TransductionError::Incomplete)?;
                    let source_cell = attempt.provision(b"source-effect");
                    source_cell.point_to(&snapshot[index]);
                    let destination_cell = attempt.provision(b"destination-effect");
                    destination_cell.point_to(&snapshot[destination]);
                    self.execute_move(
                        candidate,
                        &snapshot[index],
                        &source_cell,
                        &destination_cell,
                    )?;
                    attempt
                        .provision(b"source-after")
                        .point_to(&source_cell.follow()?);
                    attempt
                        .provision(b"destination-after")
                        .point_to(&destination_cell.follow()?);
                } else {
                    let target = attempt.provision(b"effect");
                    target.point_to(&snapshot[index]);
                    let source = Self::source_binding(candidate, context)?;
                    self.execute(candidate, &target, source.as_ref())?;
                    attempt.provision(b"after").point_to(&target.follow()?);
                }
                attempt.provision(b"completed").terminate();
                Ok(())
            })();
            if let Err(error) = result {
                attempt.provision(b"fault").terminate();
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(())
    }

    pub fn transduce_grid(
        &mut self,
        height: usize,
        width: usize,
        input: &[Address],
    ) -> Result<Vec<Address>, TransductionError> {
        if height == 0 || width == 0 || height.checked_mul(width) != Some(input.len()) {
            return Err(TransductionError::Shape);
        }
        let run = self.serial;
        self.serial += 1;
        let output_root = self
            .machine
            .output
            .root()
            .provision(b"grid-episode-results")
            .provision(&run.to_be_bytes());
        let cells = input
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let cell = output_root.provision(&index.to_be_bytes());
                cell.point_to(value);
                cell
            })
            .collect::<Vec<_>>();
        let mut values = input.to_vec();
        let shape_applied = self.apply_shape_transductions(height, width, &values, &cells)?;
        let translation_applied =
            !shape_applied && self.apply_translation_plan(height, width, &values, &cells)?;
        let component_applied = !shape_applied
            && !translation_applied
            && self.apply_component_rule(height, width, &values, &cells)?;
        let enclosed_applied = !shape_applied
            && !translation_applied
            && !component_applied
            && self.apply_enclosed_fill(height, width, &values, &cells)?;
        let stencil_applied = !shape_applied
            && !translation_applied
            && !component_applied
            && !enclosed_applied
            && self.apply_singleton_stencils(height, width, &values, &cells)?;
        if shape_applied
            || translation_applied
            || component_applied
            || enclosed_applied
            || stencil_applied
        {
            values = cells
                .iter()
                .map(Address::follow)
                .collect::<Result<Vec<_>, _>>()?;
        }
        if shape_applied || translation_applied || enclosed_applied || stencil_applied {
            // These broad plans encode a complete mask-derived state transition. Component rules
            // may deliberately cover only one role and therefore continue into local cue lookup.
            self.applications += 1;
            return Ok(values);
        }
        for _ in 0..input.len().saturating_mul(4) {
            let snapshot = values.clone();
            let mut pending = Vec::new();
            let mut targets = std::collections::BTreeSet::new();
            for index in 0..snapshot.len() {
                let context = self.grid_context(&snapshot, height, width, index, run);
                let implementations = match self.contexts.implementations(&context) {
                    Ok((_, implementations)) => implementations,
                    Err(EpisodeError::Missing) => continue,
                    Err(error) => return Err(error.into()),
                };
                if implementations.is_empty() {
                    continue;
                }
                if implementations.len() > 1 {
                    self.attempt_grid_candidates(
                        (height, width),
                        index,
                        run,
                        &snapshot,
                        &context,
                        &implementations,
                    )?;
                    return Err(TransductionError::Ambiguous);
                }
                let entry = implementations[0].clone();
                if let Some(port) = entry.child(b"move-port") {
                    let port = port.follow()?;
                    let destination = Self::move_destination(
                        height,
                        width,
                        index,
                        port.token().ok_or(TransductionError::Incomplete)?,
                    )
                    .ok_or(TransductionError::Incomplete)?;
                    if !targets.insert(index) || !targets.insert(destination) {
                        return Err(TransductionError::Incomplete);
                    }
                    pending.push(PendingAction::Move {
                        source: index,
                        destination,
                        entry,
                    });
                } else {
                    if !targets.insert(index) {
                        return Err(TransductionError::Incomplete);
                    }
                    pending.push(PendingAction::Write {
                        index,
                        entry: entry.clone(),
                        source: Self::source_binding(&entry, &context)?,
                    });
                }
            }
            for action in pending {
                match action {
                    PendingAction::Write {
                        index,
                        entry,
                        source,
                    } => self.execute(&entry, &cells[index], source.as_ref())?,
                    PendingAction::Move {
                        source,
                        destination,
                        entry,
                    } => self.execute_move(
                        &entry,
                        &snapshot[source],
                        &cells[source],
                        &cells[destination],
                    )?,
                }
            }
            values = cells
                .iter()
                .map(Address::follow)
                .collect::<Result<Vec<_>, _>>()?;
            if values.iter().zip(&snapshot).all(|(a, b)| a.same(b)) {
                return Ok(values);
            }
        }
        Err(TransductionError::Limit)
    }

    /// Readdress the derived row until no unique context construction remains.
    pub fn transduce(&mut self, input: &[Address]) -> Result<Vec<Address>, TransductionError> {
        if input.len() < 3 {
            return Err(TransductionError::Shape);
        }
        let run = self.serial;
        self.serial += 1;
        let output_root = self
            .machine
            .output
            .root()
            .provision(b"episode-results")
            .provision(&run.to_be_bytes());
        let cells = input
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let cell = output_root.provision(&index.to_be_bytes());
                cell.point_to(value);
                cell
            })
            .collect::<Vec<_>>();
        let mut values = input.to_vec();
        for _ in 0..input.len().saturating_mul(4) {
            let mut changed = false;
            for index in 1..values.len() - 1 {
                let context = self.context(&values, index, run);
                let Ok(resolved) = self.contexts.resolve(&context) else {
                    continue;
                };
                let Some(entry) = resolved.implementation else {
                    continue;
                };
                let source = Self::source_binding(&entry, &context)?;
                self.execute(&entry, &cells[index], source.as_ref())?;
                let value = cells[index].follow()?;
                if !value.same(&values[index]) {
                    values[index] = value;
                    changed = true;
                }
            }
            if !changed {
                return Ok(values);
            }
        }
        Err(TransductionError::Limit)
    }
}
