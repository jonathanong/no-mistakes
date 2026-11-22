use super::super::extract_model::JobOptionEvidence;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use std::collections::{HashMap, HashSet};

pub(super) fn read(expr: &Expression<'_>) -> JobOptionEvidence {
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => read_object(object),
        Expression::Identifier(id) => JobOptionEvidence {
            spreads: vec![id.name.to_string()],
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
                    Expression::Identifier(id) => out.spreads.push(id.name.to_string()),
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

pub(super) fn resolve_objects(
    raw: &HashMap<String, JobOptionEvidence>,
) -> HashMap<String, JobOptionEvidence> {
    let mut memo = HashMap::new();
    let mut names: Vec<&String> = raw.keys().collect();
    names.sort();
    let mut stack = HashSet::new();
    for name in names {
        resolve_one(name, raw, &mut memo, &mut stack);
    }
    memo
}

fn resolve_one(
    name: &str,
    raw: &HashMap<String, JobOptionEvidence>,
    memo: &mut HashMap<String, JobOptionEvidence>,
    stack: &mut HashSet<String>,
) -> JobOptionEvidence {
    if let Some(done) = memo.get(name) {
        return done.clone();
    }
    let Some(item) = raw.get(name) else {
        return JobOptionEvidence::open();
    };
    if !stack.insert(name.to_string()) {
        return JobOptionEvidence::open();
    }
    let mut out = JobOptionEvidence {
        keys: item.keys.clone(),
        open: item.open,
        ..JobOptionEvidence::default()
    };
    for spread in &item.spreads {
        out.absorb(&resolve_one(spread, raw, memo, stack));
    }
    stack.remove(name);
    memo.insert(name.to_string(), out.clone());
    out
}

pub(super) fn apply_evidence(
    evidence: &mut JobOptionEvidence,
    objects: &HashMap<String, JobOptionEvidence>,
) {
    for spread in std::mem::take(&mut evidence.spreads) {
        match objects.get(&spread) {
            Some(other) => {
                evidence.keys.extend(other.keys.iter().cloned());
                evidence.open |= other.open;
            }
            None => evidence.open = true,
        }
    }
    evidence.member = None;
}
