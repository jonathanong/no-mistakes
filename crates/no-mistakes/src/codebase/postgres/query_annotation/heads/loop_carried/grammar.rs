use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;

pub(super) enum Flow {
    Next,
    Backedge,
    Exit,
    Unsupported,
}

pub(super) fn sequence<'b, 'a>(
    body: &'b Statement<'a>,
    expressions: &mut Vec<&'b Expression<'a>>,
) -> Flow {
    match body {
        Statement::BlockStatement(body) => {
            for value in &body.body {
                match sequence(value, expressions) {
                    Flow::Next => {}
                    result => return result,
                }
            }
            Flow::Next
        }
        Statement::ExpressionStatement(value) => {
            expressions.push(&value.expression);
            Flow::Next
        }
        Statement::VariableDeclaration(value) => {
            expressions.extend(
                value
                    .declarations
                    .iter()
                    .filter_map(|value| value.init.as_ref()),
            );
            Flow::Next
        }
        Statement::IfStatement(_) => {
            // Calls and literals in branches are not unconditional candidates.
            // The path proof handles branch resets and exits independently.
            Flow::Next
        }
        Statement::ContinueStatement(value) if value.label.is_none() => Flow::Backedge,
        Statement::BreakStatement(_)
        | Statement::ReturnStatement(_)
        | Statement::ThrowStatement(_) => Flow::Exit,
        Statement::EmptyStatement(_) | Statement::DebuggerStatement(_) => Flow::Next,
        _ => Flow::Unsupported,
    }
}

pub(in crate::codebase::postgres::query_annotation::heads) fn iteration(
    left: &ForStatementLeft<'_>,
    right: &Expression<'_>,
) -> bool {
    matches!(right, Expression::Identifier(_))
        && matches!(left, ForStatementLeft::VariableDeclaration(value)
            if matches!(value.declarations[0].id, BindingPattern::BindingIdentifier(_)))
}

/// Loop headers must not reset a head or invoke code before its next read.
/// Literal false tests cannot provide a second iteration witness.
pub(in crate::codebase::postgres::query_annotation::heads) fn test(value: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(value) {
        Expression::Identifier(_) => true,
        Expression::BooleanLiteral(value) => value.value,
        Expression::BinaryExpression(value) => {
            pure(&value.left)
                && pure(&value.right)
                && (matches!(unwrap_ts_wrappers(&value.left), Expression::Identifier(_))
                    || matches!(unwrap_ts_wrappers(&value.right), Expression::Identifier(_)))
        }
        Expression::UnaryExpression(value) => {
            value.operator == oxc_syntax::operator::UnaryOperator::LogicalNot
                && matches!(
                    unwrap_ts_wrappers(&value.argument),
                    Expression::Identifier(_)
                )
        }
        Expression::LogicalExpression(value) => test(&value.left) && test(&value.right),
        _ => false,
    }
}
fn pure(value: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(value) {
        Expression::Identifier(_)
        | Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::NullLiteral(_) => true,
        Expression::BinaryExpression(value) => pure(&value.left) && pure(&value.right),
        _ => false,
    }
}

pub(in crate::codebase::postgres::query_annotation::heads) fn update(
    value: &Expression<'_>,
) -> bool {
    match unwrap_ts_wrappers(value) {
        Expression::UpdateExpression(value) => matches!(
            value.argument,
            SimpleAssignmentTarget::AssignmentTargetIdentifier(_)
        ),
        value => pure(value),
    }
}
