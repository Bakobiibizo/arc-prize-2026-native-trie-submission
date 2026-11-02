//! NATIVE-CONTEXT-COMPOSITION-PROTOCOL: independent contextual refinement of
//! contradictory count prefixes; never relax a known failed input key.
use super::action_partitions::Row;
use super::source_groups::{candidate, candidates};
use super::*;
use std::collections::BTreeMap;
impl SparseRewrite {
    pub(super) fn compose_count_context(
        &mut self,
        rows: &[Row],
        colour: &Address,
        palette: &Address,
        at: &Address,
    ) -> Result<Option<Vec<Address>>, Error> {
        let mut groups: BTreeMap<_, (Address, Address, Vec<Row>)> = BTreeMap::new();
        let unavailable = self
            .root
            .provision(b"complete-count-tags")
            .provision(b"unavailable");
        for (i, row) in rows.iter().enumerate() {
            if i.is_multiple_of(128) {
                self.guard((128 * 160 + 900 * 24) * 640)?;
            }
            let before = field(&row.pair, b"before")?.follow()?;
            let (region, kind) = self.component_value(&row.input, &before)?;
            let after = field(&row.pair, b"after")?.follow()?;
            let label = field(&after, b"value")?.follow()?;
            let labels = at
                .provision(b"component-labels")
                .provision(&region.identity_key());
            labels.provision(b"producer").point_to(&region);
            records::record(&labels, b"after", &label, &row.pair)?;
            if matches!(
                records::resolve(&labels, b"after")?,
                Selection::Ambiguous { .. }
            ) {
                at.provision(b"same-component-contradiction")
                    .point_to(&labels);
                return Ok(None);
            }
            let (proof, count) = self.complete_count_value(&row.input, &before)?;
            let (known, fact) = self.property_knownness(&proof, &count, &unavailable)?;
            let observation = at
                .provision(b"observations")
                .provision(&row.pair.identity_key());
            records::record(&observation, b"knownness", &known, &fact)?;
            observation.provision(b"pair").point_to(&row.pair);
            observation.provision(b"count-producer").point_to(&proof);
            if candidate(&observation, b"knownness", &self.yes).is_none() {
                if row.changed {
                    return Ok(None);
                }
                continue;
            }
            groups
                .entry((kind.identity_key(), count.identity_key()))
                .or_insert_with(|| (kind, count, Vec::new()))
                .2
                .push(row.clone());
        }
        let mut body = Vec::new();
        for (_, (kind, count, subset)) in groups {
            if !subset.iter().any(|r| r.changed) {
                continue;
            }
            let prefix = at
                .provision(b"count-prefixes")
                .provision(&kind.identity_key())
                .provision(&count.identity_key());
            let labels = prefix.provision(b"after-alternatives");
            let mut per_input = BTreeMap::new();
            for row in &subset {
                let label = field(&field(&row.pair, b"after")?.follow()?, b"value")?.follow()?;
                records::record(&labels, b"value", &label, &row.pair)?;
                let per = per_input
                    .entry(row.input.anchor.identity_key())
                    .or_insert_with(|| {
                        prefix
                            .provision(b"per-input")
                            .provision(&row.input.anchor.identity_key())
                    });
                records::record(per, b"value", &label, &row.pair)?;
            }
            if let Selection::Unique { target, .. } = records::resolve(&labels, b"value")? {
                if target.same(&self.unknown) {
                    return Ok(None);
                }
                let leaf = prefix.provision(b"action");
                leaf.provision(b"source-colour").point_to(colour);
                leaf.provision(b"expected-kind").point_to(&kind);
                leaf.provision(b"expected-count").point_to(&count);
                leaf.provision(b"constraint-prefix").point_to(&prefix);
                let slot = self
                    .root
                    .provision(b"literal-action-sources")
                    .provision(&target.identity_key());
                slot.point_to(&target);
                leaf.provision(b"literal-action-slot").point_to(&slot);
                body.push(leaf);
                continue;
            }
            prefix.provision(b"original-count-key-conflict").terminate();
            let grid_possible = per_input
                .values()
                .all(|p| matches!(records::resolve(p, b"value"), Ok(Selection::Unique { .. })));
            let mut colours: Vec<_> = candidates(palette, b"value")
                .iter()
                .map(|c| c.follow())
                .collect::<Result<_, _>>()?;
            colours.retain(|c| !c.same(&self.unknown));
            colours.sort_by_key(|c| c.token().unwrap_or_default().to_vec());
            let mut roles = Vec::new();
            for c in &colours {
                if grid_possible {
                    let role = self
                        .root
                        .provision(b"context-roles")
                        .provision(b"grid-colour-components")
                        .provision(&c.identity_key());
                    role.provision(b"context-colour").point_to(c);
                    roles.push(role);
                }
            }
            let tags = self
                .root
                .provision(b"native-components")
                .provision(b"relation-tags");
            for c in &colours {
                for k in [tags.provision(b"bounded"), tags.provision(b"escape")] {
                    let role = self
                        .root
                        .provision(b"context-roles")
                        .provision(b"adjacent-colour-kind-components")
                        .provision(&c.identity_key())
                        .provision(&k.identity_key());
                    role.provision(b"context-colour").point_to(c);
                    role.provision(b"neighbour-kind").point_to(&k);
                    roles.push(role);
                }
            }
            let mut best: Option<Vec<Address>> = None;
            for role in roles {
                let alternative = prefix
                    .provision(b"alternatives")
                    .provision(&role.identity_key());
                alternative.provision(b"role").point_to(&role);
                if let Some(leaves) =
                    self.learn_context_column(&subset, colour, &role, &alternative)?
                    && best.as_ref().is_none_or(|b| leaves.len() < b.len())
                {
                    for leaf in &leaves {
                        leaf.provision(b"expected-kind").point_to(&kind);
                        leaf.provision(b"expected-count").point_to(&count);
                        leaf.provision(b"base-prefix").point_to(&prefix);
                    }
                    best = Some(leaves);
                }
                // Every changing label needs at least one literal-action leaf.
                let targets = prefix.provision(b"changing-targets");
                for row in subset.iter().filter(|r| r.changed) {
                    let label =
                        field(&field(&row.pair, b"after")?.follow()?, b"value")?.follow()?;
                    records::record(&targets, b"value", &label, &row.pair)?;
                }
                if best
                    .as_ref()
                    .is_some_and(|b| b.len() == candidates(&targets, b"value").len())
                {
                    break;
                }
            }
            let Some(leaves) = best else {
                prefix.provision(b"no-context-refinement").terminate();
                return Ok(None);
            };
            body.extend(leaves);
        }
        at.provision(b"all-prefixes-refined-consistently")
            .terminate();
        Ok(Some(body))
    }
}
