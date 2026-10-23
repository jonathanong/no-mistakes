use super::{combine, expression, Recovered, ScopeVisitor};
use crate::codebase::postgres::embedded::first_call_argument;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

/// Fluent links do not spend recursion depth; nested suffix expressions do.
pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let mut links = vec![call];
    let mut base = receiver(call)?;
    loop {
        let Expression::CallExpression(inner) = unwrap_ts_wrappers(base) else {
            break;
        };
        let Some(inner_base) = receiver(inner) else {
            break;
        };
        links.push(inner);
        base = inner_base;
    }
    let mut versions = expression::recover(visitor, base, depth, constraints)?;
    if !versions.iter().all(|value| value.fragment) {
        return None;
    }
    for link in links.into_iter().rev() {
        let mut suffix =
            expression::recover(visitor, first_call_argument(link)?, depth, constraints)?;
        visitor.mark_append(&mut suffix, link.span.start);
        versions = combine(versions, suffix)?;
    }
    Some(versions)
}

fn receiver<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    if call.optional {
        return None;
    }
    match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "append" =>
        {
            Some(&member.object)
        }
        Expression::ComputedMemberExpression(member)
            if !member.optional
                && matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(name) if name.value == "append") =>
        {
            Some(&member.object)
        }
        _ => None,
    }
}
