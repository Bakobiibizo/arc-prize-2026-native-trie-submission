//! Protocol: docs/native/NATIVE-COMPONENT-SELECTOR-PROTOCOL.md. Input-only native property,
//! independently learned expected reference and output literal; no decoded kind decisions.
#[cfg(test)]
#[path = "sparse_component_cache_tests.rs"]
mod cache_tests;
use super::*;
use crate::region_witness::RegionWitness;
impl SparseRewrite {
    pub(super) fn component_value(
        &mut self,
        input: &Grid,
        cell: &Address,
    ) -> Result<(Address, Address), Error> {
        let owner = self.root.provision(b"native-components");
        let cache = owner
            .provision(b"cache")
            .provision(&input.anchor.identity_key());
        if let Some(pair) = RegionWitness::cached(&cache, cell)? {
            self.region_cache_hits += 1;
            return Ok(pair);
        }
        let n = input.width() * input.height() + 1;
        self.guard(512_000 + n * 38_000)?;
        RegionWitness::cardinality_zero(&owner);
        self.guard(n * 38_000)?;
        let mut w = RegionWitness::prepare_counted_bound(
            input,
            cell,
            &owner,
            &self.region_proofs.to_be_bytes(),
            &cache,
        )?;
        // IDs are local reservation identities. Failed/incomplete proofs are never reused.
        self.region_proofs += 1;
        let execution = w.execute(w.budget);
        self.region_copies += w.machine.cycles;
        execution?;
        let p = w.record()?;
        let pair = (p.proof.clone(), p.value.clone());
        self.guard((128 * 160 + 900 * 24) * 640)?;
        Ok(pair)
    }
    pub(super) fn predicate_component(
        &mut self,
        input: &Grid,
        cell: &Address,
        rule: &Address,
        at: &Address,
    ) -> Result<Address, Error> {
        let (proof, value) = self.component_value(input, cell)?;
        let expected = field(rule, b"expected-kind")?.follow()?;
        at.provision(b"source-component-proof").point_to(&proof);
        at.provision(b"observed-kind").point_to(&value);
        at.provision(b"expected-kind-consumed").point_to(&expected);
        let yes = self.yes.clone();
        let no = self.no.clone();
        let (tag, comparison) = self.compare(&value, &expected, &yes, &no)?;
        at.provision(b"comparison").point_to(&comparison);
        Ok(tag)
    }
    pub fn derive_component_relations(
        &mut self,
        id: &[u8],
        input: &Grid,
        output: &Grid,
    ) -> Result<Derived, Error> {
        if self.frozen || !input.owner.same(&self.space) || !output.owner.same(&self.space) {
            return Err(Error::NotReady);
        }
        if input.width() != output.width() || input.height() != output.height() {
            return Err(Error::Shape);
        }
        self.guard((128 * 160 + 900 * 24) * 640 + id.len())?;
        let episodes = self.root.provision(b"component-derivations");
        if episodes.child(id).is_some() {
            return Err(Error::NotReady);
        }
        let evidence = episodes.provision(id);
        evidence.provision(b"input").point_to(&input.anchor);
        evidence.provision(b"output").point_to(&output.anchor);
        let changed = evidence.provision(b"changed");
        let unchanged = evidence.provision(b"unchanged");
        for y in 0..input.height() {
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let after = output.cell(y, x)?;
                let pair = evidence
                    .provision(b"pairs")
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes());
                pair.provision(b"before").point_to(&before);
                pair.provision(b"after").point_to(&after);
                let a = field(&before, b"value")?.follow()?;
                let b = field(&after, b"value")?.follow()?;
                let (dest, proof) = self.compare(&a, &b, &unchanged, &changed)?;
                pair.provision(b"change-proof").point_to(&proof);
                records::record(&dest, b"source-colour", &a, &pair)?;
            }
        }
        let mut result = Derived {
            evidence: evidence.clone(),
            rule: None,
            changed: 0,
            positive_tests: 0,
            negative_checks: 0,
            negative_unchecked: 0,
        };
        let (candidate, colour) = match records::resolve(&changed, b"source-colour")? {
            Selection::Unique { candidate, target } => (candidate, target),
            _ => return Ok(result),
        };
        if colour.same(&self.unknown) {
            return Ok(result);
        }
        let frames = field(&candidate, b"evidence")?.inspect_children();
        result.changed = frames.len();
        let targets = evidence.provision(b"changed-targets");
        for wrapper in &frames {
            let pair = wrapper.follow()?;
            let after = field(&pair, b"after")?.follow()?;
            let value = field(&after, b"value")?.follow()?;
            records::record(&targets, b"value", &value, &pair)?;
        }
        let target = match records::resolve(&targets, b"value")? {
            Selection::Unique { target, .. } => target,
            _ => return Ok(result),
        };
        if target.same(&self.unknown) {
            return Ok(result);
        }
        let kinds = evidence.provision(b"known-positive-kinds");
        let unavailable = evidence.provision(b"unavailable-positive-kinds");
        for wrapper in frames {
            let pair = wrapper.follow()?;
            let before = field(&pair, b"before")?.follow()?;
            let (proof, value) = self.component_value(input, &before)?;
            pair.provision(b"source-component-proof").point_to(&proof);
            let unknown = self
                .root
                .provision(b"native-components")
                .provision(b"relation-tags")
                .provision(b"unresolved");
            let (dest, gate) = self.compare(&value, &unknown, &unavailable, &kinds)?;
            gate.provision(b"component-proof").point_to(&proof);
            gate.provision(b"changed-pair").point_to(&pair);
            records::record(&dest, b"value", &value, &gate)?;
            result.positive_tests += 1;
        }
        if !matches!(
            records::resolve(&unavailable, b"value")?,
            Selection::Unknown
        ) {
            return Ok(result);
        }
        let expected = match records::resolve(&kinds, b"value")? {
            Selection::Unique { target, .. } => target,
            _ => return Ok(result),
        };
        let literal = self
            .root
            .provision(b"literal-action-sources")
            .provision(&target.identity_key());
        literal.point_to(&target);
        let signature = self
            .root
            .provision(b"component-encodings")
            .provision(&colour.identity_key())
            .provision(&expected.identity_key())
            .provision(&target.identity_key());
        signature.provision(b"source-colour").point_to(&colour);
        signature.provision(b"expected-kind").point_to(&expected);
        signature
            .provision(b"literal-action-slot")
            .point_to(&literal);
        evidence
            .provision(b"candidate-encoding")
            .point_to(&signature);
        let rejected = evidence.provision(b"counterexamples");
        let accepted = evidence.provision(b"accepted-negatives");
        let yes = self.yes.clone();
        if unchanged.child(b"holds").is_some()
            && let Selection::Unique { candidate, .. } =
                records::intersect(&[Some(&unchanged), Some(&changed)], b"source-colour")?
        {
            let negatives = field(&candidate, b"evidence")?.inspect_children();
            let total = negatives.len();
            for wrapper in negatives {
                let pair = wrapper.follow()?;
                let before = field(&pair, b"before")?.follow()?;
                let tag = self.predicate_component(
                    input,
                    &before,
                    &signature,
                    &pair.provision(b"negative-component-predicate"),
                )?;
                let (dest, proof) = self.compare(&tag, &yes, &rejected, &accepted)?;
                records::record(&dest, b"cell", &before, &proof)?;
                result.negative_checks += 1;
                if !matches!(records::resolve(&rejected, b"cell")?, Selection::Unknown) {
                    result.negative_unchecked = total - result.negative_checks;
                    evidence
                        .provision(b"negative-validation-short-circuited")
                        .terminate();
                    evidence
                        .provision(b"negative-unchecked")
                        .provision(&result.negative_unchecked.to_be_bytes());
                    return Ok(result);
                }
            }
        }
        records::record(&signature, b"support", &signature, &evidence)?;
        evidence.provision(b"encoding").point_to(&signature);
        result.rule = Some(Rule {
            root: signature,
            owner: self.root.clone(),
            ports: Vec::new(),
            input: None,
            component: true,
        });
        self.guard(0)?;
        Ok(result)
    }
}
