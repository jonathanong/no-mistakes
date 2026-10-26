use super::super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::ScopeVisitor;
use super::{recovered_builder_binding_sql, untrusted_tag, DYNAMIC_SQL_FRAGMENT};
use crate::codebase::postgres::embedded::unpublished_sql_text;
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, CallExpression, Expression};

pub(in crate::codebase::postgres::embedded::walk) fn builder_fragment(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<String> {
    if visitor.head_calls_only {
        return None;
    }
    recover(expr, visitor, false)
}
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
fn recover(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>, dynamic: bool) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(&literal).into_owned()),
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
        Expression::StaticMemberExpression(_) => super::static_fragment(expr, visitor),
        Expression::TemplateLiteral(_) => super::static_fragment(expr, visitor),
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

/// A tracked binding whose text is fully known, as a value appended into a
/// fluent chain. `append` copies the text at that moment, so later changes to
/// either side do not reach the other.
pub(in crate::codebase::postgres::embedded::walk) fn builder_binding_recovered(
    name: &str,
    visitor: &ScopeVisitor<'_>,
) -> Option<super::super::functions::provenance::RecoveredSql> {
    let binding = visitor.lookup(name)?;
    if binding.kind == EmbeddedSqlKind::Dynamic {
        return None;
    }
    let sql = binding.sql?;
    Some(super::super::functions::provenance::RecoveredSql {
        origins: vec![0; sql.len()],
        sql,
        line: binding.line,
        positions: binding.sql_source_positions,
    })
}

/// Statement-level `query.append(fragment)` looks up a bound SQLStatement.
/// Dynamic or missing bindings fail closed. `chain::resolve_expr` still
/// has no `Identifier` case, so same-file helpers cannot smuggle a parameter
/// through this path when inlined.
pub(super) fn recovered_binding_sql(name: &str, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let binding = visitor.lookup(name)?;
    match binding.kind {
        // A tagged-template const is immutable but not a string. Splicing it
        // would treat an SQLStatement object as SQL text.
        EmbeddedSqlKind::ImmutableLocal
            if !visitor.literal_objects.is_text_const(&visitor.scopes, name) =>
        {
            None
        }
        EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed | EmbeddedSqlKind::Inline => {
            binding.sql
        }
        EmbeddedSqlKind::Dynamic => None,
    }
}
