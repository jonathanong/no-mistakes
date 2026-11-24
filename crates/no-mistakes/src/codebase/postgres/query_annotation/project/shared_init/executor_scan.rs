use super::File;
use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::FxHashSet;

pub(super) fn has_top_level_executor(file: &File<'_>) -> bool {
    !file.executors.is_empty() && steps_call_executor(&file.facts.roots, &file.executors)
}

fn steps_call_executor(steps: &[Step], executors: &FxHashSet<(u32, u32)>) -> bool {
    steps.iter().any(|step| match step {
        Step::Hoisted(_, expr)
        | Step::Bind(_, expr)
        | Step::Append(_, expr)
        | Step::Effect(expr)
        | Step::Return(expr) => expr_calls_executor(expr, executors),
        Step::Branch(test, left, right) => {
            expr_calls_executor(test, executors)
                || steps_call_executor(left, executors)
                || steps_call_executor(right, executors)
        }
        Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => false,
    })
}

fn expr_calls_executor(expr: &Expr, executors: &FxHashSet<(u32, u32)>) -> bool {
    match expr {
        Expr::Call {
            callee,
            args,
            start,
            end,
            ..
        } => {
            executors.contains(&(*start, *end))
                || expr_calls_executor(callee, executors)
                || args.iter().any(|arg| expr_calls_executor(arg, executors))
        }
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => expr_calls_executor(inner, executors),
        Expr::ComputedMember(a, b) | Expr::Append(a, b) | Expr::Binary(_, a, b) => {
            expr_calls_executor(a, executors) || expr_calls_executor(b, executors)
        }
        Expr::Tagged(_, parts, effects) => {
            parts.iter().any(|p| expr_calls_executor(p, executors))
                || effects.iter().any(|e| expr_calls_executor(e, executors))
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
            .any(|item| expr_calls_executor(item, executors)),
        Expr::OptionalCall { call, children } => {
            expr_calls_executor(call, executors)
                || children
                    .iter()
                    .any(|child| expr_calls_executor(child, executors))
        }
        Expr::SlotWrite {
            receiver, value, ..
        } => expr_calls_executor(receiver, executors) || expr_calls_executor(value, executors),
        Expr::Flow(steps) => steps_call_executor(steps, executors),
        _ => false,
    }
}
