// Frozen pre-optimization traversal for differential regression tests.
use crate::codebase::postgres::query_annotation::{Expr, Function, Step};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn exported_reaches(
    globals: &BTreeMap<String, Expr>,
    exported: &str,
    target: &str,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if exported == target {
        return true;
    }
    if !visiting.insert(exported.to_string()) {
        return false;
    }
    let reaches = globals
        .get(exported)
        .is_some_and(|value| value_reaches(globals, value, target, visiting, &[]));
    visiting.remove(exported);
    reaches
}

fn value_reaches(
    globals: &BTreeMap<String, Expr>,
    value: &Expr,
    target: &str,
    visiting: &mut BTreeSet<String>,
    scopes: &[&Function],
) -> bool {
    match value {
        Expr::Name(name) => {
            !scopes.iter().any(|function| shadowed(function, name))
                && exported_reaches(globals, name, target, visiting)
        }
        Expr::Spread(value)
        | Expr::Member(value, _)
        | Expr::Index(value, _)
        | Expr::OpaqueCallback(value)
        | Expr::Discard(value) => value_reaches(globals, value, target, visiting, scopes),
        Expr::ComputedMember(value, key) | Expr::Append(value, key) => {
            value_reaches(globals, value, target, visiting, scopes)
                || value_reaches(globals, key, target, visiting, scopes)
        }
        Expr::Record(values, _)
        | Expr::Object(values)
        | Expr::Array(values)
        | Expr::Container(values)
        | Expr::Sequence(values)
        | Expr::Alternatives(values) => values
            .iter()
            .any(|value| value_reaches(globals, value, target, visiting, scopes)),
        Expr::Function(function) => {
            let mut nested = scopes.to_vec();
            nested.push(function);
            returns_reach(
                globals,
                &function.body,
                target,
                visiting,
                &nested,
                !scopes.is_empty(),
            )
        }
        _ => false,
    }
}

// New branch traversal stays conservative for unsupported local bindings and
// respects lexical shadows. Existing top-level return traversal is unchanged.
fn returns_reach(
    globals: &BTreeMap<String, Expr>,
    steps: &[Step],
    target: &str,
    visiting: &mut BTreeSet<String>,
    scopes: &[&Function],
    branch_return: bool,
) -> bool {
    steps.iter().any(|step| match step {
        Step::Return(value) => value_reaches(
            globals,
            value,
            target,
            visiting,
            if branch_return { scopes } else { &[] },
        ),
        Step::Branch(test, left, right) if !blocked(steps) && !blocked(left) && !blocked(right) => {
            match test {
                Expr::Boolean(false) | Expr::Number(0) => {
                    returns_reach(globals, right, target, visiting, scopes, true)
                }
                Expr::Boolean(true) | Expr::Number(_) => {
                    returns_reach(globals, left, target, visiting, scopes, true)
                }
                _ => {
                    returns_reach(globals, left, target, visiting, scopes, true)
                        || returns_reach(globals, right, target, visiting, scopes, true)
                }
            }
        }
        _ => false,
    })
}

fn blocked(steps: &[Step]) -> bool {
    steps.iter().any(|step| {
        matches!(
            step,
            Step::Bind(_, _) | Step::Hoisted(_, _) | Step::Var(_) | Step::Unsupported
        )
    })
}

fn shadowed(function: &Function, name: &str) -> bool {
    // Prepared Reserve names cover direct declarations; Var includes hoisted
    // declarations from nested blocks in the function's scope.
    function.params.iter().any(|param| param == name)
        || function.self_name.as_deref() == Some(name)
        || function.body.iter().any(|step| match step {
            Step::Var(local) => local == name,
            Step::Reserve(names) => names.iter().any(|local| local == name),
            _ => false,
        })
}
