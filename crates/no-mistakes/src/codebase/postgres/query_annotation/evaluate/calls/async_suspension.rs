use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::{collections::BTreeMap, path::PathBuf};

use super::super::{CallSpan, ExecutionEvents, Value};

pub(super) type EventLengths = BTreeMap<(PathBuf, CallSpan), usize>;

pub(in super::super) fn event_lengths(events: &ExecutionEvents) -> EventLengths {
    events
        .iter()
        .map(|(key, values)| (key.clone(), values.len()))
        .collect()
}

pub(super) fn invalidate_new_events(events: &mut ExecutionEvents, previous: &EventLengths) {
    for (key, values) in events {
        let count = previous.get(key).copied().unwrap_or(0);
        for (_, value) in values.iter_mut().skip(count) {
            *value = Value::Unknown;
        }
    }
}

pub(super) fn has_suspension(steps: &[Step]) -> bool {
    steps.iter().any(|step| match step {
        Step::Bind(_, expr)
        | Step::Hoisted(_, expr)
        | Step::Append(_, expr)
        | Step::Effect(expr)
        | Step::Return(expr) => contains_await(expr),
        Step::Branch(test, left, right) => {
            contains_await(test) || has_suspension(left) || has_suspension(right)
        }
        Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => false,
    })
}

fn contains_await(expr: &Expr) -> bool {
    match expr {
        Expr::Await(_) => true,
        Expr::Flow(steps) => has_suspension(steps),
        Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => contains_await(inner),
        Expr::Binary(_, base, key)
        | Expr::ComputedMember(base, key)
        | Expr::Append(base, key)
        | Expr::SlotWrite {
            receiver: base,
            value: key,
            ..
        } => any_await([base.as_ref(), key.as_ref()]),
        Expr::Call { callee, args, .. } => {
            any_await(std::iter::once(callee.as_ref()).chain(args.iter()))
        }
        Expr::Tagged(_, parts, effects) => any_await(parts.iter().chain(effects)),
        Expr::Template(parts)
        | Expr::Children(parts)
        | Expr::Array(parts)
        | Expr::Container(parts)
        | Expr::Object(parts)
        | Expr::Record(parts, _)
        | Expr::Sequence(parts)
        | Expr::Alternatives(parts)
        | Expr::Opaque(parts)
        | Expr::OpaqueWrite {
            children: parts, ..
        }
        | Expr::Delete(parts, _) => any_await(parts),
        Expr::ModuleLoad(_)
        | Expr::Boolean(_)
        | Expr::Number(_)
        | Expr::Primitive
        | Expr::Unknown
        | Expr::Unsupported
        | Expr::Text(_)
        | Expr::Name(_)
        | Expr::Function(_) => false,
    }
}

fn any_await<'a>(expressions: impl IntoIterator<Item = &'a Expr>) -> bool {
    expressions.into_iter().any(contains_await)
}

#[cfg(test)]
mod tests;
