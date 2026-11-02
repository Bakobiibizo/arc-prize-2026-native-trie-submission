//! NATIVE-RELATIVE-ACTION-LEARNING-PROTOCOL: native colour-selected universe,
//! retained complete counts, strict ordinal proofs, then native relative count.
use super::action_partitions::Row;
use super::source_groups::candidates;
use super::*;
use crate::{
    complete_count_witness::CompleteCountWitness,
    ordinal_relation::OrdinalInput,
    region_witness::{RegionError, RegionWitness},
    relative_ordinal_count::{OrdinalCandidate, RelativeOrdinalCount, RelativeOrdinalError},
};
use std::{cell::Cell, rc::Rc};
impl SparseRewrite {
    fn retained_count_input(
        &mut self,
        input: &Grid,
        cell: &Address,
    ) -> Result<Rc<OrdinalInput>, Error> {
        let (proof, _) = self.complete_count_value(input, cell)?;
        let key = proof.identity_key().to_vec();
        if let Some(value) = self.relative_inputs.get(&key) {
            value.current()?;
            return Ok(value.clone());
        }
        let native = self.root.provision(b"native-components");
        let p =
            CompleteCountWitness::projection_from_committed(&self.root, &native, input, &proof)?;
        let missing = self
            .root
            .provision(b"complete-count-tags")
            .provision(b"unavailable");
        let v = OrdinalInput::from_complete(p, missing);
        self.relative_inputs.insert(key, v.clone());
        Ok(v)
    }
    pub(super) fn relative_value(
        &mut self,
        input: &Grid,
        cell: &Address,
        role: &Address,
    ) -> Result<(Address, Address), Error> {
        if role.child(b"neighbour-kind").is_some() {
            return Err(Error::Incomplete);
        }
        let colour = field(role, b"context-colour")?.follow()?;
        let (region, _) = self.component_value(input, cell)?;
        let subject = self.retained_count_input(input, cell)?;
        let (universe, universe_value) = self.context_value(input, cell, role)?;
        let native = self.root.provision(b"native-components");
        let missing = native
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let mut cache = self
            .root
            .provision(b"relative-value-index")
            .provision(&input.anchor.identity_key())
            .provision(&region.identity_key())
            .provision(&subject.current()?.identity_key())
            .provision(&colour.identity_key())
            .provision(&universe.identity_key())
            .provision(&universe_value.identity_key());
        let mut selected = Vec::new();
        if !self.context_equal(&universe_value, &missing)? {
            let set = field(&universe, b"source-set")?.follow()?;
            for c in candidates(&set, b"reference") {
                if self.relative_observations.is_multiple_of(128) {
                    self.guard((128 * 160 + 900 * 24) * 640)?;
                }
                self.relative_observations += 1;
                let reference = c.follow()?;
                if !reference
                    .child(b"committed")
                    .is_some_and(|n| n.is_terminal())
                    || !field(&reference, b"input")?.follow()?.same(&input.anchor)
                {
                    return Err(Error::Incomplete);
                }
                let seed = field(&reference, b"seed")?.follow()?;
                let (actual, _) = self.component_value(input, &seed)?;
                if !actual.same(&reference) {
                    return Err(Error::Incomplete);
                }
                let count = self.retained_count_input(input, &seed)?;
                if !field(&count.proof, b"region-proof")?
                    .follow()?
                    .same(&reference)
                {
                    return Err(Error::Incomplete);
                }
                cache = cache
                    .provision(&reference.identity_key())
                    .provision(&count.proof.identity_key())
                    .provision(&count.current()?.identity_key());
                selected.push(OrdinalCandidate { reference, count });
            }
        }
        let key = cache.identity_key().to_vec();
        if let Some(value) = self.relative_values.get(&key) {
            return Ok((value.proof.clone(), value.current()?));
        }
        cache.provision(b"subject").point_to(&region);
        cache.provision(b"candidate-universe").point_to(&universe);
        cache.provision(b"colour-at-bind").point_to(&colour);
        cache.provision(b"input").point_to(&input.anchor);
        self.guard(512000)?;
        let zero = RegionWitness::cardinality_zero(&native);
        let mut factory = self
            .relative_factory
            .take()
            .unwrap_or_else(|| RelativeOrdinalCount::new(&native, &zero));
        let limited = Cell::new(false);
        // Reserve the unchanged constructor bounds against a conservative upper
        // estimate. Each accepted preparation debits its entire reserve; refresh
        // all FIVE actual roots only when that pessimistic estimate would fail.
        // No allocation is authorized beyond the original 286720000 envelope.
        let upper = Cell::new(self.workspace_bytes());
        let result = factory.prepare(
            subject,
            &selected,
            &[(field(&universe, b"value")?, missing.clone())],
            &self.serial.to_be_bytes(),
            |reserve| {
                let mut charged = upper.get();
                if charged.saturating_add(reserve) > 286720000 {
                    charged = self.workspace_bytes();
                }
                if charged.saturating_add(reserve) > 286720000 {
                    limited.set(true);
                    return Err(RegionError::Limit);
                }
                upper.set(charged.saturating_add(reserve));
                Ok(())
            },
        );
        self.serial += 1;
        self.relative_factory = Some(factory);
        let projection = match result {
            Ok(p) => p,
            Err(_) if limited.get() => return Err(Error::WorkspaceLimit),
            Err(RelativeOrdinalError::Region(e)) => return Err(e.into()),
            Err(RelativeOrdinalError::Record(e)) => return Err(e.into()),
        };
        let value = OrdinalInput::from_relative(projection, missing);
        cache
            .provision(b"relative-count-proof")
            .point_to(&value.proof);
        let out = (value.proof.clone(), value.current()?);
        self.relative_values.insert(key, value);
        self.guard((128 * 160 + 900 * 24) * 640)?;
        Ok(out)
    }
    pub(super) fn relative_alternative(
        &mut self,
        rows: &[Row],
        colour: &Address,
        palette: &Address,
        at: &Address,
    ) -> Result<Option<Vec<Address>>, Error> {
        // Every offered property is component-constant. Native component identity plus
        // actual teacher labels is a decisive impossibility certificate, not a heuristic.
        for (i, row) in rows.iter().enumerate() {
            if i.is_multiple_of(128) {
                self.guard((128 * 160 + 900 * 24) * 640)?;
            }
            let cell = field(&row.pair, b"before")?.follow()?;
            let (proof, _) = self.component_value(&row.input, &cell)?;
            let labels = at
                .provision(b"component-labels")
                .provision(&proof.identity_key());
            let label = field(&field(&row.pair, b"after")?.follow()?, b"value")?.follow()?;
            records::record(&labels, b"after", &label, &row.pair)?;
            if matches!(
                records::resolve(&labels, b"after")?,
                Selection::Ambiguous { .. }
            ) {
                at.provision(b"same-component-contradiction")
                    .point_to(&labels);
                return Ok(None);
            }
        }
        let mut colours: Vec<_> = candidates(palette, b"value")
            .iter()
            .map(|c| c.follow())
            .collect::<Result<_, _>>()?;
        colours.retain(|c| !c.same(&self.unknown));
        colours.sort_by_key(|c| {
            (
                usize::from(!c.same(colour)),
                c.token().unwrap_or_default().to_vec(),
            )
        });
        let mut best: Option<Vec<Address>> = None;
        let targets = at.provision(b"changing-targets");
        for row in rows.iter().filter(|r| r.changed) {
            let t = field(&field(&row.pair, b"after")?.follow()?, b"value")?.follow()?;
            records::record(&targets, b"value", &t, &row.pair)?;
        }
        let lower = candidates(&targets, b"value").len();
        for c in colours {
            let role = self
                .root
                .provision(b"relative-roles")
                .provision(&c.identity_key());
            role.provision(b"context-colour").point_to(&c);
            let alternative = at.provision(b"alternatives").provision(&c.identity_key());
            alternative.provision(b"role").point_to(&role);
            if let Some(body) = self.learn_relative_column(rows, colour, &role, &alternative)?
                && best.as_ref().is_none_or(|b| body.len() < b.len())
            {
                best = Some(body)
            }
            if best.as_ref().is_some_and(|b| b.len() == lower) {
                break;
            }
        }
        Ok(best)
    }
    pub(super) fn predicate_relative(
        &mut self,
        input: &Grid,
        cell: &Address,
        group: &Address,
        at: &Address,
    ) -> Result<Address, Error> {
        let role = field(group, b"relative-role")?.follow()?;
        let (producer, value) = self.relative_value(input, cell, &role)?;
        let missing = self
            .root
            .provision(b"native-components")
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let (known, fact) = self.property_knownness(&producer, &value, &missing)?;
        let expected = field(group, b"expected-relative-count")?.follow()?;
        let yes = self.yes.clone();
        let no = self.no.clone();
        let (tag, proof) = self.compare(&value, &expected, &yes, &no)?;
        proof.provision(b"relative-producer").point_to(&producer);
        proof.provision(b"relative-count-consumed").point_to(&value);
        proof
            .provision(b"expected-relative-count-consumed")
            .point_to(&expected);
        proof.provision(b"relative-role-consumed").point_to(&role);
        let all = at.provision(b"predicates");
        records::record(&all, b"predicate", &known, &fact)?;
        records::record(&all, b"predicate", &tag, &proof)?;
        self.tag(&all)
    }
    pub fn relative_statistics(&self) -> (usize, u64, u64) {
        self.relative_factory.as_ref().map_or((0, 0, 0), |f| {
            (f.bank_count(), f.comparison_copies, f.count_copies)
        })
    }
}
