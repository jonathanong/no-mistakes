//! Direct mutable literal syntax, recorded during the existing head walk.
use super::{syntax::Head, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::*;
mod proofs;
pub(super) use proofs::Proofs;

#[derive(Clone, Copy, Default)]
pub(super) struct Context {
    pub function: Option<u32>,
    pub repeated: Option<Span>,
}
#[derive(Clone, Copy)]
pub(super) enum Value {
    Literal(Head),
    Undefined(Span),
    NoSql,
    Unknown,
}
#[derive(Clone, Copy)]
pub(super) struct Write {
    pub span: Span,
    pub context: Context,
    pub value: Value,
}
#[derive(Default)]
pub(super) struct Assignments {
    pub context: Context,
    pub iteration_target: bool,
    pub declarations: FxHashMap<Span, Write>,
    pub writes: FxHashMap<Span, Write>,
}
impl Assignments {
    pub fn declaration(&mut self, value: &VariableDeclaration<'_>) {
        if !matches!(
            value.kind,
            VariableDeclarationKind::Let | VariableDeclarationKind::Var
        ) {
            return;
        }
        for declaration in &value.declarations {
            if let BindingPattern::BindingIdentifier(id) = &declaration.id {
                self.declarations.insert(
                    (id.span.start, id.span.end),
                    Write {
                        span: (declaration.span.start, declaration.span.end),
                        context: self.context,
                        value: if self.iteration_target {
                            Value::Unknown
                        } else {
                            declaration.init.as_ref().map_or(Value::NoSql, literal)
                        },
                    },
                );
            }
        }
    }
    pub fn assignment(&mut self, value: &AssignmentExpression<'_>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &value.left {
            self.writes.insert(
                (id.span.start, id.span.end),
                Write {
                    span: (value.span.start, value.span.end),
                    context: self.context,
                    value: if value.operator == AssignmentOperator::Assign {
                        literal(&value.right)
                    } else {
                        Value::Unknown
                    },
                },
            );
        }
    }
}
fn literal(expression: &Expression<'_>) -> Value {
    let expression = unwrap_ts_wrappers(expression);
    match expression {
        Expression::StringLiteral(_) => Value::Literal(super::syntax::head(expression)),
        Expression::TemplateLiteral(value) if value.expressions.is_empty() => {
            Value::Literal(super::syntax::head(expression))
        }
        Expression::TaggedTemplateExpression(value) if value.quasi.expressions.is_empty() => {
            Value::Literal(super::syntax::head(expression))
        }
        Expression::Identifier(id) if id.name == "undefined" => {
            Value::Undefined((id.span.start, id.span.end))
        }
        Expression::UnaryExpression(value)
            if value.operator == UnaryOperator::Void
                && matches!(
                    unwrap_ts_wrappers(&value.argument),
                    Expression::NumericLiteral(_)
                        | Expression::StringLiteral(_)
                        | Expression::BooleanLiteral(_)
                        | Expression::NullLiteral(_)
                        | Expression::BigIntLiteral(_)
                ) =>
        {
            Value::NoSql
        }
        _ => Value::Unknown,
    }
}
