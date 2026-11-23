use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_job_id::is_imported_callee;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    CallExpression, Expression, IdentifierReference, NewExpression, StaticMemberExpression,
};
use oxc_syntax::symbol::SymbolId;

pub(in crate::queue) fn is_worker_constructor_callee(
    visitor: &QueueVisitor<'_>,
    callee: &Expression<'_>,
) -> bool {
    match callee.get_inner_expression() {
        Expression::Identifier(id) => visitor.worker_classes.contains(id.name.as_str()),
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(ns) = member.object.get_inner_expression() else {
                return false;
            };
            matches!(member.property.name.as_str(), "Worker" | "TestWorker")
                && visitor
                    .namespace_imports
                    .get(ns.name.as_str())
                    .is_some_and(|source| is_queue_package(source))
        }
        _ => false,
    }
}

pub(super) fn is_worker_new(visitor: &QueueVisitor<'_>, new_expr: &NewExpression<'_>) -> bool {
    let callee = new_expr.callee.get_inner_expression();
    match callee {
        Expression::Identifier(id) => {
            visitor.worker_classes.contains(id.name.as_str())
                && (visitor.scoping.is_none() || is_imported_callee(visitor, callee))
        }
        Expression::StaticMemberExpression(member) => {
            is_worker_constructor_callee(visitor, callee)
                && (visitor.scoping.is_none() || is_imported_callee(visitor, &member.object))
        }
        _ => false,
    }
}

pub(super) fn is_worker_factory_call(
    visitor: &QueueVisitor<'_>,
    call: &CallExpression<'_>,
) -> bool {
    let callee = call.callee.get_inner_expression();
    matches!(callee, Expression::Identifier(id)
        if visitor.worker_factories.contains_key(id.name.as_str())
            && (visitor.scoping.is_none() || is_imported_callee(visitor, callee)))
}

pub(super) fn is_queue_factory_call(visitor: &QueueVisitor<'_>, call: &CallExpression<'_>) -> bool {
    let callee = call.callee.get_inner_expression();
    matches!(callee, Expression::Identifier(id)
        if visitor.queue_name_index.contains_key(id.name.as_str())
            && (visitor.scoping.is_none() || is_imported_callee(visitor, callee)))
}

pub(super) fn namespace_member(
    visitor: &QueueVisitor<'_>,
    member: &StaticMemberExpression<'_>,
) -> Option<(String, String)> {
    let Expression::Identifier(id) = member.object.get_inner_expression() else {
        return None;
    };
    let source = visitor.namespace_imports.get(id.name.as_str())?.clone();
    if visitor.scoping.is_some() && !is_imported_callee(visitor, &member.object) {
        return None;
    }
    Some((source, member.property.name.to_string()))
}

pub(super) fn unwrap_worker_init<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr.get_inner_expression() {
        Expression::AwaitExpression(awaited) => awaited.argument.get_inner_expression(),
        other => other,
    }
}

pub(super) fn is_non_worker_expr(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
    )
}

pub(super) fn referenced_symbol(
    visitor: &QueueVisitor<'_>,
    id: &IdentifierReference<'_>,
) -> Option<SymbolId> {
    visitor
        .scoping?
        .get_reference(id.reference_id.get()?)
        .symbol_id()
}

pub(super) fn is_imported_symbol(visitor: &QueueVisitor<'_>, symbol: SymbolId) -> bool {
    visitor.scoping.is_some_and(|scoping| {
        let flags = scoping.symbol_flags(symbol);
        flags.is_import() && !flags.is_type_import()
    })
}

pub(super) fn is_class_or_function(visitor: &QueueVisitor<'_>, symbol: SymbolId) -> bool {
    visitor.scoping.is_some_and(|scoping| {
        let flags = scoping.symbol_flags(symbol);
        flags.is_class() || flags.is_function()
    })
}
