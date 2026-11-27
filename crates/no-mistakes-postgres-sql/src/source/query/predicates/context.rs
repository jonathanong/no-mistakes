use super::*;

pub(super) fn opaque_context(
    mut context: PostgresSqlPredicateContext,
) -> PostgresSqlPredicateContext {
    context.mandatory = false;
    context.under_other = true;
    context.effective_mandatory = None;
    context
}

pub(super) fn binary_context(
    mut context: PostgresSqlPredicateContext,
    op: &BinaryOperator,
    not_depth: u32,
) -> PostgresSqlPredicateContext {
    match op {
        BinaryOperator::And if !not_depth.is_multiple_of(2) => {
            context.effective_mandatory = context.effective_mandatory.map(|_| false);
        }
        BinaryOperator::Or => {
            context.mandatory = false;
            context.under_or = true;
            if not_depth.is_multiple_of(2) {
                context.effective_mandatory = context.effective_mandatory.map(|_| false);
            }
        }
        BinaryOperator::And => {}
        _ => return opaque_context(context),
    }
    context
}
