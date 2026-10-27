use super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::super::tags::{interpolating_untrusted_tag, kind_for_const};
use super::super::super::{unpublished_sql_text, EmbeddedSqlKind};
use super::super::ScopeVisitor;
use super::{chain, functions};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, Expression};

mod builder;
mod provenance;
pub(in crate::codebase::postgres::embedded::walk) use provenance::helper_provenance;
mod parameter_positions;
pub(in crate::codebase::postgres::embedded::walk) use builder::{
    appended_builder_fragment, builder_binding_recovered, builder_fragment, is_builder_append,
};
pub(in crate::codebase::postgres::embedded::walk) use parameter_positions::contains_parameter_helper;
pub(super) use parameter_positions::parameter_source_positions;

pub(super) const DYNAMIC_SQL_FRAGMENT: &str = "sql_dynamic_outer.column";

pub(super) fn classify_init(
    expr: &Expression<'_>,
    is_const: bool,
    visitor: &ScopeVisitor<'_>,
) -> (Option<String>, EmbeddedSqlKind) {
    if visitor.head_calls_only {
        return (
            super::super::head_calls::text(expr, visitor),
            EmbeddedSqlKind::Dynamic,
        );
    }
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
        Expression::StringLiteral(literal) => {
            kind_for_const(crate::ast::js_literal(literal).into_owned(), is_const)
        }
        Expression::TaggedTemplateExpression(_) => {
            kind_for_const(unpublished_sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            kind_for_const(unpublished_sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(_) => visitor.classified_const_template(expr, is_const),
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

pub(in crate::codebase::postgres::embedded::walk) fn static_fragment(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    if visitor.head_calls_only {
        return super::super::head_calls::text(expr, visitor);
    }
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            unpublished_sql_text(expr)
        }
        Expression::TaggedTemplateExpression(_) if untrusted_tag(expr, visitor) => None,
        Expression::TaggedTemplateExpression(_) => unpublished_sql_text(expr),
        Expression::BinaryExpression(_) => composed_sql(expr, visitor).map(|(text, _)| text),
        Expression::CallExpression(_) => resolve_chain(expr, visitor),
        Expression::Identifier(ident) => {
            builder::recovered_binding_sql(ident.name.as_str(), visitor)
        }
        Expression::StaticMemberExpression(member) => visitor.spliced_member(member, 1),
        Expression::TemplateLiteral(template) => visitor.spliced_template(template, 8),
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
    resolve_chain_recovered(expr, visitor).map(|value| value.sql)
}

fn resolve_chain_recovered(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<functions::provenance::RecoveredSql> {
    let mut lookup = |call: &oxc_ast::ast::CallExpression<'_>, name: &str, depth: u8| {
        if visitor.shadowed_locally(name) {
            return None;
        }
        depth.checked_sub(1)?;
        visitor.functions.get_call_recovered(call, name, visitor)
    };
    let mut is_shadowed = |name: &str| tag_shadowed(name, visitor);
    let mut idents = |name: &str| builder_binding_recovered(name, visitor);
    chain::resolve_expr(
        expr,
        visitor.source,
        functions::MAX_RESOLVE_DEPTH,
        &mut lookup,
        &mut is_shadowed,
        &mut idents,
        visitor.functions.imported_sql_tags(),
    )
}

fn resolve_dynamic_chain_prefix(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    let mut lookup = |call: &oxc_ast::ast::CallExpression<'_>, name: &str, depth: u8| {
        if visitor.shadowed_locally(name) {
            return None;
        }
        depth.checked_sub(1)?;
        visitor.functions.get_call_recovered(call, name, visitor)
    };
    let mut is_shadowed = |name: &str| tag_shadowed(name, visitor);
    let mut idents = |name: &str| builder_binding_recovered(name, visitor);
    chain::resolve_dynamic_prefix(
        expr,
        visitor.source,
        functions::MAX_RESOLVE_DEPTH,
        &mut lookup,
        &mut is_shadowed,
        &mut idents,
        visitor.functions.imported_sql_tags(),
    )
    .map(|value| value.sql)
}

pub(super) fn untrusted_tag(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> bool {
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
pub(in crate::codebase::postgres::embedded::walk) fn tag_shadowed(
    name: &str,
    visitor: &ScopeVisitor<'_>,
) -> bool {
    visitor.shadowed_locally(name) || visitor.functions.is_tag_shadowed(name)
}

pub(super) fn recovered_builder_binding_sql(
    name: &str,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    let binding = visitor.lookup(name)?;
    binding.sql_builder.then_some(binding.sql).flatten()
}
