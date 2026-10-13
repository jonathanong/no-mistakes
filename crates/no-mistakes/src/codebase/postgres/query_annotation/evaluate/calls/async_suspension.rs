use crate::codebase::postgres::query_annotation::{Expr, Step};

pub(super) fn has_suspension(steps: &[Step]) -> bool {
    steps.iter().any(|step| match step {
        Step::Bind(_, expr)
        | Step::Hoisted(_, expr)
        | Step::Append(_, expr)
        | Step::Effect(expr)
        | Step::Return(expr) => contains_await(expr),
        Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => false,
    })
}

fn contains_await(expr: &Expr) -> bool {
    match expr {
        Expr::Await(_) => true,
        Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => contains_await(inner),
        Expr::ComputedMember(base, key)
        | Expr::Append(base, key)
        | Expr::SlotWrite {
            receiver: base,
            value: key,
            ..
        } => contains_await(base) || contains_await(key),
        Expr::Call { callee, args, .. } => {
            contains_await(callee) || args.iter().any(contains_await)
        }
        Expr::Tagged(_, parts, effects) => {
            parts.iter().any(contains_await) || effects.iter().any(contains_await)
        }
        Expr::Template(parts)
        | Expr::Children(parts)
        | Expr::Container(parts)
        | Expr::Object(parts)
        | Expr::Record(parts, _)
        | Expr::Sequence(parts)
        | Expr::Alternatives(parts)
        | Expr::Opaque(parts)
        | Expr::OpaqueWrite {
            children: parts, ..
        }
        | Expr::Delete(parts, _) => parts.iter().any(contains_await),
        Expr::Primitive
        | Expr::Unknown
        | Expr::Unsupported
        | Expr::Text(_)
        | Expr::Name(_)
        | Expr::Function(_) => false,
    }
}
