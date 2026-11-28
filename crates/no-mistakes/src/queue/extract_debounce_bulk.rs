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
            Some((init, stable)) => walk_jobs(visitor, init, depth + 1) && stable,
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
        &mut 32,
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
) -> Option<(&'a Expression<'a>, bool)> {
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
    let stable = !mutated_before(
        visitor.source,
        id.name.as_str(),
        init.span().end,
        id.span.start,
    );
    Some((init, stable))
}

fn mutated_before(source: &str, name: &str, from: u32, to: u32) -> bool {
    let Some(window) = source.get(from as usize..to as usize) else {
        return true;
    };
    let name_bytes = name.as_bytes();
    let bytes = window.as_bytes();
    let mut index = 0;
    while index + name_bytes.len() <= bytes.len() {
        let matched = &bytes[index..index + name_bytes.len()] == name_bytes
            && (index == 0 || !is_ident_byte(bytes[index - 1]));
        if matched {
            let after = window[index + name_bytes.len()..].trim_start();
            if after.starts_with('.') || after.starts_with('[') || after.starts_with("?.") {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn is_ident_byte(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}
