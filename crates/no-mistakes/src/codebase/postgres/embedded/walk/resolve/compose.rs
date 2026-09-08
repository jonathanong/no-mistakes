use super::super::super::tags::{interpolating_untrusted_tag, kind_for_const};
use super::super::super::{sql_text, EmbeddedSqlKind};
use super::super::ScopeVisitor;
use super::{chain, functions};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, Expression};

pub(super) fn classify_init(
    expr: &Expression<'_>,
    is_const: bool,
    visitor: &ScopeVisitor<'_>,
) -> (Option<String>, EmbeddedSqlKind) {
    if let Some((text, kind)) = composed_sql(expr, visitor) {
        return if is_const {
            (Some(text), kind)
        } else {
            (Some(text), EmbeddedSqlKind::Dynamic)
        };
    }
    if interpolating_untrusted_tag(expr, &mut |name| visitor.shadowed_locally(name)) {
        return (sql_text(expr), EmbeddedSqlKind::Dynamic);
    }
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => kind_for_const(literal.value.to_string(), is_const),
        Expression::TaggedTemplateExpression(_) => {
            kind_for_const(sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            kind_for_const(sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(_) => (sql_text(expr), EmbeddedSqlKind::Dynamic),
        Expression::CallExpression(_) => match resolve_chain(expr, visitor) {
            Some(text) if is_const => (Some(text), EmbeddedSqlKind::Composed),
            Some(text) => (Some(text), EmbeddedSqlKind::Dynamic),
            None => (None, EmbeddedSqlKind::Dynamic),
        },
        _ => (None, EmbeddedSqlKind::Dynamic),
    }
}

fn composed_sql(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<(String, EmbeddedSqlKind)> {
    let Expression::BinaryExpression(binary) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    if binary.operator != BinaryOperator::Addition {
        return None;
    }
    let left = static_fragment(&binary.left, visitor)?;
    let right = static_fragment(&binary.right, visitor)?;
    let right = chain::renumber_placeholders(&right, chain::count_placeholders(&left));
    Some((format!("{left}{right}"), EmbeddedSqlKind::Composed))
}

pub(super) fn static_fragment(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => sql_text(expr),
        Expression::TaggedTemplateExpression(_)
            if interpolating_untrusted_tag(expr, &mut |name| visitor.shadowed_locally(name)) =>
        {
            None
        }
        Expression::TaggedTemplateExpression(_) => sql_text(expr),
        Expression::BinaryExpression(_) => composed_sql(expr, visitor).map(|(text, _)| text),
        Expression::CallExpression(_) => resolve_chain(expr, visitor),
        _ => None,
    }
}

/// Resolves a fluent `.append()` chain or a call into a same-file
/// statically-composed function, per [`chain::resolve_expr`]. A callee name
/// shadowed by an in-scope parameter or nested local at this call site is
/// rejected rather than resolved against the same-named top-level
/// declaration; the top-level declaration's own binding (e.g. a const-bound
/// helper referencing itself) is not a shadow of itself.
fn resolve_chain(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let mut lookup = |name: &str, _depth: u8| {
        if visitor.shadowed_locally(name) {
            return None;
        }
        visitor.functions.get(name)
    };
    let mut is_shadowed = |name: &str| visitor.shadowed_locally(name);
    chain::resolve_expr(
        expr,
        functions::MAX_RESOLVE_DEPTH,
        &mut lookup,
        &mut is_shadowed,
    )
}
