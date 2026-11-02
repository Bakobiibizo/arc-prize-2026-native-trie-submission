//! Compare consistent input-property constructions, not whole-grid mappings.
//! Objective: number of learned action leaves; producer/proof costs are separate.
use super::action_partitions::{Prefix, Row};
use super::source_groups::{candidate, candidates, frames};
use super::*;
use std::collections::BTreeMap;
impl SparseRewrite {
    pub(super) fn learn_context_column(
        &mut self,
        rows: &[Row],
        colour: &Address,
        role: &Address,
        at: &Address,
    ) -> Result<Option<Vec<Address>>, Error> {
        self.learn_count_column(rows, colour, role, at, false)
    }
    pub(super) fn learn_relative_column(
        &mut self,
        rows: &[Row],
        colour: &Address,
        role: &Address,
        at: &Address,
    ) -> Result<Option<Vec<Address>>, Error> {
        self.learn_count_column(rows, colour, role, at, true)
    }
    fn learn_count_column(
        &mut self,
        rows: &[Row],
        colour: &Address,
        role: &Address,
        at: &Address,
        relative: bool,
    ) -> Result<Option<Vec<Address>>, Error> {
        let unavailable = self
            .root
            .provision(b"native-components")
            .provision(b"set-count-tags")
            .provision(b"unavailable");
        let mut buckets: BTreeMap<_, (Address, Vec<&Row>)> = BTreeMap::new();
        for (i, row) in rows.iter().enumerate() {
            if i.is_multiple_of(128) {
                self.guard((128 * 160 + 900 * 24) * 640)?;
            }
            let before = field(&row.pair, b"before")?.follow()?;
            let (proof, value) = if relative {
                self.relative_value(&row.input, &before, role)?
            } else {
                self.context_value(&row.input, &before, role)?
            };
            let (known, fact) = self.property_knownness(&proof, &value, &unavailable)?;
            let observed = at
                .provision(b"observations")
                .provision(&row.pair.identity_key());
            observed.provision(b"producer").point_to(&proof);
            observed.provision(b"value").point_to(&value);
            observed.provision(b"pair").point_to(&row.pair);
            records::record(&observed, b"knownness", &known, &fact)?;
            if candidate(&observed, b"knownness", &self.yes).is_none() {
                if row.changed {
                    at.provision(b"unavailable-positive").point_to(&observed);
                    return Ok(None);
                }
                continue;
            }
            let node = at.provision(b"values").provision(&value.identity_key());
            node.provision(b"value").point_to(&value);
            let after = field(&row.pair, b"after")?.follow()?;
            let label = field(&after, b"value")?.follow()?;
            records::record(
                &node.provision(b"after-alternatives"),
                b"value",
                &label,
                &row.pair,
            )?;
            let entry = buckets
                .entry(value.identity_key())
                .or_insert_with(|| (value, Vec::new()));
            entry.1.push(row);
            if entry.1.iter().any(|r| r.changed)
                && !matches!(
                    records::resolve(&node.provision(b"after-alternatives"), b"value")?,
                    Selection::Unique { .. }
                )
            {
                at.provision(b"known-key-conflict").point_to(&node);
                return Ok(None);
            }
        }
        let mut body = Vec::new();
        for (_, (value, subset)) in buckets {
            if !subset.iter().any(|r| r.changed) {
                continue;
            }
            let node = at.provision(b"values").provision(&value.identity_key());
            let target = match records::resolve(&node.provision(b"after-alternatives"), b"value")? {
                Selection::Unique { target, .. } => target,
                _ => return Ok(None),
            };
            if target.same(&self.unknown) {
                return Ok(None);
            }
            let leaf = node.provision(b"action");
            leaf.provision(b"source-colour").point_to(colour);
            leaf.provision(if relative {
                b"relative-role"
            } else {
                b"context-role"
            })
            .point_to(role);
            leaf.provision(if relative {
                b"expected-relative-count"
            } else {
                b"expected-context-count"
            })
            .point_to(&value);
            leaf.provision(b"constraint-prefix").point_to(&node);
            let slot = self
                .root
                .provision(b"literal-action-sources")
                .provision(&target.identity_key());
            slot.point_to(&target);
            leaf.provision(b"literal-action-slot").point_to(&slot);
            body.push(leaf);
        }
        at.provision(b"all-observed-labels-consistent").terminate();
        Ok(Some(body))
    }
    pub fn agree_context_actions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        self.agree_context_mode(lessons, false, false)
    }
    pub fn agree_context_composition(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        self.agree_context_mode(lessons, true, false)
    }
    pub fn agree_relative_actions(
        &mut self,
        lessons: &[GroupLesson],
    ) -> Result<Option<GroupProgram>, Error> {
        self.agree_context_mode(lessons, true, true)
    }
    fn agree_context_mode(
        &mut self,
        lessons: &[GroupLesson],
        composition: bool,
        relative: bool,
    ) -> Result<Option<GroupProgram>, Error> {
        if self.frozen || lessons.is_empty() || lessons.iter().any(|d| !d.owner.same(&self.root)) {
            return Err(Error::NotReady);
        }
        self.guard((128 * 160 + 900 * 24) * 640)?;
        let assembly = self
            .root
            .provision(b"context-action-assemblies")
            .provision(&self.serial.to_be_bytes());
        self.serial += 1;
        let sources = assembly.provision(b"changing-sources");
        let palette = assembly.provision(b"input-colours");
        for (i, d) in lessons.iter().enumerate() {
            assembly
                .provision(b"lessons")
                .provision(&i.to_be_bytes())
                .point_to(&d.evidence);
            for c in candidates(&field(&d.evidence, b"changed")?, b"source-colour") {
                records::record(&sources, b"source", &c.follow()?, &c)?;
            }
            for head in [b"changed".as_slice(), b"unchanged"] {
                for c in candidates(&field(&d.evidence, head)?, b"source-colour") {
                    records::record(&palette, b"value", &c.follow()?, &c)?;
                }
            }
        }
        let mut whole = Vec::new();
        for source in candidates(&sources, b"source") {
            let colour = source.follow()?;
            if colour.same(&self.unknown) {
                return Ok(None);
            }
            let node = assembly
                .provision(b"source-prefixes")
                .provision(&colour.identity_key());
            let mut rows = Vec::new();
            let changing_labels = node.provision(b"changing-labels");
            let mut grid_possible = true;
            for d in lessons {
                let positives = frames(candidate(
                    &field(&d.evidence, b"changed")?,
                    b"source-colour",
                    &colour,
                ))?;
                let negatives = frames(candidate(
                    &field(&d.evidence, b"unchanged")?,
                    b"source-colour",
                    &colour,
                ))?;
                let per = node
                    .provision(b"per-input-labels")
                    .provision(&d.input.anchor.identity_key());
                for (pair, changed) in positives
                    .into_iter()
                    .map(|p| (p, true))
                    .chain(negatives.into_iter().map(|p| (p, false)))
                {
                    let after = field(&pair, b"after")?.follow()?;
                    let value = field(&after, b"value")?.follow()?;
                    records::record(&per, b"value", &value, &pair)?;
                    if changed {
                        records::record(&changing_labels, b"value", &value, &pair)?;
                    }
                    rows.push(Row {
                        input: d.input.clone(),
                        pair,
                        changed,
                    });
                }
                if matches!(
                    records::resolve(&per, b"value")?,
                    Selection::Ambiguous { .. }
                ) {
                    grid_possible = false;
                }
            }
            let lower = candidates(&changing_labels, b"value").len();
            let mut baseline = Vec::new();
            let fits = self.refine_prefix(
                rows.clone(),
                Prefix {
                    colour: colour.clone(),
                    kind: None,
                    count: None,
                },
                &node.provision(b"baseline"),
                &mut baseline,
            )?;
            let mut best = if fits { Some(baseline) } else { None };
            let mut best_size = best.as_ref().map_or(usize::MAX, Vec::len);
            node.provision(b"distinct-changing-label-lower-bound")
                .provision(&lower.to_be_bytes());
            // A single changing action is left to the existing size/source-binding
            // generalizer. Context proposals address MULTIPLE changing actions only.
            if lower > 1 && best_size > lower {
                let mut colours: Vec<_> = candidates(&palette, b"value")
                    .iter()
                    .map(|c| c.follow())
                    .collect::<Result<_, _>>()?;
                colours.retain(|c| !c.same(&self.unknown));
                colours.sort_by_key(|c| {
                    let other = candidate(&sources, b"source", c).is_some() && !c.same(&colour);
                    (
                        if other {
                            0
                        } else if c.same(&colour) {
                            2
                        } else {
                            1
                        },
                        c.token().unwrap_or_default().to_vec(),
                    )
                });
                let mut roles = Vec::new();
                if grid_possible {
                    for c in &colours {
                        let role = self
                            .root
                            .provision(b"context-roles")
                            .provision(b"grid-colour-components")
                            .provision(&c.identity_key());
                        role.provision(b"context-colour").point_to(c);
                        roles.push(role);
                    }
                } else {
                    node.provision(b"grid-constant-field-cannot-separate-input-labels")
                        .terminate();
                }
                let tags = self
                    .root
                    .provision(b"native-components")
                    .provision(b"relation-tags");
                for c in &colours {
                    for kind in [tags.provision(b"bounded"), tags.provision(b"escape")] {
                        let role = self
                            .root
                            .provision(b"context-roles")
                            .provision(b"adjacent-colour-kind-components")
                            .provision(&c.identity_key())
                            .provision(&kind.identity_key());
                        role.provision(b"context-colour").point_to(c);
                        role.provision(b"neighbour-kind").point_to(&kind);
                        roles.push(role);
                    }
                }
                for role in roles {
                    let at = node
                        .provision(b"alternatives")
                        .provision(&role.identity_key());
                    at.provision(b"role").point_to(&role);
                    if let Some(candidate) =
                        self.learn_context_column(&rows, &colour, &role, &at)?
                    {
                        at.provision(b"action-leaves")
                            .provision(&candidate.len().to_be_bytes());
                        if candidate.len() < best_size {
                            best_size = candidate.len();
                            best = Some(candidate);
                            node.provision(b"selected-property").point_to(&role);
                        }
                    }
                    if best_size == lower {
                        node.provision(b"attained-action-label-lower-bound")
                            .terminate();
                        break;
                    }
                }
            }
            if composition && best.is_none() {
                best = self.compose_count_context(
                    &rows,
                    &colour,
                    &palette,
                    &node.provision(b"composed-alternative"),
                )?;
            }
            if relative
                && lower > 1
                && best.as_ref().is_none_or(|b| b.len() > lower)
                && let Some(body) = self.relative_alternative(
                    &rows,
                    &colour,
                    &palette,
                    &node.provision(b"relative-alternative"),
                )?
                && best.as_ref().is_none_or(|b| body.len() < b.len())
            {
                best = Some(body)
            }
            let Some(body) = best else {
                node.provision(b"no-consistent-construction").terminate();
                return Ok(None);
            };
            whole.extend(body);
        }
        let mut encoding = self.root.provision(b"context-action-encodings");
        for leaf in &whole {
            encoding = encoding.provision(&field(leaf, b"source-colour")?.follow()?.identity_key());
            for k in [
                b"expected-kind".as_slice(),
                b"expected-count",
                b"context-role",
                b"expected-context-count",
                b"relative-role",
                b"expected-relative-count",
            ] {
                encoding = if let Some(s) = leaf.child(k) {
                    encoding.provision(k).provision(&s.follow()?.identity_key())
                } else {
                    encoding.provision(k).provision(b"absent")
                };
            }
            encoding = encoding.provision(
                &field(leaf, b"literal-action-slot")?
                    .follow()?
                    .follow()?
                    .identity_key(),
            );
        }
        let program = encoding.provision(b"program");
        let groups = program.provision(b"groups");
        for (i, leaf) in whole.iter().enumerate() {
            groups.provision(&i.to_be_bytes()).point_to(leaf);
        }
        assembly.provision(b"encoding").point_to(&program);
        self.guard(0)?;
        let p = GroupProgram {
            root: program,
            owner: self.root.clone(),
        };
        Ok(Some(self.generalize_source_bindings(lessons, p)?))
    }
}
