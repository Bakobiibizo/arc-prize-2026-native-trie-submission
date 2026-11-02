//! Prototype: native boundary roles and original-vs-footprint count proof.
//! No decoded dimensions/cardinal comparison or host pixel/region classifier.
use super::*;
use crate::{
    complete_count_witness::{CompleteCountProjection, CompleteCountWitness},
    reference_set_membership::SetMembershipWitness,
    region_witness::RegionWitness,
};
use std::collections::BTreeSet;
pub struct RectangleProjection {
    pub proof: Address,
    pub status: Address,
    pub(super) input: Grid,
    pub(super) owner: Address,
    _certificate: CompleteCountProjection,
    _owners: Vec<Address>,
    bindings: Vec<(Address, Address)>,
    shape: Vec<(Address, Vec<Address>)>,
}
impl RectangleProjection {
    pub(super) fn check_bindings(&self) -> Result<(), Error> {
        for (slot, value) in &self.bindings {
            if !slot.follow()?.same(value) {
                return Err(Error::Incomplete);
            }
        }
        let rows = self
            .proof
            .child(b"source-rows")
            .map_or_else(Vec::new, |r| r.inspect_children());
        if rows.len() != self.shape.len() {
            return Err(Error::Incomplete);
        }
        for (row, (bound, cells)) in rows.iter().zip(&self.shape) {
            let actual = field(row, b"cells")?.inspect_children();
            if !row.same(bound)
                || actual.len() != cells.len()
                || !actual.iter().zip(cells).all(|(a, b)| a.same(b))
            {
                return Err(Error::Incomplete);
            }
        }
        Ok(())
    }
}
impl SparseRewrite {
    pub(super) fn rectangle_observe(&mut self) -> Result<(), Error> {
        if self.rectangle_observations.is_multiple_of(128) {
            self.guard((128 * 160 + 900 * 24) * 640)?;
        }
        self.rectangle_observations += 1;
        Ok(())
    }
    fn rectangle_boundary(
        &mut self,
        input: &Grid,
        region: &Address,
        edge: &Address,
        at: &Address,
    ) -> Result<Address, Error> {
        let neighbour = field(edge, b"source-neighbor")?.follow()?;
        let view = RegionWitness::recorded_view(region, &neighbour).ok_or(Error::Incomplete)?;
        at.provision(b"native-edge").point_to(edge);
        at.provision(b"native-neighbour-view").point_to(&view);
        let class_slot = field(&view, b"class")?;
        let class = class_slot.follow()?;
        let classes = self
            .root
            .provision(b"native-components")
            .provision(b"class-tags");
        let scope = self
            .root
            .provision(b"rectangle-boundary-facts")
            .provision(&class.identity_key());
        at.provision(b"edge-class-slot").point_to(&class_slot);
        at.provision(b"class-at-bind").point_to(&class);
        let key = scope.identity_key().to_vec();
        if let Some(p) = self.rectangle_boundaries.get(&key) {
            if !p.proof.child(b"committed").is_some_and(|n| n.is_terminal()) {
                return Err(Error::Incomplete);
            }
            at.provision(b"native-membership-proof").point_to(&p.proof);
            return Ok(field(&p.proof, b"value")?.follow()?);
        }
        let set = self.root.provision(b"rectangle-boundary-reference-set");
        for tag in [b"barrier".as_slice(), b"outside"] {
            let t = classes.provision(tag);
            records::record(&set, b"reference", &t, &t)?;
        }
        let slot = scope.provision(b"class-source");
        slot.point_to(&class);
        let guards = vec![(slot.clone(), classes.provision(b"unknown"))];
        self.guard(SetMembershipWitness::reserve_bytes(
            &set,
            b"reference",
            guards.len(),
        ))?;
        let parent = self.root.provision(b"rectangle-boundary-memberships");
        let mut w = SetMembershipWitness::prepare(
            set,
            b"reference",
            &slot,
            &guards,
            &parent,
            &class.identity_key(),
            vec![input.clone()],
        )?;
        self.membership_proofs += 1;
        let done = w.execute(w.budget);
        self.membership_copies += w.machine.cycles;
        done?;
        let p = w.record()?.retain_owners(self.owned_roots().to_vec());
        at.provision(b"native-membership-proof").point_to(&p.proof);
        let result = p.value.clone();
        self.rectangle_boundaries.insert(key, p);
        Ok(result)
    }
    fn rectangle_member(
        &mut self,
        region: &Address,
        cell: &Address,
        at: &Address,
    ) -> Result<Option<bool>, Error> {
        let Some(view) = RegionWitness::recorded_view(region, cell) else {
            return Err(Error::Incomplete);
        };
        let class = field(&view, b"class")?.follow()?;
        let tags = self
            .root
            .provision(b"native-components")
            .provision(b"class-tags");
        let (unknown, p) = self.context_equality_fact(&class, &tags.provision(b"unknown"))?;
        at.provision(b"native-unknown-comparison").point_to(&p);
        at.provision(b"native-source-view").point_to(&view);
        at.provision(b"source-class-at-bind").point_to(&class);
        if unknown.same(&self.yes) {
            return Ok(None);
        }
        let (member, p) = self.context_equality_fact(&class, &tags.provision(b"member"))?;
        at.provision(b"native-member-comparison").point_to(&p);
        Ok(Some(member.same(&self.yes)))
    }
    fn finish_rectangle(
        &mut self,
        input: &Grid,
        p: CompleteCountProjection,
        proof: Address,
        status: &[u8],
    ) -> Result<RectangleProjection, Error> {
        let tag = self.root.provision(b"rectangle-tags").provision(status);
        let source = proof.provision(b"status-source");
        source.point_to(&tag);
        self.emit(&source, &proof.provision(b"status"))?;
        proof.provision(b"committed").terminate();
        let mut bindings = Vec::new();
        let mut shape = Vec::new();
        for key in [
            b"input".as_slice(),
            b"region-proof",
            b"complete-count-proof",
            b"status",
        ] {
            let slot = field(&proof, key)?;
            bindings.push((slot.clone(), slot.follow()?));
        }
        let region = field(&proof, b"region-proof")?.follow()?;
        for view in field(&region, b"views")?.inspect_children() {
            let slot = field(&view, b"class")?;
            bindings.push((slot.clone(), slot.follow()?));
            let cell = field(&view, b"source")?.follow()?;
            if let Some(slot) = cell.child(b"value") {
                bindings.push((slot.clone(), slot.follow()?));
            }
        }
        for key in [b"original-count-proof".as_slice(), b"covered-count-proof"] {
            if let Some(slot) = proof.child(key) {
                let producer = slot.follow()?;
                bindings.push((slot, producer.clone()));
                let value = field(&producer, b"value")?;
                bindings.push((value.clone(), value.follow()?));
            }
        }
        if let Some(rows) = proof.child(b"source-rows") {
            for row in rows.inspect_children() {
                let cells = field(&row, b"cells")?.inspect_children();
                for cell in &cells {
                    for key in [b"source-cell".as_slice(), b"source-value-slot"] {
                        if let Some(slot) = cell.child(key) {
                            bindings.push((slot.clone(), slot.follow()?));
                        }
                    }
                }
                shape.push((row, cells));
            }
        }
        Ok(RectangleProjection {
            proof,
            status: tag,
            input: input.clone(),
            owner: self.root.clone(),
            _certificate: p,
            _owners: self.owned_roots().to_vec(),
            bindings,
            shape,
        })
    }
    pub fn rectangle_source(
        &mut self,
        input: &Grid,
        seed: &Address,
    ) -> Result<RectangleProjection, Error> {
        if !input.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        let (count_proof, value) = self.complete_count_value(input, seed)?;
        let native = self.root.provision(b"native-components");
        let retained = CompleteCountWitness::projection_from_committed(
            &self.root,
            &native,
            input,
            &count_proof,
        )?;
        let region = field(&count_proof, b"region-proof")?.follow()?;
        let views = field(&region, b"views")?.inspect_children();
        self.guard((128 * (views.len() + 1) + 512) * 640)?;
        let proof = self
            .root
            .provision(b"rectangle-proofs")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        proof.provision(b"input").point_to(&input.anchor);
        proof
            .provision(b"complete-count-proof")
            .point_to(&count_proof);
        proof.provision(b"region-proof").point_to(&region);
        let unavailable = self
            .root
            .provision(b"complete-count-tags")
            .provision(b"unavailable");
        let (known, fact) = self.property_knownness(&count_proof, &value, &unavailable)?;
        proof.provision(b"native-completeness").point_to(&fact);
        if !known.same(&self.yes) {
            return self.finish_rectangle(input, retained, proof, b"unavailable");
        }
        let members = proof.provision(b"original-member-set");
        let boundary_tags = self
            .root
            .provision(b"rectangle-boundary-memberships")
            .provision(b"set-membership-tags");
        let present = boundary_tags.provision(b"present");
        let unavailable_boundary = boundary_tags.provision(b"unavailable");
        for view in views {
            self.rectangle_observe()?;
            let cell = field(&view, b"source")?.follow()?;
            let at = proof
                .provision(b"member-observations")
                .provision(&cell.identity_key());
            match self.rectangle_member(&region, &cell, &at)? {
                None => return self.finish_rectangle(input, retained, proof, b"unavailable"),
                Some(false) => continue,
                Some(true) => {}
            }
            records::record(&members, b"reference", &cell, &view)?;
            let mut borders = Vec::new();
            for port in PORTS {
                let edge = field(&field(&view, b"edges")?, port)?;
                let tag = self.rectangle_boundary(
                    input,
                    &region,
                    &edge,
                    &at.provision(b"boundaries").provision(port),
                )?;
                if tag.same(&unavailable_boundary) {
                    return self.finish_rectangle(input, retained, proof, b"unavailable");
                }
                borders.push((port, tag));
            }
            for (role, a, b) in [
                (b"NW".as_slice(), b"north".as_slice(), b"west".as_slice()),
                (b"NE", b"north", b"east"),
                (b"SW", b"south", b"west"),
                (b"SE", b"south", b"east"),
            ] {
                let tags = at.provision(b"corner-predicates").provision(role);
                for port in [a, b] {
                    let tag = &borders
                        .iter()
                        .find(|(p, _)| *p == port)
                        .ok_or(Error::Incomplete)?
                        .1;
                    records::record(
                        &tags,
                        b"predicate",
                        tag,
                        &field(&field(&view, b"edges")?, port)?,
                    )?;
                }
                if self.tag(&tags)?.same(&present) {
                    records::record(
                        &proof.provision(b"corners").provision(role),
                        b"reference",
                        &cell,
                        &tags,
                    )?;
                }
            }
        }
        let one = RegionWitness::cardinality_zero(&native)
            .child(b"successor")
            .ok_or(Error::Incomplete)?;
        let mut corners = Vec::new();
        for role in [b"NW".as_slice(), b"NE", b"SW", b"SE"] {
            let set = proof.provision(b"corners").provision(role);
            let (cp, cv) = self.run_set_count(set.clone(), &[], input)?;
            let (tag, comparison) = self.context_equality_fact(&cv, &one)?;
            set.provision(b"native-count-proof").point_to(&cp);
            set.provision(b"native-single-corner").point_to(&comparison);
            if !tag.same(&self.yes) {
                return self.finish_rectangle(input, retained, proof, b"not-rectangle");
            }
            let Selection::Unique { target, .. } = records::resolve(&set, b"reference")? else {
                return Err(Error::Incomplete);
            };
            proof
                .provision(b"corner-roles")
                .provision(role)
                .point_to(&target);
            corners.push(target);
        }
        let covered = proof.provision(b"covered-member-set");
        let mut first_row = Vec::new();
        let mut at = corners[0].clone();
        let mut seen = BTreeSet::new();
        loop {
            self.rectangle_observe()?;
            let fact = proof
                .provision(b"first-row-probes")
                .provision(&at.identity_key());
            match self.rectangle_member(&region, &at, &fact)? {
                None => return self.finish_rectangle(input, retained, proof, b"unavailable"),
                Some(false) => break,
                Some(true) => {}
            }
            if !seen.insert(at.identity_key()) || first_row.len() >= 900 {
                return Err(Error::Incomplete);
            }
            first_row.push(at.clone());
            at = field(&at, b"east")?.follow()?;
        }
        let mut row_start = corners[0].clone();
        let mut row_seen = BTreeSet::new();
        let mut row_index = 0usize;
        loop {
            self.rectangle_observe()?;
            let fact = proof
                .provision(b"row-start-probes")
                .provision(&row_start.identity_key());
            match self.rectangle_member(&region, &row_start, &fact)? {
                None => return self.finish_rectangle(input, retained, proof, b"unavailable"),
                Some(false) => break,
                Some(true) => {}
            }
            if !row_seen.insert(row_start.identity_key()) || row_index >= 900 {
                return Err(Error::Incomplete);
            }
            let row = proof
                .provision(b"source-rows")
                .provision(&row_index.to_be_bytes());
            let mut source = row_start.clone();
            for (i, _) in first_row.iter().enumerate() {
                self.rectangle_observe()?;
                let at = row.provision(b"cells").provision(&i.to_be_bytes());
                match self.rectangle_member(&region, &source, &at)? {
                    None => return self.finish_rectangle(input, retained, proof, b"unavailable"),
                    Some(false) => {
                        return self.finish_rectangle(input, retained, proof, b"not-rectangle");
                    }
                    Some(true) => {}
                }
                at.provision(b"source-cell").point_to(&source);
                at.provision(b"source-value-slot")
                    .point_to(&field(&source, b"value")?);
                records::record(&covered, b"reference", &source, &at)?;
                source = field(&source, b"east")?.follow()?;
            }
            let end = row.provision(b"right-boundary");
            match self.rectangle_member(&region, &source, &end)? {
                None => return self.finish_rectangle(input, retained, proof, b"unavailable"),
                Some(true) => {
                    return self.finish_rectangle(input, retained, proof, b"not-rectangle");
                }
                Some(false) => {}
            }
            row_start = field(&row_start, b"south")?.follow()?;
            row_index += 1;
        }
        let (op, ov) = self.run_set_count(members, &[], input)?;
        let (cp, cv) = self.run_set_count(covered, &[], input)?;
        proof.provision(b"original-count-proof").point_to(&op);
        proof.provision(b"covered-count-proof").point_to(&cp);
        let (equal, fact) = self.context_equality_fact(&ov, &cv)?;
        proof.provision(b"native-coverage-equality").point_to(&fact);
        if !equal.same(&self.yes) {
            return self.finish_rectangle(input, retained, proof, b"not-rectangle");
        }
        self.guard(0)?;
        self.finish_rectangle(input, retained, proof, b"rectangle")
    }
}
