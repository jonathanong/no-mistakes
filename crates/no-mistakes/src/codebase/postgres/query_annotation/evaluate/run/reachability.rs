use crate::codebase::postgres::query_annotation::{Expr, Step};
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
        .is_some_and(|value| value_reaches(globals, value, target, visiting));
    visiting.remove(exported);
    reaches
}

fn value_reaches(
    globals: &BTreeMap<String, Expr>,
    value: &Expr,
    target: &str,
    visiting: &mut BTreeSet<String>,
) -> bool {
    match value {
        Expr::Name(name) => exported_reaches(globals, name, target, visiting),
        Expr::Spread(value)
        | Expr::Member(value, _)
        | Expr::Index(value, _)
        | Expr::OpaqueCallback(value)
        | Expr::Discard(value) => value_reaches(globals, value, target, visiting),
        Expr::ComputedMember(value, key) | Expr::Append(value, key) => {
            value_reaches(globals, value, target, visiting)
                || value_reaches(globals, key, target, visiting)
        }
        Expr::Record(values, _)
        | Expr::Object(values)
        | Expr::Array(values)
        | Expr::Container(values)
        | Expr::Sequence(values)
        | Expr::Alternatives(values) => values
            .iter()
            .any(|value| value_reaches(globals, value, target, visiting)),
        Expr::Function(function) => returns_reach(globals, &function.body, target, visiting),
        _ => false,
    }
}

// A returned private function remains publicly callable through either branch.
fn returns_reach(
    globals: &BTreeMap<String, Expr>,
    steps: &[Step],
    target: &str,
    visiting: &mut BTreeSet<String>,
) -> bool {
    steps.iter().any(|step| match step {
        Step::Return(value) => value_reaches(globals, value, target, visiting),
        Step::Branch(_, left, right) => {
            returns_reach(globals, left, target, visiting)
                || returns_reach(globals, right, target, visiting)
        }
        _ => false,
    })
}
