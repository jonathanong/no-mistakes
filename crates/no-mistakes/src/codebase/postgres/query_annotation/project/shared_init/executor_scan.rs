use super::File;
use crate::codebase::postgres::query_annotation::{Expr, Step};

pub(super) fn has_uncacheable_top_level(file: &File<'_>) -> bool {
    steps_any(
        &file.facts.roots,
        &|step| matches!(step, Step::Unsupported | Step::PotentialCalls(_)),
        &|expr| match expr {
            Expr::Call {
                callee, start, end, ..
            } => file.executors.contains(&(*start, *end)) || calls_stateful_local(callee, file),
            _ => false,
        },
    )
}

fn calls_stateful_local(callee: &Expr, file: &File<'_>) -> bool {
    let Expr::Name(name) = callee else {
        return false;
    };
    let Some(Expr::Function(function)) = file.facts.globals.get(name) else {
        return false;
    };
    steps_any(
        &function.body,
        &|step| matches!(step, Step::Branch(..)),
        &|expr| match expr {
            Expr::Name(name) => name == "arguments",
            Expr::SlotWrite { .. } | Expr::OpaqueWrite { .. } => true,
            Expr::Call { start, end, .. } => file.executors.contains(&(*start, *end)),
            _ => false,
        },
    )
}

fn steps_any(
    steps: &[Step],
    step_pred: &impl Fn(&Step) -> bool,
    expr_pred: &impl Fn(&Expr) -> bool,
) -> bool {
    steps.iter().any(|step| {
        step_pred(step)
            || match step {
                Step::Hoisted(_, expr)
                | Step::Bind(_, expr)
                | Step::Append(_, expr)
                | Step::Effect(expr)
                | Step::Return(expr) => expr_any(expr, step_pred, expr_pred),
                Step::Branch(test, left, right) => {
                    expr_any(test, step_pred, expr_pred)
                        || steps_any(left, step_pred, expr_pred)
                        || steps_any(right, step_pred, expr_pred)
                }
                Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => {
                    false
                }
            }
    })
}

fn expr_any(
    expr: &Expr,
    step_pred: &impl Fn(&Step) -> bool,
    expr_pred: &impl Fn(&Expr) -> bool,
) -> bool {
    if expr_pred(expr) {
        return true;
    }
    match expr {
        Expr::Call { callee, args, .. } => {
            expr_any(callee, step_pred, expr_pred)
                || args.iter().any(|arg| expr_any(arg, step_pred, expr_pred))
        }
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => expr_any(inner, step_pred, expr_pred),
        Expr::ComputedMember(a, b)
        | Expr::Append(a, b)
        | Expr::Binary(_, a, b)
        | Expr::SlotWrite {
            receiver: a,
            value: b,
            ..
        } => expr_any(a, step_pred, expr_pred) || expr_any(b, step_pred, expr_pred),
        Expr::Tagged(_, parts, effects) => {
            parts.iter().any(|p| expr_any(p, step_pred, expr_pred))
                || effects.iter().any(|e| expr_any(e, step_pred, expr_pred))
        }
        Expr::Template(items)
        | Expr::Children(items)
        | Expr::Container(items)
        | Expr::Array(items)
        | Expr::Object(items)
        | Expr::Record(items, _)
        | Expr::Sequence(items)
        | Expr::Alternatives(items)
        | Expr::Opaque(items)
        | Expr::OpaqueWrite {
            children: items, ..
        }
        | Expr::Delete(items, _) => items
            .iter()
            .any(|item| expr_any(item, step_pred, expr_pred)),
        Expr::OptionalCall { call, children } => {
            expr_any(call, step_pred, expr_pred)
                || children
                    .iter()
                    .any(|child| expr_any(child, step_pred, expr_pred))
        }
        Expr::Flow(steps) => steps_any(steps, step_pred, expr_pred),
        _ => false,
    }
}
