use crate::codebase::postgres::query_annotation::{Expr, Function, Step};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn export_reachable_globals<'a>(
    globals: &'a BTreeMap<String, Expr>,
    exported: impl IntoIterator<Item = &'a str>,
) -> BTreeSet<&'a str> {
    let mut reachable = BTreeSet::new();
    for name in exported {
        visit_global(globals, name, &mut reachable);
    }
    reachable
}

fn visit_global<'a>(
    globals: &'a BTreeMap<String, Expr>,
    name: &'a str,
    reachable: &mut BTreeSet<&'a str>,
) {
    if reachable.insert(name) {
        if let Some(value) = globals.get(name) {
            visits::record();
            // A global binding has its own lexical scope, independent of its caller.
            value_reaches(globals, value, reachable, &[]);
        }
    }
}

fn value_reaches<'a>(
    globals: &'a BTreeMap<String, Expr>,
    value: &'a Expr,
    reachable: &mut BTreeSet<&'a str>,
    scopes: &[&Function],
) {
    match value {
        Expr::Name(name) => {
            if !scopes.iter().any(|function| shadowed(function, name)) {
                visit_global(globals, name, reachable);
            }
        }
        Expr::Spread(value)
        | Expr::Member(value, _)
        | Expr::Index(value, _)
        | Expr::OpaqueCallback(value)
        | Expr::Discard(value) => value_reaches(globals, value, reachable, scopes),
        Expr::ComputedMember(value, key) | Expr::Append(value, key) => {
            value_reaches(globals, value, reachable, scopes);
            value_reaches(globals, key, reachable, scopes);
        }
        Expr::Record(values, _)
        | Expr::Object(values)
        | Expr::Array(values)
        | Expr::Container(values)
        | Expr::Sequence(values)
        | Expr::Alternatives(values) => values
            .iter()
            .for_each(|value| value_reaches(globals, value, reachable, scopes)),
        Expr::Function(function) => {
            let mut nested = scopes.to_vec();
            nested.push(function);
            returns_reach(
                globals,
                &function.body,
                reachable,
                &nested,
                !scopes.is_empty(),
            )
        }
        _ => (),
    }
}

// New branch traversal stays conservative for unsupported local bindings and
// respects lexical shadows. Existing top-level return traversal is unchanged.
fn returns_reach<'a>(
    globals: &'a BTreeMap<String, Expr>,
    steps: &'a [Step],
    reachable: &mut BTreeSet<&'a str>,
    scopes: &[&Function],
    branch_return: bool,
) {
    steps.iter().for_each(|step| match step {
        Step::Return(value) => value_reaches(
            globals,
            value,
            reachable,
            if branch_return { scopes } else { &[] },
        ),
        Step::Branch(test, left, right) if !blocked(steps) && !blocked(left) && !blocked(right) => {
            match test {
                Expr::Boolean(false) | Expr::Number(0) => {
                    returns_reach(globals, right, reachable, scopes, true)
                }
                Expr::Boolean(true) | Expr::Number(_) => {
                    returns_reach(globals, left, reachable, scopes, true)
                }
                _ => {
                    returns_reach(globals, left, reachable, scopes, true);
                    returns_reach(globals, right, reachable, scopes, true);
                }
            }
        }
        _ => (),
    });
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

#[cfg(test)]
mod tests;

#[cfg(not(test))]
#[path = "reachability/runtime.rs"]
mod visits;
#[cfg(test)]
#[path = "reachability/tests/visits.rs"]
mod visits;
