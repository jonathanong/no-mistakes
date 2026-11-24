use super::super::extract_model::{FileFacts, JobOptionEvidence, JobOptionSpread};
use super::super::object_bindings::ObjectBindings;
use super::traversal::DeclarationTraversal;
use crate::fx::FxHashMap;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};

pub(in crate::queue) fn read(expr: &Expression<'_>) -> JobOptionEvidence {
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => read_object(object),
        Expression::Identifier(id) => JobOptionEvidence {
            spreads: vec![(&**id).into()],
            ..JobOptionEvidence::default()
        },
        _ => JobOptionEvidence::open(),
    }
}

pub(super) fn read_object(object: &ObjectExpression<'_>) -> JobOptionEvidence {
    let mut out = JobOptionEvidence::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key) {
                    Some(name) => {
                        out.keys.insert(name.into_owned());
                    }
                    None => out.open = true,
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                match spread.argument.get_inner_expression() {
                    Expression::ObjectExpression(inner) => out.absorb(&read_object(inner)),
                    Expression::Identifier(id) => out.spreads.push((&**id).into()),
                    _ => out.open = true,
                }
            }
        }
    }
    out
}

pub(super) fn read_member(object: &ObjectExpression<'_>, key: &str) -> Option<JobOptionEvidence> {
    read_member_with(object, key, |spread| match spread.get_inner_expression() {
        Expression::ObjectExpression(inner) => read_member(inner, key),
        _ => Some(JobOptionEvidence::open()),
    })
}

pub(super) fn read_member_with(
    object: &ObjectExpression<'_>,
    key: &str,
    mut spread_value: impl FnMut(&Expression<'_>) -> Option<JobOptionEvidence>,
) -> Option<JobOptionEvidence> {
    let mut evidence = None;
    for property in &object.properties {
        let next = match property {
            ObjectPropertyKind::SpreadProperty(spread) => spread_value(&spread.argument),
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key) {
                    Some(name) if name == key => Some(read(&property.value)),
                    Some(_) => None,
                    None => Some(JobOptionEvidence::open()),
                }
            }
        };
        if next.is_some() {
            evidence = next;
        }
    }
    evidence
}

pub(in crate::queue) fn finish(facts: &mut FileFacts, bindings: &ObjectBindings<'_>) {
    let mut resolver = SpreadResolver {
        bindings,
        objects: FxHashMap::default(),
        members: FxHashMap::default(),
        traversal: DeclarationTraversal::default(),
    };
    let wrappers = facts
        .enqueue_wrappers
        .iter_mut()
        .map(|wrapper| &mut wrapper.options);
    for site in facts.job_option_sites.iter_mut().chain(wrappers) {
        if site.evidence.member.take().is_some() {
            resolver.fold_members(&mut site.evidence);
        } else {
            resolver.fold_keys(&mut site.evidence);
        }
    }
}

struct SpreadResolver<'binding, 'ast> {
    bindings: &'binding ObjectBindings<'ast>,
    objects: FxHashMap<u32, JobOptionEvidence>,
    members: FxHashMap<u32, Option<JobOptionEvidence>>,
    traversal: DeclarationTraversal,
}

impl SpreadResolver<'_, '_> {
    fn fold_keys(&mut self, evidence: &mut JobOptionEvidence) {
        for spread in std::mem::take(&mut evidence.spreads) {
            evidence.absorb(&self.option_keys(&spread));
        }
    }

    fn fold_members(&mut self, evidence: &mut JobOptionEvidence) {
        for spread in std::mem::take(&mut evidence.spreads) {
            if let Some(member) = self.opts_member(&spread) {
                *evidence = member;
            }
        }
    }

    fn option_keys(&mut self, spread: &JobOptionSpread) -> JobOptionEvidence {
        let Some((declaration, value)) = self.value(spread) else {
            return JobOptionEvidence::open();
        };
        if let Some(found) = self.objects.get(&declaration) {
            return found.clone();
        }
        let Some(cycle_count) = self.traversal.enter(declaration) else {
            return JobOptionEvidence::open();
        };
        let mut evidence = read(value);
        self.fold_keys(&mut evidence);
        if self.traversal.complete(declaration, cycle_count) {
            self.objects.insert(declaration, evidence.clone());
        }
        evidence
    }

    fn opts_member(&mut self, spread: &JobOptionSpread) -> Option<JobOptionEvidence> {
        let Some((declaration, value)) = self.value(spread) else {
            return Some(JobOptionEvidence::open());
        };
        if let Some(found) = self.members.get(&declaration) {
            return found.clone();
        }
        let Some(cycle_count) = self.traversal.enter(declaration) else {
            return Some(JobOptionEvidence::open());
        };
        let member = match value {
            Expression::ObjectExpression(object) => read_member(object, "opts").map(|mut opts| {
                self.fold_keys(&mut opts);
                opts
            }),
            Expression::Identifier(id) => self.opts_member(&(&**id).into()),
            _ => Some(JobOptionEvidence::open()),
        };
        if self.traversal.complete(declaration, cycle_count) {
            self.members.insert(declaration, member.clone());
        }
        member
    }
}

impl<'ast> SpreadResolver<'_, 'ast> {
    fn value(&self, spread: &JobOptionSpread) -> Option<(u32, &'ast Expression<'ast>)> {
        let binding = self.bindings.lookup(&spread.name, spread.at)?;
        Some((binding.resolve_at, binding.value()?))
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "job_options_walk_tests.rs"]
mod tests;
