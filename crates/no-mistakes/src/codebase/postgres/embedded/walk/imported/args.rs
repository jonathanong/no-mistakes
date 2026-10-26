//! What a helper call's arguments are, as far as the projection may know.

use super::super::resolve::compose::static_fragment;
use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::helpers::{LiteralArg, SiteArg};
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use crate::codebase::ts_source::byte_offset_to_line;
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
///
/// A quoted string and a no-substitution template are the same literal. A
/// template with substitutions is not, same as a same-file template helper.
fn value(visitor: &ScopeVisitor<'_>, expression: &Expression<'_>, nested: bool) -> Option<SiteArg> {
    Some(match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(literal) => SiteArg::Literal(LiteralArg {
            text: crate::ast::js_literal(&literal).into_owned(),
            line: byte_offset_to_line(visitor.source, literal.span.start as usize),
            origin: literal.span.start + 1,
        }),
        // Cooked text is the value a quoted string would have. An invalid
        // escape has no cooked value, so the argument is not spliced.
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let text = cooked_template(template)?;
            SiteArg::Literal(LiteralArg {
                text,
                line: byte_offset_to_line(visitor.source, template.span.start as usize),
                origin: template.span.start + 1,
            })
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

fn cooked_template(template: &oxc_ast::ast::TemplateLiteral<'_>) -> Option<String> {
    template.quasis.first()?.value.cooked.as_ref()?;
    Some(crate::ast::template_quasi_text(template.quasis.first()?).into_owned())
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
            PropertyKey::StringLiteral(literal) => crate::ast::js_literal(&literal).into_owned(),
            _ => return None,
        };
        props.push((key, value(visitor, &property.value, false)?));
    }
    Some(SiteArg::Object(props))
}
