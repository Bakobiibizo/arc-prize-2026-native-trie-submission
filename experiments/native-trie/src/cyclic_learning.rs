//! ALL-demo cyclic lattice grammar. No decoded period/modulo or palette order.
use super::cyclic_lattice::CyclicProjection;
use super::*;
use crate::region_witness::RegionWitness;
use std::collections::BTreeMap;

/// Provenance of one independently represented construction that replayed every demonstration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneralizedCandidateOrigin {
    Prior,
    Broad,
    Group,
    Partition,
    Cyclic,
    Intrinsic,
}
impl GeneralizedCandidateOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prior => "prior-replay-valid",
            Self::Broad => "broad-replay-valid",
            Self::Group => "group-replay-valid",
            Self::Partition => "partition-replay-valid",
            Self::Cyclic => "cyclic-replay-valid",
            Self::Intrinsic => "intrinsic-replay-valid",
        }
    }
}

/// One ordered replay-valid construction. Candidate identity is the programme's trie address.
#[derive(Clone)]
pub struct GeneralizedCandidate {
    program: GroupProgram,
    origin: GeneralizedCandidateOrigin,
    replay_address: Option<(usize, Address)>,
}
impl GeneralizedCandidate {
    pub fn broad_replay_valid(program: GroupProgram) -> Self {
        let addressed = program.root.provision(b"standalone-replay-valid-candidate");
        addressed.provision(b"program").point_to(&program.root);
        addressed
            .provision(b"provenance")
            .provision(GeneralizedCandidateOrigin::Broad.as_str().as_bytes())
            .terminate();
        addressed
            .provision(b"all-demonstrations-replayed")
            .terminate();
        Self {
            program,
            origin: GeneralizedCandidateOrigin::Broad,
            // Index zero is reserved for the historical attempt-1 slot even when it abstains.
            replay_address: Some((1, addressed)),
        }
    }

    pub fn program(&self) -> &GroupProgram {
        &self.program
    }
    pub fn origin(&self) -> GeneralizedCandidateOrigin {
        self.origin
    }
    pub fn replay_address_index(&self) -> Option<usize> {
        self.replay_address.as_ref().map(|(index, _)| *index)
    }
    pub fn has_all_demonstration_replay_provenance(&self) -> bool {
        let Some((_, addressed)) = &self.replay_address else {
            return false;
        };
        addressed
            .child(b"program")
            .and_then(|program| program.follow().ok())
            .is_some_and(|program| program.same(&self.program.root))
            && addressed
                .child(b"provenance")
                .and_then(|provenance| provenance.child(self.origin.as_str().as_bytes()))
                .is_some_and(|origin| origin.is_terminal())
            && addressed
                .child(b"all-demonstrations-replayed")
                .is_some_and(|marker| marker.is_terminal())
    }
}

/// Canonically ordered candidates from one generalized induction pass.
/// The first entry is exactly the historical minimum-operation selection.
pub struct GeneralizedCandidateSet {
    pub root: Address,
    candidates: Vec<GeneralizedCandidate>,
}
impl GeneralizedCandidateSet {
    pub fn candidates(&self) -> &[GeneralizedCandidate] {
        &self.candidates
    }
    pub fn primary(&self) -> Option<&GeneralizedCandidate> {
        self.candidates.first()
    }
    pub fn secondary(&self) -> Option<&GeneralizedCandidate> {
        self.candidates.get(1)
    }
}

fn retain_distinct_candidate(
    candidates: &mut Vec<GeneralizedCandidate>,
    candidate: GeneralizedCandidate,
) {
    if !candidates
        .iter()
        .any(|retained| retained.program.root.same(&candidate.program.root))
    {
        candidates.push(candidate);
    }
}

fn order_generalized_candidates(
    induced: &[(&GroupProgram, GeneralizedCandidateOrigin)],
) -> Vec<GeneralizedCandidate> {
    let mut candidates = induced
        .iter()
        .map(|(program, origin)| GeneralizedCandidate {
            program: (*program).clone(),
            origin: *origin,
            replay_address: None,
        })
        .collect::<Vec<_>>();
    // This is the historical primary policy. Stable sorting preserves prior-before-cyclic and
    // group-before-partition ties while making minimum operation count the selection key.
    candidates.sort_by_key(|candidate| candidate.program.groups());
    let mut distinct = Vec::new();
    for candidate in candidates {
        retain_distinct_candidate(&mut distinct, candidate);
    }
    distinct
}

fn has_canonical_broad_representation(program: &GroupProgram) -> bool {
    [
        b"canonical-mask-composition".as_slice(),
        b"canonical-axis-projection",
        b"canonical-cardinal-segment-placement",
        b"canonical-relative-stencil-placement",
        b"canonical-dihedral-translation-role-transform",
        b"canonical-periodic-lattice-coordinate-generator",
        b"canonical-bounding-envelope-corner-geometry",
    ]
    .into_iter()
    .any(|name| program.root.child(name).is_some())
}

fn retain_broad_alternatives(
    candidates: &mut Vec<GeneralizedCandidate>,
    broad_candidates: &[GroupProgram],
) {
    let mut alternatives = broad_candidates
        .iter()
        .skip(1)
        .map(|program| GeneralizedCandidate {
            program: program.clone(),
            origin: GeneralizedCandidateOrigin::Broad,
            replay_address: None,
        })
        .collect::<Vec<_>>();
    alternatives.sort_by_key(|candidate| !has_canonical_broad_representation(&candidate.program));
    for candidate in alternatives {
        retain_distinct_candidate(candidates, candidate);
    }
}

impl SparseRewrite {
    fn cyclic_teacher_gate(
        &mut self,
        lessons: &[GroupLesson],
        scope: &Address,
    ) -> Result<bool, Error> {
        let unknown = self.unknown.clone();
        for d in lessons {
            let mut successors: BTreeMap<_, (Address, Address)> = BTreeMap::new();
            for row in field(&d.evidence, b"pairs")?.inspect_children() {
                for pair in row.inspect_children() {
                    let before = field(&pair, b"after")?.follow()?;
                    let slot = field(&before, b"value")?;
                    let value = slot.follow()?;
                    for port in [b"east".as_slice(), b"south"] {
                        self.rectangle_observe()?;
                        let next = field(&before, port)?.follow()?;
                        let Some(next_slot) = next.child(b"value") else {
                            continue;
                        };
                        let next_value = next_slot.follow()?;
                        let at = scope.provision(&pair.identity_key()).provision(port);
                        at.provision(b"teacher-source-slot").point_to(&slot);
                        at.provision(b"teacher-next-slot").point_to(&next_slot);
                        for (s, v) in [(&slot, &value), (&next_slot, &next_value)] {
                            let (tag, fact) = self.context_equality_fact(v, &unknown)?;
                            at.provision(b"knownness")
                                .provision(&s.identity_key())
                                .point_to(&fact);
                            if tag.same(&self.yes) {
                                scope.provision(b"unavailable").point_to(&at);
                                return Ok(false);
                            }
                        }
                        if let Some((representative, expected)) =
                            successors.get(&value.identity_key())
                        {
                            let (tag, fact) =
                                self.context_equality_fact(&value, &representative.follow()?)?;
                            at.provision(b"source-class-equality").point_to(&fact);
                            if !tag.same(&self.yes) {
                                return Err(Error::Incomplete);
                            }
                            let (tag, fact) =
                                self.context_equality_fact(&next_value, &expected.follow()?)?;
                            at.provision(b"successor-equality").point_to(&fact);
                            if !tag.same(&self.yes) {
                                scope.provision(b"contradiction").point_to(&at);
                                return Ok(false);
                            }
                        } else {
                            successors.insert(value.identity_key(), (slot.clone(), next_slot));
                        }
                    }
                }
            }
        }
        scope.provision(b"all-teacher-edges-valid").terminate();
        Ok(true)
    }
    fn cyclic_input_projection(
        &mut self,
        input: &Grid,
        bg: &Address,
    ) -> Result<Option<CyclicProjection>, Error> {
        if !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        let scope = self
            .root
            .provision(b"cyclic-input-projections")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        scope.provision(b"input").point_to(&input.anchor);
        scope.provision(b"background-reference").point_to(bg);
        let values = scope.provision(b"input-value-references");
        let mut distinct = BTreeMap::new();
        let mut guards = Vec::new();
        for y in 0..input.height() {
            for x in 0..input.width() {
                self.rectangle_observe()?;
                let slot = field(&input.cell(y, x)?, b"value")?;
                let value = slot.follow()?;
                guards.push((slot.clone(), self.unknown.clone()));
                let (tag, fact) = self.context_equality_fact(&value, bg)?;
                scope
                    .provision(b"native-background-exclusions")
                    .provision(&slot.identity_key())
                    .point_to(&fact);
                if !tag.same(&self.yes) {
                    records::record(&values, b"reference", &value, &slot)?;
                    distinct.entry(value.identity_key()).or_insert(value);
                }
            }
        }
        let (cp, cv) = self.run_set_count(values, &guards, input)?;
        scope.provision(b"input-value-count").point_to(&cp);
        let missing = self
            .root
            .provision(b"native-components")
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let (tag, fact) = self.property_knownness(&cp, &cv, &missing)?;
        scope.provision(b"input-value-knownness").point_to(&fact);
        if !tag.same(&self.yes) {
            return Ok(None);
        }
        let zero = RegionWitness::cardinality_zero(&self.root.provision(b"native-components"));
        if self.context_equal(&cv, &zero)? {
            scope.provision(b"empty-value-universe").terminate();
            return Ok(None);
        }
        // One structural node per actual indexed reference entry. No ordinal decoding
        // or assignment of a palette value to that anonymous phase.
        let program = self.cyclic_program(bg, distinct.len())?;
        scope
            .provision(b"constructed-cycle")
            .point_to(&program.root);
        let phases = field(&program.root, b"phases")?.inspect_children();
        let phase_set = scope.provision(b"phase-references");
        for ((_, value), phase) in distinct.into_iter().zip(&phases) {
            let entry = scope
                .provision(b"one-phase-per-entry")
                .provision(&value.identity_key());
            entry.provision(b"input-value-reference").point_to(&value);
            entry.provision(b"anonymous-phase").point_to(phase);
            records::record(&phase_set, b"reference", phase, &entry)?;
        }
        let (pp, pv) = self.run_set_count(phase_set, &guards, input)?;
        scope.provision(b"phase-count").point_to(&pp);
        let (tag, fact) = self.context_equality_fact(&cv, &pv)?;
        scope.provision(b"native-count-coverage").point_to(&fact);
        if !tag.same(&self.yes) {
            return Err(Error::Incomplete);
        }
        match self.cyclic_projection(input, &program) {
            Ok(p) => {
                scope.provision(b"native-render").point_to(&p.proof);
                scope.provision(b"committed").terminate();
                Ok(Some(p))
            }
            Err(Error::Incomplete) => {
                scope.provision(b"incomplete-construction").terminate();
                Ok(None)
            }
            Err(Error::Fault(crate::Fault::Unset)) => {
                scope.provision(b"missing-phase-binding").terminate();
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
    pub fn learn_cyclic_actions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        let assembly = self
            .root
            .provision(b"cyclic-assemblies")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        if !self.cyclic_teacher_gate(lessons, &assembly.provision(b"teacher-gate"))? {
            return Ok(None);
        }
        let mut best = None;
        'background: for bg in self.ordered_backgrounds(lessons)? {
            let body = assembly
                .provision(b"background-candidates")
                .provision(&bg.identity_key());
            body.provision(b"background-reference").point_to(&bg);
            for d in lessons {
                let Some(output) = self.cyclic_input_projection(&d.input, &bg)? else {
                    body.provision(b"unavailable-input")
                        .point_to(&d.input.anchor);
                    continue 'background;
                };
                body.provision(b"native-renders")
                    .provision(&d.input.anchor.identity_key())
                    .point_to(&output.proof);
                let teacher_rows = field(&d.evidence, b"pairs")?.inspect_children();
                if teacher_rows.len() != output.rows.len() {
                    return Err(Error::Incomplete);
                }
                for (row, produced) in teacher_rows.iter().zip(&output.rows) {
                    let teacher_cells = row.inspect_children();
                    let actual_cells = field(produced, b"cells")?.inspect_children();
                    if teacher_cells.len() != actual_cells.len() {
                        return Err(Error::Incomplete);
                    }
                    for (pair, actual) in teacher_cells.iter().zip(actual_cells) {
                        self.rectangle_observe()?;
                        let actual_slot = field(&actual, b"value")?;
                        let expected =
                            field(&field(pair, b"after")?.follow()?, b"value")?.follow()?;
                        let unknown = self.unknown.clone();
                        let (unknown_tag, unknown_fact) =
                            self.context_equality_fact(&expected, &unknown)?;
                        body.provision(b"teacher-label-knownness")
                            .provision(&pair.identity_key())
                            .point_to(&unknown_fact);
                        if unknown_tag.same(&self.yes) {
                            body.provision(b"unavailable-teacher-label").point_to(pair);
                            continue 'background;
                        }
                        let (tag, fact) =
                            self.context_equality_fact(&actual_slot.follow()?, &expected)?;
                        let at = body
                            .provision(b"all-label-validation")
                            .provision(&pair.identity_key());
                        at.provision(b"pair").point_to(pair);
                        at.provision(b"actual-output-slot").point_to(&actual_slot);
                        at.provision(b"native-output-equality").point_to(&fact);
                        if !tag.same(&self.yes) {
                            body.provision(b"contradiction").point_to(&at);
                            continue 'background;
                        }
                    }
                }
            }
            body.provision(b"all-observations-valid").terminate();
            let program = body.provision(b"program");
            program.provision(b"cyclic-lattice-program").terminate();
            program.provision(b"background-value").point_to(&bg);
            program
                .provision(b"groups")
                .provision(b"phase-bound-original-slot-copy")
                .terminate();
            if best.is_none() {
                best = Some(GroupProgram {
                    root: program,
                    owner: self.root.clone(),
                });
            }
        }
        self.guard(0)?;
        Ok(best)
    }
    fn validate_persistent_episode(
        &mut self,
        episode: &[u8],
        resident_evidence: &Address,
        shape: (usize, usize),
        input: &[Address],
        expected: &[Address],
    ) -> Result<(), Error> {
        self.persistent_arc()?;
        let disk = self.persistent_arc.as_ref().ok_or(Error::Incomplete)?;
        let evidence = disk
            .store_root()
            .provision(b"teaching-evidence")
            .map_err(|_| Error::Incomplete)?
            .provision(&resident_evidence.identity_key())
            .map_err(|_| Error::Incomplete)?;
        evidence.terminate().map_err(|_| Error::Incomplete)?;
        self.episode_transducer
            .validate_grid_persistent(disk, episode, &evidence, shape, input, expected)
            .map_err(transduction_error)
    }

    /// Analyze all demonstrations once and retain every independently replay-valid construction.
    /// The historical minimum-operation selection is pinned first. Additional broad candidates
    /// from the same complete-demonstration pass follow it, with canonical representations first.
    pub fn agree_generalized_candidates(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<GeneralizedCandidateSet, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        let broad_candidates = self.learn_broad_structural_candidates(lessons)?;
        let mut prior_candidates = Vec::new();
        if let Some(program) = broad_candidates.first() {
            // Candidate zero is the first matching schema in the historical induction order.
            prior_candidates.push((program.clone(), GeneralizedCandidateOrigin::Prior));
        } else if let Some(program) = self.agree_ordered_objects(lessons)? {
            prior_candidates.push((program, GeneralizedCandidateOrigin::Prior));
        } else if lessons
            .iter()
            .all(|lesson| lesson.input.height().saturating_mul(lesson.input.width()) <= 400)
        {
            // These independently replayed construction forms are not ancestors of the ordered
            // relation chain. Retrieve them as one evidence-addressed fallback set rather than
            // requiring a command-line family choice. The 400-cell admission bound protects the
            // measured 286,720,000-byte resident workspace ceiling: attempting all alternatives
            // on 900-cell demonstrations reached 260,090,611 bytes before completion.
            if let Some(program) = self.agree_groups(lessons)? {
                prior_candidates.push((program, GeneralizedCandidateOrigin::Group));
            }
            if let Some(program) = self.agree_partitions(lessons)? {
                prior_candidates.push((program, GeneralizedCandidateOrigin::Partition));
            }
        }
        let prior = prior_candidates
            .iter()
            .min_by_key(|(program, _)| program.groups())
            .map(|(program, _)| program);
        let cyclic = self.learn_cyclic_actions(lessons)?;
        let choice = self
            .root
            .provision(b"broad-construction-evidence")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        if let Some(program) = &prior {
            choice.provision(b"prior-candidate").point_to(&program.root);
        }
        if let Some(program) = &cyclic {
            choice
                .provision(b"cyclic-candidate")
                .point_to(&program.root);
        }

        let mut historically_induced = prior_candidates
            .iter()
            .map(|(program, origin)| (program, *origin))
            .collect::<Vec<_>>();
        if let Some(program) = cyclic.as_ref() {
            historically_induced.push((program, GeneralizedCandidateOrigin::Cyclic));
        }
        let historical_order = order_generalized_candidates(&historically_induced);
        let mut candidates = Vec::new();
        if let Some(primary) = historical_order.first() {
            retain_distinct_candidate(&mut candidates, primary.clone());
        }

        retain_broad_alternatives(&mut candidates, &broad_candidates);
        for candidate in historical_order.into_iter().skip(1) {
            retain_distinct_candidate(&mut candidates, candidate);
        }

        for (index, candidate) in candidates.iter_mut().enumerate() {
            let addressed = choice
                .provision(b"ordered-replay-valid-candidates")
                .provision(&index.to_be_bytes());
            addressed
                .provision(b"program")
                .point_to(&candidate.program.root);
            addressed
                .provision(b"provenance")
                .provision(candidate.origin.as_str().as_bytes())
                .terminate();
            addressed
                .provision(b"all-demonstrations-replayed")
                .terminate();
            candidate.replay_address = Some((index, addressed));
        }
        if let Some(primary) = candidates.first() {
            choice
                .provision(b"selected")
                .point_to(&primary.program.root);
            choice
                .provision(b"minimum-operation-count-policy")
                .terminate();
        } else {
            choice.provision(b"no-broad-construction").terminate();
            if let Some(program) = self.agree_cyclic_actions(lessons)? {
                let addressed = choice
                    .provision(b"ordered-replay-valid-candidates")
                    .provision(&0usize.to_be_bytes());
                addressed.provision(b"program").point_to(&program.root);
                addressed
                    .provision(b"provenance")
                    .provision(GeneralizedCandidateOrigin::Intrinsic.as_str().as_bytes())
                    .terminate();
                addressed
                    .provision(b"all-demonstrations-replayed")
                    .terminate();
                choice.provision(b"selected").point_to(&program.root);
                candidates.push(GeneralizedCandidate {
                    program,
                    origin: GeneralizedCandidateOrigin::Intrinsic,
                    replay_address: Some((0, addressed)),
                });
            }
        }
        Ok(GeneralizedCandidateSet {
            root: choice,
            candidates,
        })
    }

    /// Historical compatibility entry point. It returns candidate one from the same single pass.
    pub fn agree_generalized_actions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        Ok(self
            .agree_generalized_candidates(lessons)?
            .primary()
            .map(|candidate| candidate.program.clone()))
    }

    /// Learn output constructions at intrinsic input addresses. The historical method name is
    /// retained as an API compatibility entry point and keeps its direct intrinsic semantics.
    pub fn agree_cyclic_actions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        let continuation = self
            .root
            .provision(b"teaching-continuations")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        continuation
            .provision(b"input-to-mask-to-construction-to-evaluation-to-output")
            .terminate();
        let mut admitted = Vec::new();
        for (index, lesson) in lessons.iter().enumerate() {
            let step = continuation
                .provision(b"examples")
                .provision(&index.to_be_bytes());
            step.provision(b"evidence").point_to(&lesson.evidence);
            let input = (0..lesson.input.height())
                .flat_map(|y| (0..lesson.input.width()).map(move |x| (y, x)))
                .map(|(y, x)| Ok(field(&lesson.input.cell(y, x)?, b"value")?.follow()?))
                .collect::<Result<Vec<_>, Error>>()?;
            let output = (0..lesson.output.height())
                .flat_map(|y| (0..lesson.output.width()).map(move |x| (y, x)))
                .map(|(y, x)| Ok(field(&lesson.output.cell(y, x)?, b"value")?.follow()?))
                .collect::<Result<Vec<_>, Error>>()?;

            // A later demonstration is held out from induction. It must first be reproduced by
            // the constructions admitted from earlier examples.
            if index > 0 {
                step.provision(b"held-out-before-admission").terminate();
                let mut validation_episode = continuation.identity_key().to_vec();
                validation_episode.extend_from_slice(&index.to_be_bytes());
                validation_episode.extend_from_slice(b"held-out");
                if self
                    .validate_persistent_episode(
                        &validation_episode,
                        &lesson.evidence,
                        (lesson.input.height(), lesson.input.width()),
                        &input,
                        &output,
                    )
                    .is_err()
                {
                    // A mismatch is evidence that the current vocabulary is incomplete, not a
                    // reason to discard the demonstration. Admit its distinct state cues below;
                    // publication still requires every accumulated example to validate afterward.
                    step.provision(b"held-out-mask-mismatch").terminate();
                    step.provision(b"requires-additional-construction")
                        .terminate();
                } else {
                    step.provision(b"held-out-mask-match").terminate();
                }
            }

            step.provision(b"input-admitted").terminate();
            step.provision(b"expected-mask-derived").terminate();
            self.episode_transducer
                .teach_grid(lesson.input.height(), lesson.input.width(), &input, &output)
                .map_err(transduction_error)?;
            step.provision(b"constructions-induced-at-state-cues")
                .terminate();
            admitted.push((lesson, input, output));

            // Re-evaluate every admitted example after consolidation. This catches a newly
            // introduced ambiguity or collision instead of assuming the shared trie is safe.
            for (prior_index, (prior, prior_input, prior_output)) in admitted.iter().enumerate() {
                let evaluation = step
                    .provision(b"post-admission-evaluations")
                    .provision(&prior_index.to_be_bytes());
                let mut validation_episode = continuation.identity_key().to_vec();
                validation_episode.extend_from_slice(&index.to_be_bytes());
                validation_episode.extend_from_slice(&prior_index.to_be_bytes());
                validation_episode.extend_from_slice(b"replay");
                if self
                    .validate_persistent_episode(
                        &validation_episode,
                        &prior.evidence,
                        (prior.input.height(), prior.input.width()),
                        prior_input,
                        prior_output,
                    )
                    .is_err()
                {
                    evaluation.provision(b"mask-mismatch").terminate();
                    return Ok(None);
                }
                evaluation.provision(b"mask-match").terminate();
            }
            step.provision(b"output-validated").terminate();
        }
        let program = self
            .root
            .provision(b"episode-transduction-programs")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        program
            .provision(b"teaching-continuation")
            .point_to(&continuation);
        program
            .provision(b"episode-transduction-program")
            .terminate();
        for (index, lesson) in lessons.iter().enumerate() {
            program
                .provision(b"evidence")
                .provision(&index.to_be_bytes())
                .point_to(&lesson.evidence);
        }
        Ok(Some(GroupProgram {
            root: program,
            owner: self.root.clone(),
        }))
    }
    pub(super) fn cyclic_applicable(
        &mut self,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<bool, Error> {
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        Ok(self
            .cyclic_input_projection(input, &field(&program.root, b"background-value")?.follow()?)?
            .is_some())
    }
    pub(super) fn apply_cyclic_actions(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<Grid, Error> {
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        let p = self
            .cyclic_input_projection(input, &field(&program.root, b"background-value")?.follow()?)?
            .ok_or(Error::NotReady)?;
        p.proof
            .provision(b"learned-template")
            .point_to(&program.root);
        self.frozen = true;
        p.export(s)
    }
}

#[cfg(test)]
mod candidate_order_tests {
    use super::*;
    use crate::arc_grids::Grids;

    fn program(machine: &SparseRewrite, name: &[u8], groups: usize) -> GroupProgram {
        let root = machine
            .root
            .provision(b"candidate-order-test")
            .provision(name);
        for index in 0..groups {
            root.provision(b"groups").provision(&index.to_be_bytes());
        }
        GroupProgram {
            root,
            owner: machine.root.clone(),
        }
    }

    #[test]
    fn minimum_operation_order_is_stable_and_distinct() {
        let grids = Grids::new();
        let machine = SparseRewrite::new(&grids);
        let prior = program(&machine, b"prior", 2);
        let cyclic = program(&machine, b"cyclic", 1);
        let candidates = order_generalized_candidates(&[
            (&prior, GeneralizedCandidateOrigin::Prior),
            (&cyclic, GeneralizedCandidateOrigin::Cyclic),
        ]);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].origin(), GeneralizedCandidateOrigin::Cyclic);
        assert!(candidates[0].program().root.same(&cyclic.root));
        assert_eq!(candidates[1].origin(), GeneralizedCandidateOrigin::Prior);

        let tie = program(&machine, b"tie", 2);
        let candidates = order_generalized_candidates(&[
            (&prior, GeneralizedCandidateOrigin::Group),
            (&tie, GeneralizedCandidateOrigin::Partition),
        ]);
        assert_eq!(candidates[0].origin(), GeneralizedCandidateOrigin::Group);
        assert!(candidates[0].program().root.same(&prior.root));

        let candidates = order_generalized_candidates(&[
            (&prior, GeneralizedCandidateOrigin::Prior),
            (&prior, GeneralizedCandidateOrigin::Cyclic),
        ]);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].origin(), GeneralizedCandidateOrigin::Prior);
    }

    #[test]
    fn broad_alternatives_never_displace_primary_and_are_deduplicated() {
        let grids = Grids::new();
        let machine = SparseRewrite::new(&grids);
        let primary = program(&machine, b"primary", 2);
        let fewer_operations = program(&machine, b"fewer-operations", 0);
        let canonical = program(&machine, b"canonical", 1);
        canonical
            .root
            .provision(b"canonical-axis-projection")
            .terminate();
        let mut candidates = vec![GeneralizedCandidate {
            program: primary.clone(),
            origin: GeneralizedCandidateOrigin::Prior,
            replay_address: None,
        }];

        retain_broad_alternatives(
            &mut candidates,
            &[
                primary.clone(),
                fewer_operations.clone(),
                canonical.clone(),
                canonical.clone(),
            ],
        );

        assert_eq!(candidates.len(), 3);
        assert!(candidates[0].program().root.same(&primary.root));
        assert!(candidates[1].program().root.same(&canonical.root));
        assert!(candidates[2].program().root.same(&fewer_operations.root));
        assert!(candidates[2].program().groups() < candidates[0].program().groups());
    }
}
