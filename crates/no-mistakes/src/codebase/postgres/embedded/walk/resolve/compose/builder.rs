use super::super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::ScopeVisitor;
use super::{recovered_builder_binding_sql, untrusted_tag, DYNAMIC_SQL_FRAGMENT};
use crate::codebase::postgres::embedded::unpublished_sql_text;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, CallExpression, Expression};

#[inline(never)]
pub(in crate::codebase::postgres::embedded::walk) fn builder_fragment(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    recover(expr, visitor, false)
}
#[inline(never)]
pub(in crate::codebase::postgres::embedded::walk) fn appended_builder_fragment(
    call: &CallExpression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    is_builder_append(call, visitor)
        .then(|| {
            call.arguments
                .first()?
                .as_expression()
                .and_then(|arg| builder_fragment(arg, visitor))
        })
        .flatten()
}
#[inline(never)]
pub(in crate::codebase::postgres::embedded::walk) fn is_builder_append(
    call: &CallExpression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return false;
    };
    member.property.name == "append"
        && match unwrap_ts_wrappers(&member.object) {
            Expression::Identifier(receiver) => visitor.is_sql_builder(receiver.name.as_str()),
            Expression::TaggedTemplateExpression(_) => !untrusted_tag(&member.object, visitor),
            Expression::CallExpression(_) => recover(&member.object, visitor, false).is_some(),
            _ => false,
        }
}
#[inline(never)]
fn recover(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>, dynamic: bool) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            unpublished_sql_text(expr)
        }
        Expression::TaggedTemplateExpression(_) if untrusted_tag(expr, visitor) => None,
        Expression::TaggedTemplateExpression(_) => unpublished_sql_text(expr),
        Expression::Identifier(ident) if dynamic => visitor
            .lookup(ident.name.as_str())
            .and_then(|binding| binding.sql)
            .or_else(|| Some(DYNAMIC_SQL_FRAGMENT.to_string())),
        Expression::Identifier(ident) => {
            recovered_builder_binding_sql(ident.name.as_str(), visitor)
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = recover(&binary.left, visitor, dynamic)?;
            let right = renumber_placeholders(
                &recover(&binary.right, visitor, dynamic)?,
                count_placeholders(&left),
            );
            Some(format!("{left}{right}"))
        }
        Expression::CallExpression(call) => {
            let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee)
            else {
                return None;
            };
            (member.property.name == "append")
                .then(|| {
                    let base = recover(&member.object, visitor, false)?;
                    let arg = recover(call.arguments.first()?.as_expression()?, visitor, true)?;
                    Some(format!(
                        "{base}{}",
                        renumber_placeholders(&arg, count_placeholders(&base))
                    ))
                })
                .flatten()
        }
        _ => None,
    }
}
