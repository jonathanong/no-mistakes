//! Executor choices at low effort: `options.query ?? read` and `left || right`.
mod functions;
mod prepare;
use super::{span, Span, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, LogicalOperator};

pub(in crate::codebase::postgres::sql_identifier_syntax) use prepare::prepare;

pub(super) fn take_call(
    syntax: &mut Syntax<'_>,
    callee: &Expression<'_>,
) -> Option<(Span, Vec<String>)> {
    if !is_choice(callee, syntax) {
        return None;
    }
    let anchor = anchor(callee, syntax)?;
    mark_uses(callee, syntax);
    Some(anchor)
}

pub(super) fn is_choice(expression: &Expression<'_>, syntax: &Syntax<'_>) -> bool {
    let Expression::LogicalExpression(value) = unwrap_ts_wrappers(expression) else {
        return false;
    };
    matches!(
        value.operator,
        LogicalOperator::Or | LogicalOperator::Coalesce
    ) && proven(&value.left, syntax)
        && proven(&value.right, syntax)
}

fn proven(expression: &Expression<'_>, syntax: &Syntax<'_>) -> bool {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(id) => {
            let at = span(id.span);
            syntax.targets.contains(&at) || syntax.typed_targets.contains(&at)
        }
        Expression::StaticMemberExpression(member) => member_query(member, syntax),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                member_query(member, syntax)
            }
            _ => false,
        },
        Expression::LogicalExpression(value)
            if matches!(
                value.operator,
                LogicalOperator::Or | LogicalOperator::Coalesce
            ) =>
        {
            proven(&value.left, syntax) && proven(&value.right, syntax)
        }
        _ => false,
    }
}

fn anchor(expression: &Expression<'_>, syntax: &Syntax<'_>) -> Option<(Span, Vec<String>)> {
    let mut best: Option<(u32, Span, Vec<String>)> = None;
    walk_anchor(expression, syntax, &mut best);
    best.map(|(_, at, parts)| (at, parts))
}

fn walk_anchor(
    expression: &Expression<'_>,
    syntax: &Syntax<'_>,
    best: &mut Option<(u32, Span, Vec<String>)>,
) {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(id) => {
            let at = span(id.span);
            if syntax.targets.contains(&at) || syntax.typed_targets.contains(&at) {
                consider(best, at.1, at, vec![id.name.to_string()]);
            }
        }
        Expression::StaticMemberExpression(member) => {
            note_member(member, syntax, best);
            walk_anchor(&member.object, syntax, best);
        }
        Expression::ChainExpression(chain) => {
            if let oxc_ast::ast::ChainElement::StaticMemberExpression(member) = &chain.expression {
                note_member(member, syntax, best);
                walk_anchor(&member.object, syntax, best);
            }
        }
        Expression::LogicalExpression(value)
            if matches!(
                value.operator,
                LogicalOperator::Or | LogicalOperator::Coalesce
            ) =>
        {
            walk_anchor(&value.left, syntax, best);
            walk_anchor(&value.right, syntax, best);
        }
        _ => {}
    }
}

fn member_query(member: &oxc_ast::ast::StaticMemberExpression<'_>, syntax: &Syntax<'_>) -> bool {
    member.property.name == "query"
        && super::chain(&member.object).is_some_and(|(root, parts)| {
            parts.len() == 1
                && (syntax.typed_targets.contains(&root) || syntax.targets.contains(&root))
        })
}

fn note_member(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    syntax: &Syntax<'_>,
    best: &mut Option<(u32, Span, Vec<String>)>,
) {
    if member.property.name != "query" {
        return;
    }
    let Some((root, parts)) = super::chain(&member.object) else {
        return;
    };
    if parts.len() == 1 && (syntax.typed_targets.contains(&root) || syntax.targets.contains(&root))
    {
        consider(
            best,
            member.span.end,
            root,
            vec![parts[0].clone(), "query".to_string()],
        );
    }
}

fn consider(best: &mut Option<(u32, Span, Vec<String>)>, end: u32, at: Span, parts: Vec<String>) {
    if best.as_ref().is_none_or(|(at, _, _)| end >= *at) {
        *best = Some((end, at, parts));
    }
}

pub(super) fn mark_uses(expression: &Expression<'_>, syntax: &mut Syntax<'_>) {
    let mut leaves = Vec::new();
    executor_spans(expression, syntax, &mut leaves);
    for at in leaves {
        syntax.choice_uses.insert(at);
    }
}

pub(super) fn executor_spans(
    expression: &Expression<'_>,
    syntax: &Syntax<'_>,
    out: &mut Vec<Span>,
) {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(id) => {
            let at = span(id.span);
            if syntax.targets.contains(&at) || syntax.typed_targets.contains(&at) {
                out.push(at);
            }
        }
        Expression::StaticMemberExpression(member) => {
            executor_spans(&member.object, syntax, out);
        }
        Expression::ChainExpression(chain) => {
            if let oxc_ast::ast::ChainElement::StaticMemberExpression(member) = &chain.expression {
                executor_spans(&member.object, syntax, out);
            }
        }
        Expression::LogicalExpression(value)
            if matches!(
                value.operator,
                LogicalOperator::Or | LogicalOperator::Coalesce
            ) =>
        {
            executor_spans(&value.left, syntax, out);
            executor_spans(&value.right, syntax, out);
        }
        _ => {}
    }
}
