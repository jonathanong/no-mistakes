use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{
    DeadlineDeclaration, DeadlineUnknownReason, DeadlineValue, DeclaredDeadlines,
};
use oxc_ast::ast::Expression;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use std::path::Path;

pub(super) fn declaration(value: &Expression<'_>, path: &Path) -> DeadlineDeclaration {
    let span = value.span();
    let value = match unwrap_ts_wrappers(value) {
        Expression::NumericLiteral(number) => finite(number.value),
        Expression::UnaryExpression(unary) => match unwrap_ts_wrappers(&unary.argument) {
            Expression::NumericLiteral(number) => match unary.operator {
                UnaryOperator::UnaryNegation => finite(-number.value),
                UnaryOperator::UnaryPlus => finite(number.value),
                _ => DeadlineValue::Unknown(DeadlineUnknownReason::Expression),
            },
            _ => DeadlineValue::Unknown(DeadlineUnknownReason::Expression),
        },
        _ => DeadlineValue::Unknown(DeadlineUnknownReason::Expression),
    };
    evidence(value, path, span)
}

fn finite(value: f64) -> DeadlineValue {
    if value.is_finite() {
        DeadlineValue::Known(value)
    } else {
        DeadlineValue::Unknown(DeadlineUnknownReason::NonFinite)
    }
}

fn evidence(value: DeadlineValue, path: &Path, span: Span) -> DeadlineDeclaration {
    DeadlineDeclaration {
        value,
        path: path.to_path_buf(),
        span: Some((span.start, span.end)),
        inherited_through: Vec::new(),
    }
}

pub(super) fn obscure(
    slots: &mut DeclaredDeadlines,
    path: &Path,
    span: Span,
    reason: DeadlineUnknownReason,
    hook: bool,
) {
    let declaration = evidence(DeadlineValue::Unknown(reason), path, span);
    slots.case = Some(declaration.clone());
    if hook {
        slots.hook = Some(declaration);
    }
}

pub(super) fn json_declaration(value: &serde_json::Value, path: &Path) -> DeadlineDeclaration {
    let value = value
        .as_f64()
        .map(finite)
        .unwrap_or(DeadlineValue::Unknown(DeadlineUnknownReason::Expression));
    // JSON project interpretation has no AST span; retain path, not a fabricated location.
    let mut declaration = evidence(value, path, Span::new(0, 0));
    declaration.span = None;
    declaration
}

pub(super) fn unknown(
    reason: DeadlineUnknownReason,
    path: &Path,
    span: Option<Span>,
) -> DeadlineDeclaration {
    DeadlineDeclaration {
        value: DeadlineValue::Unknown(reason),
        path: path.to_path_buf(),
        span: span.map(|span| (span.start, span.end)),
        inherited_through: Vec::new(),
    }
}
