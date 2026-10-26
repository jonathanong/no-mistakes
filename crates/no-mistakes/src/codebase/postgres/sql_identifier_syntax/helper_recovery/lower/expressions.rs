use super::{forms, templates, Expr, Scope};
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, UnaryOperator};

pub(super) fn lower(scope: &mut Scope<'_>, node: &Expression<'_>) -> Expr {
    let node = unwrap_ts_wrappers(node);
    match node {
        Expression::StringLiteral(value) => {
            let at = value.span;
            let raw = scope
                .ctx
                .source
                .get(at.start as usize + 1..at.end as usize - 1);
            if raw.is_none_or(|raw| {
                !super::super::super::literal_projection::literal_matches(
                    raw,
                    crate::ast::js_literal(&value).as_ref(),
                )
            }) {
                return Expr::Opaque;
            }
            Expr::Lit(RecoveredSql::literal(
                node,
                scope.ctx.source,
                crate::ast::js_literal(&value).into_owned(),
            ))
        }
        Expression::BooleanLiteral(value) => Expr::Bool(value.value),
        Expression::NullLiteral(_) => Expr::Null,
        Expression::NumericLiteral(_) => Expr::Num,
        Expression::Identifier(value) => {
            if let Some(id) = scope.ctx.bindings.id(value.span) {
                scope.references.push((id, super::span(value.span)));
                Expr::Ident(format!("binding:{id}"))
            } else if value.name == "undefined" {
                Expr::Undefined
            } else {
                Expr::Opaque
            }
        }
        Expression::TemplateLiteral(value) => templates::lower(scope, value, false),
        Expression::TaggedTemplateExpression(value) => {
            let Some(id) = scope.ctx.tags.tag_owner(&value.tag) else {
                return Expr::Opaque;
            };
            scope.tags.insert(id);
            templates::lower(scope, &value.quasi, true)
        }
        Expression::StaticMemberExpression(value) if !value.optional => Expr::Member(
            Box::new(scope.expression(&value.object)),
            value.property.name.to_string(),
        ),
        Expression::ComputedMemberExpression(value) if !value.optional => {
            let Expression::StringLiteral(key) = unwrap_ts_wrappers(&value.expression) else {
                return Expr::Opaque;
            };
            Expr::Member(
                Box::new(scope.expression(&value.object)),
                crate::ast::js_literal(&key).into_owned(),
            )
        }
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::BinaryExpression(_) => forms::lower(scope, node),
        Expression::UnaryExpression(value) if value.operator == UnaryOperator::LogicalNot => {
            Expr::Not(Box::new(scope.expression(&value.argument)))
        }
        Expression::AwaitExpression(value) => {
            // This operation cannot collide with JS calls: callee names above
            // and below are always exact semantic binding slots.
            Expr::Call(
                "operation:await".into(),
                vec![scope.expression(&value.argument)],
            )
        }
        Expression::CallExpression(call) if !call.optional => super::calls::lower(scope, call),

        _ => Expr::Opaque,
    }
}
