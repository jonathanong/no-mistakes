use super::File;
use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::FxHashSet;

pub(super) fn has_uncacheable_top_level(file: &File<'_>) -> bool {
    steps_uncacheable(&file.facts.roots, file)
}

fn steps_uncacheable(steps: &[Step], file: &File<'_>) -> bool {
    steps.iter().any(|step| match step {
        Step::Hoisted(_, expr)
        | Step::Bind(_, expr)
        | Step::Append(_, expr)
        | Step::Effect(expr)
        | Step::Return(expr) => expr_uncacheable(expr, file),
        Step::Branch(test, left, right) => {
            expr_uncacheable(test, file)
                || steps_uncacheable(left, file)
                || steps_uncacheable(right, file)
        }
        Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => false,
    })
}

fn expr_uncacheable(expr: &Expr, file: &File<'_>) -> bool {
    match expr {
        Expr::Call {
            callee,
            args,
            start,
            end,
            ..
        } => {
            file.executors.contains(&(*start, *end))
                || calls_stateful_local(callee, file)
                || expr_uncacheable(callee, file)
                || args.iter().any(|arg| expr_uncacheable(arg, file))
        }
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => expr_uncacheable(inner, file),
        Expr::ComputedMember(a, b) | Expr::Append(a, b) | Expr::Binary(_, a, b) => {
            expr_uncacheable(a, file) || expr_uncacheable(b, file)
        }
        Expr::Tagged(_, parts, effects) => {
            parts.iter().any(|p| expr_uncacheable(p, file))
                || effects.iter().any(|e| expr_uncacheable(e, file))
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
        | Expr::Delete(items, _) => items.iter().any(|item| expr_uncacheable(item, file)),
        Expr::OptionalCall { call, children } => {
            expr_uncacheable(call, file)
                || children.iter().any(|child| expr_uncacheable(child, file))
        }
        Expr::SlotWrite {
            receiver, value, ..
        } => expr_uncacheable(receiver, file) || expr_uncacheable(value, file),
        Expr::Flow(steps) => steps_uncacheable(steps, file),
        _ => false,
    }
}

fn calls_stateful_local(callee: &Expr, file: &File<'_>) -> bool {
    let Expr::Name(name) = callee else {
        return false;
    };
    let Some(Expr::Function(function)) = file.facts.globals.get(name) else {
        return false;
    };
    steps_have_stateful_ops(&function.body, &file.executors)
}

fn steps_have_stateful_ops(steps: &[Step], executors: &FxHashSet<(u32, u32)>) -> bool {
    steps.iter().any(|step| match step {
        Step::Branch(..) => true,
        Step::Hoisted(_, expr)
        | Step::Bind(_, expr)
        | Step::Append(_, expr)
        | Step::Effect(expr)
        | Step::Return(expr) => expr_has_stateful_ops(expr, executors),
        Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => false,
    })
}

fn expr_has_stateful_ops(expr: &Expr, executors: &FxHashSet<(u32, u32)>) -> bool {
    match expr {
        Expr::Name(name) => name == "arguments",
        Expr::SlotWrite { .. } | Expr::OpaqueWrite { .. } => true,
        Expr::Call {
            callee,
            args,
            start,
            end,
            ..
        } => {
            executors.contains(&(*start, *end))
                || expr_has_stateful_ops(callee, executors)
                || args.iter().any(|arg| expr_has_stateful_ops(arg, executors))
        }
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => expr_has_stateful_ops(inner, executors),
        Expr::ComputedMember(a, b) | Expr::Append(a, b) | Expr::Binary(_, a, b) => {
            expr_has_stateful_ops(a, executors) || expr_has_stateful_ops(b, executors)
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
        | Expr::Delete(items, _) => items
            .iter()
            .any(|item| expr_has_stateful_ops(item, executors)),
        Expr::Flow(steps) => steps_have_stateful_ops(steps, executors),
        Expr::Function(f) => steps_have_stateful_ops(&f.body, executors),
        _ => false,
    }
}
