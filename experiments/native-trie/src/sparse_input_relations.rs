//! Opt-in input-input selection and independent literal action. Roles are actual trie
//! references; their keys readdress bound input slots, not numeric semantic opcodes.
use super::*;
#[derive(Clone)]
pub(super) struct Relation {
    a: Address,
    b: Address,
    expected: Address,
}
#[derive(Clone)]
pub(super) struct InputSelection {
    pub(super) body: Address,
}
impl SparseRewrite {
    fn bind_input_values(&self, cell: &Address, at: &Address) -> Result<Vec<Address>, Error> {
        let bank = self.root.provision(b"input-roles");
        let mut roles = vec![bank.provision(b"centre")];
        let mut slots = vec![field(cell, b"value")?];
        for p in PORTS {
            roles.push(bank.provision(p));
            slots.push(self.neighbour(cell, p)?);
        }
        for (role, slot) in roles.iter().zip(slots) {
            let value = at.provision(b"values").provision(&role.identity_key());
            value.point_to(&slot.follow()?);
            value.provision(b"source-slot").point_to(&slot);
        }
        Ok(roles)
    }
    fn input_value(at: &Address, role: &Address) -> Result<Address, Error> {
        Ok(field(&field(at, b"values")?, &role.identity_key())?.follow()?)
    }
    fn known_tag(&self, at: &Address, role: &Address) -> Result<Address, Error> {
        self.tag(&field(&field(at, b"known")?, &role.identity_key())?)
    }
    fn known_tags(&mut self, roles: &[Address], at: &Address) -> Result<(), Error> {
        let yes = self.yes.clone();
        let no = self.no.clone();
        for role in roles {
            let value = Self::input_value(at, role)?;
            let anchor = at.provision(b"known").provision(&role.identity_key());
            anchor.provision(b"input-role").point_to(role);
            for bad in [self.outside.clone(), self.unknown.clone()] {
                let (tag, proof) = self.compare(&value, &bad, &no, &yes)?;
                records::record(&anchor, b"predicate", &tag, &proof)?;
            }
        }
        Ok(())
    }
    pub(super) fn predicate_input(
        &mut self,
        cell: &Address,
        program: &InputSelection,
        at: &Address,
    ) -> Result<Address, Error> {
        let mut bindings = Vec::new();
        for clause in program.body.inspect_children() {
            bindings.push((
                clause.clone(),
                field(&clause, b"input-a")?.follow()?,
                field(&clause, b"input-b")?.follow()?,
                field(&clause, b"expected-sign")?.follow()?,
            ));
        }
        if bindings.is_empty() {
            at.provision(b"producer")
                .provision(b"empty-conjunction-identity")
                .terminate();
            return Ok(self.yes.clone());
        }
        let roles = self.bind_input_values(cell, at)?;
        let used: Vec<_> = roles
            .into_iter()
            .filter(|role| {
                bindings
                    .iter()
                    .any(|(_, a, b, _)| a.same(role) || b.same(role))
            })
            .collect();
        self.known_tags(&used, at)?;
        let all = at.provision(b"conjunction");
        for role in &used {
            let tag = self.known_tag(at, role)?;
            records::record(
                &all,
                b"predicate",
                &tag,
                &field(&field(at, b"known")?, &role.identity_key())?,
            )?;
        }
        let yes = self.yes.clone();
        let no = self.no.clone();
        for (clause, left, right, expected) in &bindings {
            let a = Self::input_value(at, left)?;
            let b = Self::input_value(at, right)?;
            let (actual, proof) = self.compare(&a, &b, &yes, &no)?;
            let (satisfied, check) = self.compare(&actual, expected, &yes, &no)?;
            check.provision(b"input-comparison").point_to(&proof);
            check.provision(b"source-clause").point_to(clause);
            check.provision(b"input-a-role").point_to(left);
            check.provision(b"input-b-role").point_to(right);
            check.provision(b"expected-sign").point_to(expected);
            records::record(&all, b"predicate", &satisfied, &check)?;
        }
        self.tag(&all)
    }
    pub fn derive_input_relations(
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
        let episodes = self.root.provision(b"input-derivations");
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
        let literal = self
            .root
            .provision(b"literal-action-sources")
            .provision(&target.identity_key());
        literal.point_to(&target);
        let features = evidence.provision(b"input-features");
        let yes = self.yes.clone();
        let no = self.no.clone();
        let invalid = self
            .root
            .provision(b"tags")
            .provision(b"invalid-input-relation");
        let bank = self.root.provision(b"input-roles");
        let mut role_bank = vec![bank.provision(b"centre")];
        role_bank.extend(PORTS.iter().map(|p| bank.provision(p)));
        for wrapper in frames {
            let pair = wrapper.follow()?;
            let before = field(&pair, b"before")?.follow()?;
            let context = pair.provision(b"input-context");
            let roles = self.bind_input_values(&before, &context)?;
            self.known_tags(&roles, &context)?;
            for (a, left) in roles.iter().enumerate() {
                for right in roles.iter().skip(a + 1) {
                    let at = context
                        .provision(b"validity")
                        .provision(&left.identity_key())
                        .provision(&right.identity_key());
                    for role in [left, right] {
                        let tag = self.known_tag(&context, role)?;
                        records::record(
                            &at,
                            b"predicate",
                            &tag,
                            &field(&field(&context, b"known")?, &role.identity_key())?,
                        )?;
                    }
                    let valid = self.tag(&at)?;
                    let x = Self::input_value(&context, left)?;
                    let y = Self::input_value(&context, right)?;
                    let (actual, proof) = self.compare(&x, &y, &yes, &no)?;
                    let (tag, gate) = self.compare(&valid, &yes, &actual, &invalid)?;
                    gate.provision(b"input-comparison").point_to(&proof);
                    gate.provision(b"changed-pair").point_to(&pair);
                    records::record(
                        &features
                            .provision(&left.identity_key())
                            .provision(&right.identity_key()),
                        b"predicate",
                        &tag,
                        &gate,
                    )?;
                    result.positive_tests += 1;
                }
            }
        }
        let mut relations = Vec::new();
        for (a, left) in role_bank.iter().enumerate() {
            for right in role_bank.iter().skip(a + 1) {
                let feature = features
                    .provision(&left.identity_key())
                    .provision(&right.identity_key());
                let value = self.tag(&feature)?;
                let included = feature.provision(b"included-sign");
                let excluded = feature.provision(b"excluded-sign");
                for expected in [&yes, &no] {
                    let (dest, proof) = self.compare(&value, expected, &included, &excluded)?;
                    records::record(&dest, b"sign", expected, &proof)?;
                }
                if let Selection::Unique {
                    target: expected, ..
                } = records::resolve(&included, b"sign")?
                {
                    relations.push(Relation {
                        a: left.clone(),
                        b: right.clone(),
                        expected,
                    });
                }
            }
        }
        let mut signature = self
            .root
            .provision(b"input-encodings")
            .provision(&colour.identity_key())
            .provision(&target.identity_key());
        for r in &relations {
            signature = signature
                .provision(&r.a.identity_key())
                .provision(&r.b.identity_key())
                .provision(&r.expected.identity_key());
        }
        signature.provision(b"source-colour").point_to(&colour);
        signature
            .provision(b"literal-action-slot")
            .point_to(&literal);
        let body = signature.provision(b"relations");
        for (i, r) in relations.iter().enumerate() {
            let at = body.provision(&i.to_be_bytes());
            at.provision(b"input-a").point_to(&r.a);
            at.provision(b"input-b").point_to(&r.b);
            at.provision(b"expected-sign").point_to(&r.expected);
        }
        evidence
            .provision(b"candidate-encoding")
            .point_to(&signature);
        let program = InputSelection { body };
        let rejected = evidence.provision(b"counterexamples");
        let accepted = evidence.provision(b"accepted-negatives");
        if unchanged.child(b"holds").is_some()
            && let Selection::Unique { candidate, .. } =
                records::intersect(&[Some(&unchanged), Some(&changed)], b"source-colour")?
        {
            let negatives = field(&candidate, b"evidence")?.inspect_children();
            let total = negatives.len();
            for wrapper in negatives {
                let pair = wrapper.follow()?;
                let before = field(&pair, b"before")?.follow()?;
                let tag = self.predicate_input(
                    &before,
                    &program,
                    &pair.provision(b"negative-input-predicate"),
                )?;
                let (dest, proof) = self.compare(&tag, &yes, &rejected, &accepted)?;
                records::record(&dest, b"cell", &before, &proof)?;
                result.negative_checks += 1;
                // A fixed candidate cannot be repaired by further negative observations.
                // Preserve the actual counterexample and all mask facts; explicitly mark
                // remaining predicate checks unperformed, never label them passed.
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
        if !matches!(records::resolve(&rejected, b"cell")?, Selection::Unknown) {
            return Ok(result);
        }
        records::record(&signature, b"support", &signature, &evidence)?;
        evidence.provision(b"encoding").point_to(&signature);
        result.rule = Some(Rule {
            root: signature,
            owner: self.root.clone(),
            ports: Vec::new(),
            input: Some(program),
            component: false,
        });
        self.guard(0)?;
        Ok(result)
    }
}
