use super::*;

/// A conditional element keeps both arms. Direct call arguments already expand
/// in the escaped-argument walk; named arrays only see the stored value.
pub(in super::super) fn value(
    collector: &ImportCollector,
    expression: &Expression<'_>,
) -> ValueExpression {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ConditionalExpression(conditional) => ValueExpression::Either(
            std::sync::Arc::new(value(collector, &conditional.consequent)),
            std::sync::Arc::new(value(collector, &conditional.alternate)),
        ),
        other => collector.value_expression(other),
    }
}
