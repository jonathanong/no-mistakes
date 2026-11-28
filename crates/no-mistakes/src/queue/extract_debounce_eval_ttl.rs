use super::TtlState;
use crate::queue::extract_job_id::referenced_symbol;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

pub(super) fn classify(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> TtlState {
    classify_inner(visitor, expr, 0)
}

fn classify_inner(visitor: &QueueVisitor<'_>, expr: &Expression<'_>, depth: u8) -> TtlState {
    if depth > 8 {
        return TtlState::Dynamic;
    }
    let inner = expr.get_inner_expression();
    if super::unbound_undefined(visitor, inner) {
        return TtlState::Absent;
    }
    if matches!(
        inner,
        Expression::ConditionalExpression(_)
            | Expression::CallExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
            | Expression::ChainExpression(_)
    ) {
        return TtlState::Dynamic;
    }
    let Expression::Identifier(id) = inner else {
        return TtlState::Present;
    };
    let Some(symbol) = referenced_symbol(visitor, id) else {
        return TtlState::Dynamic;
    };
    let Some(scoping) = visitor.scoping else {
        return TtlState::Dynamic;
    };
    if !scoping.symbol_flags(symbol).is_const_variable()
        || scoping
            .get_resolved_references(symbol)
            .any(|reference| reference.is_write())
    {
        return TtlState::Dynamic;
    }
    let Some(init) = visitor
        .object_bindings
        .lookup(id.name.as_str(), id.span.start)
        .and_then(|binding| binding.init)
    else {
        return TtlState::Dynamic;
    };
    if init.span().end > id.span.start {
        return TtlState::Dynamic;
    }
    classify_inner(visitor, init, depth + 1)
}
