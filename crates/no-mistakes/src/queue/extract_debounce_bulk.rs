use super::members::MemberLookup;
use super::{check_opts_member, emit_verdict, DebounceTtlVerdict};
use crate::queue::extract_helpers::literal_expr;
use crate::queue::extract_job_id::referenced_symbol;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, IdentifierReference,
    ObjectExpression,
};
use oxc_span::GetSpan;

pub(super) fn check<'a>(visitor: &mut QueueVisitor<'a>, call: &CallExpression<'a>) {
    let line = line_number(visitor.source, call.span.start);
    let Some(expr) = call
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        emit_verdict(visitor, DebounceTtlVerdict::Abstain, line, None);
        return;
    };
    if !walk_jobs(visitor, expr, 0) {
        emit_verdict(visitor, DebounceTtlVerdict::Abstain, line, None);
    }
}

fn walk_jobs<'a>(visitor: &mut QueueVisitor<'a>, expr: &Expression<'a>, depth: u8) -> bool {
    if depth > 8 {
        return false;
    }
    match expr.get_inner_expression() {
        Expression::ArrayExpression(array) => {
            let mut resolved = true;
            for element in &array.elements {
                if !walk_element(visitor, element, depth + 1) {
                    resolved = false;
                }
            }
            resolved
        }
        Expression::ObjectExpression(object) => {
            let job = job_name(visitor, object);
            check_opts_member(
                visitor,
                object,
                line_number(visitor.source, object.span.start),
                job,
            );
            true
        }
        Expression::Identifier(id) => match const_init(visitor, id) {
            Some(init) => walk_jobs(visitor, init, depth + 1),
            None => false,
        },
        _ => false,
    }
}

fn walk_element<'a>(
    visitor: &mut QueueVisitor<'a>,
    element: &ArrayExpressionElement<'a>,
    depth: u8,
) -> bool {
    if let ArrayExpressionElement::SpreadElement(spread) = element {
        return walk_jobs(visitor, &spread.argument, depth);
    }
    if matches!(element, ArrayExpressionElement::Elision(_)) {
        return true;
    }
    element
        .as_expression()
        .is_some_and(|expr| walk_jobs(visitor, expr.get_inner_expression(), depth))
}

fn job_name<'a>(visitor: &QueueVisitor<'a>, object: &ObjectExpression<'a>) -> Option<String> {
    match super::members::extract_member(
        object,
        "name",
        &visitor.object_bindings,
        &mut crate::fx::FxHashSet::default(),
    ) {
        MemberLookup::Found(expr) => {
            literal_expr(expr.get_inner_expression(), &visitor.const_strings)
        }
        MemberLookup::Absent | MemberLookup::Unknown => None,
    }
}

fn const_init<'a>(
    visitor: &QueueVisitor<'a>,
    id: &IdentifierReference<'_>,
) -> Option<&'a Expression<'a>> {
    let symbol = referenced_symbol(visitor, id)?;
    let scoping = visitor.scoping?;
    if !scoping.symbol_flags(symbol).is_const_variable()
        || scoping
            .get_resolved_references(symbol)
            .any(|reference| reference.is_write())
    {
        return None;
    }
    let init = visitor
        .object_bindings
        .lookup(id.name.as_str(), id.span.start)?
        .init?;
    if init.span().end > id.span.start {
        return None;
    }
    Some(init)
}
