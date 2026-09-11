use super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::super::tags::{interpolating_untrusted_tag, kind_for_const};
use super::super::super::{unpublished_sql_text, EmbeddedSqlKind};
use super::super::ScopeVisitor;
use super::{chain, functions};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, Expression};

const DYNAMIC_SQL_FRAGMENT: &str = "sql_dynamic_outer.column";

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
    if untrusted_tag(expr, visitor) {
        return (unpublished_sql_text(expr), EmbeddedSqlKind::Dynamic);
    }
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => kind_for_const(literal.value.to_string(), is_const),
        Expression::TaggedTemplateExpression(_) => {
            kind_for_const(unpublished_sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            kind_for_const(unpublished_sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(_) => (unpublished_sql_text(expr), EmbeddedSqlKind::Dynamic),
        Expression::CallExpression(_) => match resolve_chain(expr, visitor) {
            Some(text) if is_const => (Some(text), EmbeddedSqlKind::Composed),
            Some(text) => (Some(text), EmbeddedSqlKind::Dynamic),
            None => (
                resolve_dynamic_chain_prefix(expr, visitor),
                EmbeddedSqlKind::Dynamic,
            ),
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
    let right = renumber_placeholders(&right, count_placeholders(&left));
    Some((format!("{left}{right}"), EmbeddedSqlKind::Composed))
}

pub(super) fn static_fragment(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            unpublished_sql_text(expr)
        }
        Expression::TaggedTemplateExpression(_) if untrusted_tag(expr, visitor) => None,
        Expression::TaggedTemplateExpression(_) => unpublished_sql_text(expr),
        Expression::BinaryExpression(_) => composed_sql(expr, visitor).map(|(text, _)| text),
        Expression::CallExpression(_) => resolve_chain(expr, visitor),
        Expression::Identifier(ident) => recovered_binding_sql(ident.name.as_str(), visitor),
        _ => None,
    }
}

/// Recovers SQL assembled by a builder without classifying the builder as an
/// executed query. Existing tag-shadow checks still apply. A raw identifier
/// appended into SQL becomes a qualified synthetic outer reference: this
/// intentionally preserves possible correlation for shape policy while
/// remaining valid PostgreSQL in relation and value positions.
pub(in crate::codebase::postgres::embedded::walk) fn builder_fragment(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    recover_builder_fragment(expr, visitor, false)
}

pub(in crate::codebase::postgres::embedded::walk) fn appended_builder_fragment(
    call: &oxc_ast::ast::CallExpression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    if !is_builder_append(call, visitor) {
        return None;
    }
    let Expression::StaticMemberExpression(_) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let argument = call.arguments.first()?.as_expression()?;
    builder_fragment(argument, visitor)
}

pub(in crate::codebase::postgres::embedded::walk) fn is_builder_append(
    call: &oxc_ast::ast::CallExpression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return false;
    };
    if member.property.name != "append" {
        return false;
    }
    let Expression::Identifier(receiver) = unwrap_ts_wrappers(&member.object) else {
        return false;
    };
    if !visitor.is_sql_builder(receiver.name.as_str()) {
        return false;
    }
    true
}

fn recover_builder_fragment(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
    allow_dynamic_identifier: bool,
) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            unpublished_sql_text(expr)
        }
        Expression::TaggedTemplateExpression(_) if untrusted_tag(expr, visitor) => None,
        Expression::TaggedTemplateExpression(_) => unpublished_sql_text(expr),
        Expression::Identifier(ident) if allow_dynamic_identifier => visitor
            .lookup(ident.name.as_str())
            .and_then(|binding| binding.sql)
            .or_else(|| Some(DYNAMIC_SQL_FRAGMENT.to_string())),
        Expression::Identifier(ident) => {
            recovered_builder_binding_sql(ident.name.as_str(), visitor)
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = recover_builder_fragment(&binary.left, visitor, allow_dynamic_identifier)?;
            let right = recover_builder_fragment(&binary.right, visitor, allow_dynamic_identifier)?;
            let right = renumber_placeholders(&right, count_placeholders(&left));
            Some(format!("{left}{right}"))
        }
        Expression::CallExpression(call) => recover_builder_append(call, visitor),
        _ => None,
    }
}

fn recover_builder_append(
    call: &oxc_ast::ast::CallExpression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.property.name != "append" {
        return None;
    }
    let base = recover_builder_fragment(&member.object, visitor, false)?;
    let argument = call.arguments.first()?.as_expression()?;
    let appended = recover_builder_fragment(argument, visitor, true)?;
    let appended = renumber_placeholders(&appended, count_placeholders(&base));
    Some(format!("{base}{appended}"))
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
    let mut is_shadowed = |name: &str| tag_shadowed(name, visitor);
    chain::resolve_expr(
        expr,
        functions::MAX_RESOLVE_DEPTH,
        &mut lookup,
        &mut is_shadowed,
        visitor.functions.imported_sql_tags(),
    )
}

fn resolve_dynamic_chain_prefix(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    let mut lookup = |name: &str, _depth: u8| {
        if visitor.shadowed_locally(name) {
            return None;
        }
        visitor.functions.get(name)
    };
    let mut is_shadowed = |name: &str| tag_shadowed(name, visitor);
    chain::resolve_dynamic_prefix(
        expr,
        functions::MAX_RESOLVE_DEPTH,
        &mut lookup,
        &mut is_shadowed,
        visitor.functions.imported_sql_tags(),
    )
}

fn untrusted_tag(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> bool {
    interpolating_untrusted_tag(
        expr,
        &mut |name| tag_shadowed(name, visitor),
        visitor.functions.imported_sql_tags(),
    )
}

/// Whether `name` is untrustworthy as a tagged template's own trusted-tag
/// reference — either a lexical shadow at a nested scope
/// ([`ScopeVisitor::shadowed_locally`]), or a top-level rebinding away from a
/// same-file helper's trusted meaning ([`functions::LocalFunctions::is_tag_shadowed`]).
/// Resolution routed through an intermediate same-file helper body already
/// combines both checks (`functions::resolve_named`'s own `is_shadowed`);
/// this mirrors that for calls and tags resolved directly here, without an
/// intermediate helper.
fn tag_shadowed(name: &str, visitor: &ScopeVisitor<'_>) -> bool {
    visitor.shadowed_locally(name) || visitor.functions.is_tag_shadowed(name)
}

/// Statement-level `query.append(fragment)` looks up a bound SQLStatement.
/// Dynamic or missing bindings fail closed. `chain::resolve_expr` still
/// has no `Identifier` case, so same-file helpers cannot smuggle a parameter
/// through this path when inlined.
fn recovered_binding_sql(name: &str, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let binding = visitor.lookup(name)?;
    match binding.kind {
        EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed | EmbeddedSqlKind::Inline => {
            binding.sql
        }
        EmbeddedSqlKind::Dynamic => None,
    }
}

fn recovered_builder_binding_sql(name: &str, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let binding = visitor.lookup(name)?;
    binding.sql_builder.then_some(binding.sql).flatten()
}
