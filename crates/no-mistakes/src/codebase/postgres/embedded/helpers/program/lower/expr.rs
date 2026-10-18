use super::super::{Entry, Expr};
use super::{template, Scope};
use crate::codebase::postgres::embedded::unpublished_sql_text;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, BinaryOperator, CallExpression, ChainElement, Expression, LogicalOperator,
    ObjectPropertyKind, PropertyKey, PropertyKind, UnaryOperator,
};
use oxc_span::GetSpan;

pub(super) fn lower(scope: &mut Scope<'_, '_>, expr: &Expression<'_>) -> Expr {
    let node = unwrap_ts_wrappers(expr);
    match node {
        Expression::StringLiteral(literal) => literal_text(scope, node, literal.value.to_string()),
        Expression::NumericLiteral(_) => Expr::Num,
        Expression::BooleanLiteral(literal) => Expr::Bool(literal.value),
        Expression::NullLiteral(_) => Expr::Null,
        Expression::Identifier(ident) => match ident.name.as_str() {
            "undefined" => Expr::Undefined,
            "arguments" => Expr::Opaque,
            name => Expr::Ident(name.to_string()),
        },
        Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => {
            match unpublished_sql_text(node) {
                Some(text) => literal_text(scope, node, text),
                None => Expr::Opaque,
            }
        }
        Expression::TemplateLiteral(literal) => template::untagged(scope, literal),
        Expression::TaggedTemplateExpression(tagged) => template::tagged(scope, tagged),
        Expression::StaticMemberExpression(member) => Expr::Member(
            Box::new(lower(scope, &member.object)),
            member.property.name.to_string(),
        ),
        Expression::ComputedMemberExpression(member) => {
            match unwrap_ts_wrappers(&member.expression) {
                Expression::StringLiteral(key) => Expr::Member(
                    Box::new(lower(scope, &member.object)),
                    key.value.to_string(),
                ),
                _ => Expr::Opaque,
            }
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => Expr::Member(
                Box::new(lower(scope, &member.object)),
                member.property.name.to_string(),
            ),
            _ => Expr::Opaque,
        },
        Expression::ObjectExpression(object) => object_literal(scope, object),
        Expression::ArrayExpression(array) => {
            let mut elements = Vec::with_capacity(array.elements.len());
            for element in &array.elements {
                let Some(element) = element.as_expression() else {
                    return Expr::Opaque;
                };
                elements.push(lower(scope, element));
            }
            Expr::Array(elements)
        }
        Expression::ConditionalExpression(branch) => Expr::Cond(
            Box::new(lower(scope, &branch.test)),
            Box::new(lower(scope, &branch.consequent)),
            Box::new(lower(scope, &branch.alternate)),
        ),
        Expression::LogicalExpression(logical) => {
            let left = Box::new(lower(scope, &logical.left));
            let right = Box::new(lower(scope, &logical.right));
            match logical.operator {
                LogicalOperator::And => Expr::And(left, right),
                LogicalOperator::Or => Expr::Or(left, right),
                LogicalOperator::Coalesce => Expr::Nullish(left, right),
            }
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            Expr::Not(Box::new(lower(scope, &unary.argument)))
        }
        Expression::BinaryExpression(binary) => binary_expression(scope, binary),
        Expression::CallExpression(call) => call_expression(scope, call),
        _ => Expr::Opaque,
    }
}

fn literal_text(scope: &Scope<'_, '_>, node: &Expression<'_>, text: String) -> Expr {
    Expr::Lit(RecoveredSql::literal(node, scope.ctx.source, text))
}

fn binary_expression(
    scope: &mut Scope<'_, '_>,
    binary: &oxc_ast::ast::BinaryExpression<'_>,
) -> Expr {
    let (strict, negate) = match binary.operator {
        BinaryOperator::StrictEquality => (true, false),
        BinaryOperator::StrictInequality => (true, true),
        BinaryOperator::Equality => (false, false),
        BinaryOperator::Inequality => (false, true),
        BinaryOperator::Addition => {
            return Expr::Add(
                Box::new(lower(scope, &binary.left)),
                Box::new(lower(scope, &binary.right)),
            )
        }
        _ => return Expr::Opaque,
    };
    Expr::Eq(
        Box::new(lower(scope, &binary.left)),
        Box::new(lower(scope, &binary.right)),
        strict,
        negate,
    )
}

fn object_literal(scope: &mut Scope<'_, '_>, object: &oxc_ast::ast::ObjectExpression<'_>) -> Expr {
    let mut entries = Vec::with_capacity(object.properties.len());
    for property in &object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(spread) => {
                entries.push(Entry::Spread(lower(scope, &spread.argument)));
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                let key = match &property.key {
                    PropertyKey::StaticIdentifier(ident) if !property.computed => {
                        ident.name.to_string()
                    }
                    PropertyKey::StringLiteral(literal) => literal.value.to_string(),
                    _ => return Expr::Opaque,
                };
                if property.kind != PropertyKind::Init || property.method {
                    return Expr::Opaque;
                }
                entries.push(Entry::Prop(key, lower(scope, &property.value)));
            }
        }
    }
    Expr::Object(entries)
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
