use super::super::Expr;
use super::{compound, template, Scope};
use crate::codebase::postgres::embedded::unpublished_sql_text;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, CallExpression, Expression, UnaryOperator};
use oxc_span::GetSpan;

pub(super) fn lower(scope: &mut Scope<'_, '_>, expr: &Expression<'_>) -> Expr {
    let node = unwrap_ts_wrappers(expr);
    match node {
        Expression::StringLiteral(literal) => {
            literal_text(scope, node, crate::ast::js_literal(&literal).into_owned())
        }
        Expression::NumericLiteral(_) => Expr::Num,
        Expression::BooleanLiteral(literal) => Expr::Bool(literal.value),
        Expression::NullLiteral(_) => Expr::Null,
        Expression::Identifier(ident) => identifier(ident.name.as_str()),
        Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => {
            match unpublished_sql_text(node) {
                Some(text) => literal_text(scope, node, text),
                None => Expr::Opaque,
            }
        }
        Expression::TemplateLiteral(literal) => template::untagged(scope, literal),
        Expression::TaggedTemplateExpression(tagged) => template::tagged(scope, tagged),
        Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::ChainExpression(_) => compound::member_expression(scope, node),
        Expression::ObjectExpression(object) => compound::object_literal(scope, object),
        Expression::ArrayExpression(array) => compound::array_literal(scope, array),
        Expression::ConditionalExpression(branch) => compound::conditional(scope, branch),
        Expression::LogicalExpression(logical) => compound::logical(scope, logical),
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            Expr::Not(Box::new(lower(scope, &unary.argument)))
        }
        Expression::BinaryExpression(binary) => compound::binary_expression(scope, binary),
        Expression::CallExpression(call) => call_expression(scope, call),
        _ => Expr::Opaque,
    }
}

fn identifier(name: &str) -> Expr {
    match name {
        "undefined" => Expr::Undefined,
        "arguments" => Expr::Opaque,
        name => Expr::Ident(name.to_string()),
    }
}

fn literal_text(scope: &Scope<'_, '_>, node: &Expression<'_>, text: String) -> Expr {
    Expr::Lit(RecoveredSql::literal(node, scope.ctx.source, text))
}

fn call_expression(scope: &mut Scope<'_, '_>, call: &CallExpression<'_>) -> Expr {
    if call.optional
        || call
            .arguments
            .iter()
            .any(|a| matches!(a, Argument::SpreadElement(_)))
    {
        return Expr::Opaque;
    }
    match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member)
            if member.property.name == "append" && !member.optional =>
        {
            let [argument] = call.arguments.as_slice() else {
                return Expr::Opaque;
            };
            let Some(argument) = argument.as_expression() else {
                return Expr::Opaque;
            };
            let mark = scope.mark(argument.span());
            Expr::Append(
                Box::new(lower(scope, &member.object)),
                Box::new(lower(scope, argument)),
                mark,
            )
        }
        Expression::Identifier(callee) => {
            let args = call
                .arguments
                .iter()
                .filter_map(Argument::as_expression)
                .map(|argument| lower(scope, argument))
                .collect();
            Expr::Call(callee.name.to_string(), args)
        }
        _ => Expr::Opaque,
    }
}

/// Whether evaluating the expression can produce or hand on SQL text.
pub(super) fn contributes_sql(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(_) | Expr::Template(..) | Expr::Sql(..) | Expr::Append(..) | Expr::Call(..) => {
            true
        }
        Expr::Cond(a, b, c) => [a, b, c].into_iter().any(|e| contributes_sql(e)),
        Expr::And(a, b) | Expr::Or(a, b) | Expr::Nullish(a, b) | Expr::Add(a, b) => {
            contributes_sql(a) || contributes_sql(b)
        }
        _ => false,
    }
}
