//! ALL-demo ordered-object construction. Supplied finite role grammar;
//! native predicates certify universe/geometry/order/output, never host pixels.
use super::*;
use crate::region_witness::RegionWitness;
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
pub(super) struct OrderedInput {
    bindings: Vec<(Address, Address)>,
    rectangles: Option<Vec<Rc<RectangleProjection>>>,
}
impl SparseRewrite {
    fn ordered_input(
        &mut self,
        input: &Grid,
        bg: &Address,
        port: &[u8],
    ) -> Result<Option<Vec<Rc<RectangleProjection>>>, Error> {
        if ![b"east".as_slice(), b"west"].contains(&port) {
            return Err(Error::Incomplete);
        }
        let scope = self
            .root
            .provision(b"ordered-inputs")
            .provision(&input.anchor.identity_key())
            .provision(&bg.identity_key())
            .provision(port);
        let key = scope.identity_key().to_vec();
        if let Some(cached) = self.ordered_inputs.get(&key) {
            for (s, v) in &cached.bindings {
                if !s.follow()?.same(v) {
                    return Err(Error::Incomplete);
                }
            }
            if let Some(ps) = &cached.rectangles {
                for p in ps {
                    p.check_bindings()?;
                }
            }
            return Ok(cached.rectangles.clone());
        }
        scope.provision(b"input").point_to(&input.anchor);
        scope.provision(b"background-reference").point_to(bg);
        let mut bindings = Vec::new();
        for y in 0..input.height() {
            for x in 0..input.width() {
                let slot = field(&input.cell(y, x)?, b"value")?;
                bindings.push((slot.clone(), slot.follow()?));
            }
        }
        let result = self.build_ordered_input(input, bg, port, &scope)?;
        self.ordered_inputs.insert(
            key,
            OrderedInput {
                bindings,
                rectangles: result.clone(),
            },
        );
        Ok(result)
    }
    fn build_ordered_input(
        &mut self,
        input: &Grid,
        bg: &Address,
        port: &[u8],
        scope: &Address,
    ) -> Result<Option<Vec<Rc<RectangleProjection>>>, Error> {
        let mut all = BTreeMap::new();
        let original = scope.provision(b"all-components");
        let native = self.root.provision(b"native-components");
        let member = native.provision(b"class-tags").provision(b"member");
        let zero = RegionWitness::cardinality_zero(&native);
        let outside = self.outside.clone();
        for y in 0..input.height() {
            for x in 0..input.width() {
                self.rectangle_observe()?;
                let cell = input.cell(y, x)?;
                let value = field(&cell, b"value")?.follow()?;
                if self.context_equal(&value, bg)? {
                    continue;
                }
                let (region, _) = self.component_value(input, &cell)?;
                if all.contains_key(&region.identity_key()) {
                    continue;
                }
                let (cp, cv) = self.complete_count_value(input, &cell)?;
                let missing = self
                    .root
                    .provision(b"complete-count-tags")
                    .provision(b"unavailable");
                let (known, fact) = self.property_knownness(&cp, &cv, &missing)?;
                scope
                    .provision(b"complete-components")
                    .provision(&region.identity_key())
                    .point_to(&fact);
                if !known.same(&self.yes) {
                    scope.provision(b"unavailable-component").point_to(&region);
                    return Ok(None);
                }
                let bottom = scope
                    .provision(b"bottom-members")
                    .provision(&region.identity_key());
                for view in field(&region, b"views")?.inspect_children() {
                    self.rectangle_observe()?;
                    if !self.context_equal(&field(&view, b"class")?.follow()?, &member)? {
                        continue;
                    }
                    let source = field(&view, b"source")?.follow()?;
                    let south = field(&source, b"south")?.follow()?;
                    let (tag, p) = self.context_equality_fact(&south, &outside)?;
                    if tag.same(&self.yes) {
                        records::record(&bottom, b"reference", &source, &p)?;
                    }
                }
                let (bp, bv) = self.run_set_count(bottom, &[], input)?;
                scope
                    .provision(b"bottom-proofs")
                    .provision(&region.identity_key())
                    .point_to(&bp);
                if self.context_equal(&bv, &zero)? {
                    scope.provision(b"off-bottom-component").point_to(&region);
                    return Ok(None);
                }
                let p = Rc::new(self.rectangle_source(input, &cell)?);
                let rectangle = self
                    .root
                    .provision(b"rectangle-tags")
                    .provision(b"rectangle");
                if !field(&p.proof, b"status")?.follow()?.same(&rectangle) {
                    scope.provision(b"invalid-rectangle").point_to(&p.proof);
                    return Ok(None);
                }
                records::record(&original, b"reference", &region, &p.proof)?;
                all.insert(region.identity_key(), p);
            }
        }
        let known = self.grid_known(input)?;
        let value = field(&known, b"value")?.follow()?;
        let missing = native
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let (ready, fact) = self.property_knownness(&known, &value, &missing)?;
        scope.provision(b"all-input-known").point_to(&fact);
        if !ready.same(&self.yes) {
            return Ok(None);
        }
        let (op, ov) = self.run_set_count(original, &[], input)?;
        scope.provision(b"original-universe-count").point_to(&op);
        if self.context_equal(&ov, &zero)? {
            scope.provision(b"empty-object-universe").terminate();
            return Ok(None);
        }
        let mut at = input.cell(
            input.height() - 1,
            if port == b"east" {
                0
            } else {
                input.width() - 1
            },
        )?;
        let mut seen = BTreeSet::new();
        let mut order = Vec::new();
        let selected = scope.provision(b"ordered-components");
        while at.child(b"value").is_some() {
            self.rectangle_observe()?;
            if !self.context_equal(&field(&at, b"value")?.follow()?, bg)? {
                let (p, _) = self.component_value(input, &at)?;
                if seen.insert(p.identity_key()) {
                    let Some(rect) = all.get(&p.identity_key()) else {
                        return Err(Error::Incomplete);
                    };
                    records::record(&selected, b"reference", &p, &at)?;
                    scope
                        .provision(b"source-order")
                        .provision(&order.len().to_be_bytes())
                        .point_to(&rect.proof);
                    order.push(rect.clone());
                }
            }
            at = field(&at, port)?.follow()?;
        }
        let (sp, sv) = self.run_set_count(selected, &[], input)?;
        scope.provision(b"ordered-universe-count").point_to(&sp);
        let (tag, fact) = self.context_equality_fact(&ov, &sv)?;
        scope.provision(b"native-complete-order").point_to(&fact);
        if !tag.same(&self.yes) {
            return Ok(None);
        }
        scope.provision(b"committed").terminate();
        Ok(Some(order))
    }
    pub(super) fn ordered_backgrounds(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Vec<Address>, Error> {
        let mut common: BTreeMap<_, Address> = BTreeMap::new();
        for (i, d) in lessons.iter().enumerate() {
            let mut observed = BTreeMap::new();
            for (y, x) in [
                (0, 0),
                (0, d.input.width() - 1),
                (d.input.height() - 1, 0),
                (d.input.height() - 1, d.input.width() - 1),
            ] {
                let value = field(&d.input.cell(y, x)?, b"value")?.follow()?;
                observed.insert(value.identity_key(), value);
            }
            if i == 0 {
                common = observed
            } else {
                common.retain(|k, _| observed.contains_key(k));
            }
        }
        Ok(common.into_values().collect())
    }
    fn learn_ordered_objects(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        let assembly = self
            .root
            .provision(b"ordered-object-assemblies")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        let backgrounds = self.ordered_backgrounds(lessons)?;
        let mut best = None;
        'background: for bg in backgrounds {
            let mut by_order = Vec::new();
            for port in [b"east".as_slice(), b"west"] {
                let mut inputs = Vec::new();
                for d in lessons {
                    let Some(sources) = self.ordered_input(&d.input, &bg, port)? else {
                        continue 'background;
                    };
                    inputs.push(sources);
                }
                by_order.push((port, inputs));
            }
            for (port, inputs) in by_order {
                for first in [b"NW".as_slice(), b"NE", b"SW", b"SE"] {
                    for next in [b"NW".as_slice(), b"NE", b"SW", b"SE"] {
                        self.guard((128 * 160 + 900 * 24) * 640)?;
                        let proposal = assembly
                            .provision(&bg.identity_key())
                            .provision(port)
                            .provision(first)
                            .provision(next);
                        let mut valid = true;
                        for (d, sources) in lessons.iter().zip(&inputs) {
                            let concrete =
                                self.rectangle_copy_program(sources, &bg, first, next)?;
                            let mut w = match self.prepare_rectangle_copy(
                                &d.input,
                                &concrete,
                                sources.clone(),
                            ) {
                                Ok(w) => w,
                                Err(Error::Incomplete) => {
                                    proposal
                                        .provision(b"outside-or-invalid-placement")
                                        .point_to(&concrete);
                                    valid = false;
                                    break;
                                }
                                Err(e) => return Err(e),
                            };
                            w.execute(w.budget)?;
                            let output = w.commit_native()?;
                            proposal
                                .provision(b"native-renders")
                                .provision(&d.input.anchor.identity_key())
                                .point_to(&output);
                            for (y, row) in field(&d.evidence, b"pairs")?
                                .inspect_children()
                                .iter()
                                .enumerate()
                            {
                                for (x, pair) in row.inspect_children().iter().enumerate() {
                                    self.rectangle_observe()?;
                                    let produced =
                                        field(&field(&output, b"rows")?, &y.to_be_bytes())?
                                            .child(b"cells")
                                            .and_then(|c| c.child(&x.to_be_bytes()))
                                            .ok_or(Error::Incomplete)?;
                                    let value = field(&produced, b"value")?.follow()?;
                                    let expected =
                                        field(&field(pair, b"after")?.follow()?, b"value")?
                                            .follow()?;
                                    let (tag, fact) =
                                        self.context_equality_fact(&value, &expected)?;
                                    let at = proposal
                                        .provision(b"all-label-validation")
                                        .provision(&pair.identity_key());
                                    at.provision(b"pair").point_to(pair);
                                    at.provision(b"actual-output-slot")
                                        .point_to(&field(&produced, b"value")?);
                                    at.provision(b"native-output-equality").point_to(&fact);
                                    if !tag.same(&self.yes) {
                                        proposal.provision(b"contradiction").point_to(&at);
                                        valid = false;
                                        break;
                                    }
                                }
                                if !valid {
                                    break;
                                }
                            }
                            if !valid {
                                break;
                            }
                        }
                        if valid {
                            proposal.provision(b"all-observations-valid").terminate();
                            let program = proposal.provision(b"program");
                            program.provision(b"ordered-object-program").terminate();
                            program.provision(b"background-value").point_to(&bg);
                            program.provision(b"source-order").point_to(
                                &self
                                    .root
                                    .provision(b"ordered-object-port-keys")
                                    .provision(port),
                            );
                            for (k, r) in [
                                (b"initial-corner".as_slice(), first),
                                (b"next-corner", next),
                            ] {
                                program.provision(k).point_to(
                                    &self.root.provision(b"rectangle-corner-keys").provision(r),
                                );
                            }
                            program
                                .provision(b"groups")
                                .provision(b"background-copy")
                                .terminate();
                            program
                                .provision(b"groups")
                                .provision(b"ordered-rectangle-copy")
                                .terminate();
                            if best.is_none() {
                                best = Some(GroupProgram {
                                    root: program,
                                    owner: self.root.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        self.guard(0)?;
        Ok(best)
    }
    pub fn agree_ordered_objects(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        let prior = self.agree_path_value_actions(lessons)?;
        let induced = self.learn_ordered_objects(lessons)?;
        let choice = self
            .root
            .provision(b"ordered-object-construction-evidence")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        if let Some(program) = &prior {
            choice.provision(b"prior-candidate").point_to(&program.root);
        }
        if let Some(program) = &induced {
            choice
                .provision(b"induced-candidate")
                .point_to(&program.root);
        }
        let selected = match (prior, induced) {
            (Some(a), Some(b)) if b.groups() < a.groups() => Some(b),
            (Some(a), _) => Some(a),
            (_, b) => b,
        };
        if let Some(program) = &selected {
            choice.provision(b"selected").point_to(&program.root);
            choice
                .provision(b"minimum-operation-count-policy")
                .terminate();
        }
        Ok(selected)
    }
    pub(super) fn ordered_applicable(
        &mut self,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<bool, Error> {
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        let bg = field(&program.root, b"background-value")?.follow()?;
        let port = field(&program.root, b"source-order")?.follow()?;
        Ok(self
            .ordered_input(input, &bg, port.token().ok_or(Error::Incomplete)?)?
            .is_some())
    }
    pub(super) fn apply_ordered_objects(
        &mut self,
        s: &mut Grids,
        input: &Grid,
        program: &GroupProgram,
    ) -> Result<Grid, Error> {
        if !program.owner.same(&self.root) || !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        let bg = field(&program.root, b"background-value")?.follow()?;
        let port = field(&program.root, b"source-order")?.follow()?;
        let sources = self
            .ordered_input(input, &bg, port.token().ok_or(Error::Incomplete)?)?
            .ok_or(Error::NotReady)?;
        let initial = field(&program.root, b"initial-corner")?.follow()?;
        let next = field(&program.root, b"next-corner")?.follow()?;
        let concrete = self.rectangle_copy_program(
            &sources,
            &bg,
            initial.token().ok_or(Error::Incomplete)?,
            next.token().ok_or(Error::Incomplete)?,
        )?;
        concrete
            .provision(b"learned-template")
            .point_to(&program.root);
        let mut w = self.prepare_rectangle_copy(input, &concrete, sources)?;
        self.frozen = true;
        w.execute(w.budget)?;
        w.record(s)
    }
}
