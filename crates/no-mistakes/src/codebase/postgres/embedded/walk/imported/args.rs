//! What a helper call's arguments are, as far as the projection may know.

use super::super::resolve::compose::static_fragment;
use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::helpers::SiteArg;
use crate::codebase::postgres::embedded::{unpublished_sql_text, EmbeddedSqlKind};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, ObjectPropertyKind, PropertyKey, PropertyKind};

/// Every argument must be a value the projection can reason about; at most one
/// may be a SQL builder.
pub(super) fn classify<'a>(
    visitor: &ScopeVisitor<'_>,
    call: &'a CallExpression<'a>,
) -> Option<(Vec<SiteArg>, Option<&'a Expression<'a>>)> {
    let mut args = Vec::with_capacity(call.arguments.len());
    let mut builder = None;
    for argument in &call.arguments {
        let expression = argument.as_expression()?;
        let arg = match unwrap_ts_wrappers(expression) {
            Expression::Identifier(ident) if is_builder(visitor, ident.name.as_str()) => {
                SiteArg::Builder
            }
            Expression::TaggedTemplateExpression(_)
                if static_fragment(expression, visitor).is_some() =>
            {
                SiteArg::Builder
            }
            _ => value(visitor, expression, true)?,
        };
        if arg == SiteArg::Builder && builder.replace(expression).is_some() {
            return None;
        }
        args.push(arg);
    }
    Some((args, builder))
}

fn is_builder(visitor: &ScopeVisitor<'_>, name: &str) -> bool {
    visitor.lookup(name).is_some_and(|binding| {
        // A builder with finite alternatives still has a known value.
        binding.sql_builder
            && (binding.sql.is_some() && binding.kind != EmbeddedSqlKind::Dynamic
                || binding.variants.is_some())
    })
}

/// A non-builder value. Objects nest only through `nested`, and never hold
/// builders: a builder inside an object cannot be tracked.
fn value(visitor: &ScopeVisitor<'_>, expression: &Expression<'_>, nested: bool) -> Option<SiteArg> {
    Some(match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(literal) => SiteArg::Literal(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            SiteArg::Literal(unpublished_sql_text(expression)?)
        }
        Expression::NumericLiteral(_) => SiteArg::Scalar,
        Expression::BooleanLiteral(literal) => SiteArg::Bool(literal.value),
        Expression::NullLiteral(_) => SiteArg::Null,
        Expression::Identifier(ident) if ident.name == "undefined" => SiteArg::Undefined,
        Expression::Identifier(ident) => {
            if is_builder(visitor, ident.name.as_str()) {
                return None;
            }
            SiteArg::Scalar
        }
        Expression::StaticMemberExpression(member) => {
            value(visitor, &member.object, false)?;
            SiteArg::Scalar
        }
        Expression::ObjectExpression(object) if nested => object_literal(visitor, object)?,
        Expression::ArrayExpression(array) if nested => {
            let mut items = Vec::with_capacity(array.elements.len());
            for element in &array.elements {
                items.push(value(visitor, element.as_expression()?, false)?);
            }
            SiteArg::Array(items)
        }
        _ => return None,
    })
}

fn object_literal(
    visitor: &ScopeVisitor<'_>,
    object: &oxc_ast::ast::ObjectExpression<'_>,
) -> Option<SiteArg> {
    let mut props = Vec::with_capacity(object.properties.len());
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.kind != PropertyKind::Init || property.method || property.computed {
            return None;
        }
        let key = match &property.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
            PropertyKey::StringLiteral(literal) => literal.value.to_string(),
            _ => return None,
        };
        props.push((key, value(visitor, &property.value, false)?));
    }
    Some(SiteArg::Object(props))
}
