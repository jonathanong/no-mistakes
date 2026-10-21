//! Conservative executed-body capability, computed once during lowering.
use super::super::super::{DeleteKey, Expr, Step};
use crate::fx::FxHashSet;

pub(super) fn factory(params: &[String], body: &[Step]) -> bool {
    // Most ordinary functions cannot qualify: reject before provenance sets.
    if !matches!(body.last(), Some(Step::Return(Expr::Function(_)))) {
        return false;
    }
    if params
        .iter()
        .any(|name| matches!(name.as_str(), "arguments" | "this"))
    {
        return false;
    }
    let mut locals = params.iter().map(String::as_str).collect::<FxHashSet<_>>();
    let mut arguments = FxHashSet::from_iter(["arguments"]);
    locals.insert("arguments");
    let mut returned = false;
    for step in body {
        if returned {
            return false;
        }
        match step {
            Step::Reserve(names)
                if !names
                    .iter()
                    .any(|name| name == "arguments" || name == "this") => {}
            Step::Bind(name, expr) if !locals.contains(name.as_str()) => {
                if !matches!(
                    expr,
                    Expr::Text(_)
                        | Expr::Boolean(_)
                        | Expr::Number(_)
                        | Expr::Primitive
                        | Expr::Function(_)
                ) && !matches!(expr, Expr::Name(source) if locals.contains(source.as_str()))
                {
                    return false;
                }
                if matches!(expr, Expr::Name(source) if arguments.contains(source.as_str())) {
                    arguments.insert(name);
                }
                locals.insert(name);
            }
            Step::Effect(Expr::Delete(parts, DeleteKey::Index(_)))
                if matches!(parts.first(), Some(Expr::Name(name)) if arguments.contains(name.as_str()))
                    && parts
                        .iter()
                        .skip(1)
                        .all(|part| matches!(part, Expr::Number(_) | Expr::Text(_))) => {}
            Step::Effect(Expr::SlotWrite {
                receiver, value, ..
            }) if matches!(receiver.as_ref(), Expr::Name(name) if arguments.contains(name.as_str()))
                && matches!(value.as_ref(), Expr::Name(name) if locals.contains(name.as_str())) => {
            }
            // Creating the closure does not execute its body; it runs after the join.
            Step::Return(Expr::Function(_)) => returned = true,
            _ => return false,
        }
    }
    returned
}
