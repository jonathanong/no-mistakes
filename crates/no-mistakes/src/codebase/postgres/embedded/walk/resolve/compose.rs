use super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::super::tags::{interpolating_untrusted_tag, kind_for_const};
use super::super::super::{unpublished_sql_text, EmbeddedSqlKind};
use super::super::ScopeVisitor;
use super::{chain, functions};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, Expression};

mod builder;
pub(in crate::codebase::postgres::embedded::walk) use builder::{
    appended_builder_fragment, builder_fragment, is_builder_append,
};

pub(super) const DYNAMIC_SQL_FRAGMENT: &str = "sql_dynamic_outer.column";

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

pub(super) fn contains_parameter_helper(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> bool {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else {
        return false;
    };
    match unwrap_ts_wrappers(&call.callee) {
        Expression::Identifier(ident) => {
            !visitor.shadowed_locally(ident.name.as_str())
                && visitor.functions.is_parameter_builder(ident.name.as_str())
        }
        Expression::StaticMemberExpression(member) if member.property.name == "append" => {
            contains_parameter_helper(&member.object, visitor)
        }
        _ => false,
    }
}

pub(super) fn parameter_source_positions(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<Vec<crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition>> {
    fn resolve(
        expr: &Expression<'_>,
        visitor: &ScopeVisitor<'_>,
    ) -> Option<(
        String,
        Vec<crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition>,
        u32,
    )> {
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else {
            return None;
        };
        match unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(ident) if !visitor.shadowed_locally(ident.name.as_str()) => {
                visitor
                    .functions
                    .call_positions(call, ident.name.as_str(), visitor)
            }
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                let (base, mut positions, origin) = resolve(&member.object, visitor)?;
                let fragment = call.arguments.first()?.as_expression()?;
                let text = static_fragment(fragment, visitor)?;
                let (line, fragment_positions) =
                    super::append::positions::fragment(call, visitor, count_placeholders(&base));
                super::append::positions::append(
                    &mut positions,
                    &base,
                    origin,
                    line,
                    &fragment_positions,
                );
                let text = renumber_placeholders(&text, count_placeholders(&base));
                Some((format!("{base}{text}"), positions, origin))
            }
            _ => None,
        }
    }
    resolve(expr, visitor).map(|(_, positions, _)| positions)
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

/// Resolves a fluent `.append()` chain or a call into a same-file
/// statically-composed function, per [`chain::resolve_expr`]. A callee name
/// shadowed by an in-scope parameter or nested local at this call site is
/// rejected rather than resolved against the same-named top-level
/// declaration; the top-level declaration's own binding (e.g. a const-bound
/// helper referencing itself) is not a shadow of itself.
fn resolve_chain(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let mut lookup = |call: &oxc_ast::ast::CallExpression<'_>, name: &str, depth: u8| {
        if visitor.shadowed_locally(name) {
            return None;
        }
        visitor.functions.get_call(
            call,
            name,
            depth,
            &mut |tag| tag_shadowed(tag, visitor),
            &|binding| recovered_complete_builder_binding_sql(binding, visitor),
        )
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
    let mut lookup = |call: &oxc_ast::ast::CallExpression<'_>, name: &str, depth: u8| {
        if visitor.shadowed_locally(name) {
            return None;
        }
        visitor.functions.get_call(
            call,
            name,
            depth,
            &mut |tag| tag_shadowed(tag, visitor),
            &|binding| recovered_complete_builder_binding_sql(binding, visitor),
        )
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

pub(super) fn recovered_builder_binding_sql(
    name: &str,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    let binding = visitor.lookup(name)?;
    binding.sql_builder.then_some(binding.sql).flatten()
}

fn recovered_complete_builder_binding_sql(
    name: &str,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    let binding = visitor.lookup(name)?;
    (binding.sql_builder && binding.kind != EmbeddedSqlKind::Dynamic)
        .then_some(binding.sql)
        .flatten()
}
