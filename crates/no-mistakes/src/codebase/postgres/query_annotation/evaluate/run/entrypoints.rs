use crate::codebase::postgres::query_annotation::{Expr, Function};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(super) fn collect(
    globals: &BTreeMap<String, Expr>,
    value: &Expr,
    visiting: &mut BTreeSet<String>,
    seen: &mut BTreeSet<u32>,
    functions: &mut Vec<Arc<Function>>,
) {
    match value {
        Expr::Function(function) => {
            if seen.insert(function.start) {
                functions.push(Arc::clone(function));
            }
        }
        Expr::Name(name) => {
            if visiting.insert(name.clone()) {
                if let Some(value) = globals.get(name) {
                    collect(globals, value, visiting, seen, functions);
                }
                visiting.remove(name);
            }
        }
        Expr::Record(values, _)
        | Expr::Object(values)
        | Expr::Container(values)
        | Expr::Sequence(values)
        | Expr::Alternatives(values) => {
            for value in values {
                collect(globals, value, visiting, seen, functions);
            }
        }
        Expr::Spread(value)
        | Expr::Member(value, _)
        | Expr::Index(value, _)
        | Expr::OpaqueCallback(value)
        | Expr::Discard(value) => collect(globals, value, visiting, seen, functions),
        Expr::ComputedMember(base, key) | Expr::Append(base, key) => {
            collect(globals, base, visiting, seen, functions);
            collect(globals, key, visiting, seen, functions);
        }
        _ => {}
    }
}
