//! Recover a tracked SQL builder or its `.text` paired with its own `.values`,
//! optionally wrapped in a template with one interpolation.

use super::{combine, piece, Recovered, ScopeVisitor, ValueKind};
use crate::codebase::postgres::embedded::walk::variants::expression;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, TemplateLiteral};

/// The optional template and builder identifier for either executor overload.
/// Paired text and values must use the same builder.
pub(in crate::codebase::postgres::embedded::walk) fn shape<'a>(
    call: &'a CallExpression<'a>,
) -> Option<(Option<&'a TemplateLiteral<'a>>, &'a Expression<'a>)> {
    if call.optional {
        return None;
    }
    if let [argument] = call.arguments.as_slice() {
        let builder = unwrap_ts_wrappers(argument.as_expression()?);
        return matches!(builder, Expression::Identifier(_)).then_some((None, builder));
    }
    if call.arguments.len() != 2 {
        return None;
    }
    let argument = unwrap_ts_wrappers(call.arguments[0].as_expression()?);
    let (template, interpolated) = match argument {
        Expression::TemplateLiteral(template) => {
            let [interpolated] = template.expressions.as_slice() else {
                return None;
            };
            (Some(&**template), interpolated)
        }
        _ => (None, argument),
    };
    let (text_owner, text_name) = builder_property(interpolated, "text")?;
    let (_, values_owner) = builder_property(call.arguments[1].as_expression()?, "values")?;
    (text_name == values_owner).then_some((template, text_owner))
}

/// `name.property` on a plain identifier: the identifier expression and its name.
fn builder_property<'a>(
    expr: &'a Expression<'a>,
    property: &str,
) -> Option<(&'a Expression<'a>, &'a str)> {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    if member.optional || member.property.name != property {
        return None;
    }
    let Expression::Identifier(owner) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    Some((&member.object, owner.name.as_str()))
}

/// Every alternative of the template, with the builder's alternatives spliced
/// where `.text` stands. The builder must be fully recovered SQL.
pub(in crate::codebase::postgres::embedded::walk::variants) fn recover(
    visitor: &ScopeVisitor<'_>,
    template: Option<&TemplateLiteral<'_>>,
    builder: &Expression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let builders = expression::recover(visitor, builder, depth, constraints)?;
    if !builders
        .iter()
        .all(|value| value.value == ValueKind::Sql && value.fragment)
    {
        return None;
    }
    let Some(template) = template else {
        return Some(builders);
    };
    let [head, tail] = template.quasis.as_slice() else {
        return None;
    };
    let mut versions = combine(vec![piece::quasi(visitor, head)], builders)?;
    versions = combine(versions, vec![piece::quasi(visitor, tail)])?;
    Some(versions)
}
