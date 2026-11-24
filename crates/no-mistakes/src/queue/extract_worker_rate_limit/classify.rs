use super::predicates::{
    is_class, is_class_or_function, is_function, is_imported_symbol, is_non_worker_expr,
    is_queue_factory_call, is_queue_package_import, is_worker_ctor_expr, is_worker_factory_call,
    is_worker_new, namespace_member, referenced_symbol, unwrap_worker_init,
};
use super::{LocalInit, PendingReceiver};
use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{CallExpression, Expression, IdentifierReference, NewExpression};

pub(super) fn rate_limit_receiver<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    match call.callee.get_inner_expression() {
        Expression::StaticMemberExpression(member) if member.property.name == "rateLimit" => {
            Some(member.object.get_inner_expression())
        }
        Expression::ComputedMemberExpression(member)
            if crate::ast::computed_property_name(member).as_deref() == Some("rateLimit") =>
        {
            Some(member.object.get_inner_expression())
        }
        _ => None,
    }
}

pub(super) fn classify_receiver(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
) -> Option<PendingReceiver> {
    let expr = unwrap_worker_init(expr);
    match expr {
        Expression::NewExpression(new_expr) => classify_new_receiver(visitor, new_expr),
        Expression::CallExpression(call_expr) => classify_call_receiver(visitor, call_expr),
        Expression::Identifier(id) => classify_ident_receiver(visitor, id),
        Expression::StaticMemberExpression(member) => {
            let (source, exported) = namespace_member(visitor, member)?;
            (!is_queue_package(&source)).then_some(PendingReceiver::Namespace { source, exported })
        }
        _ => None,
    }
}

fn classify_new_receiver(
    visitor: &QueueVisitor<'_>,
    new_expr: &NewExpression<'_>,
) -> Option<PendingReceiver> {
    if is_worker_new(visitor, new_expr) {
        return Some(PendingReceiver::WorkerSpan(new_expr.span.start));
    }
    match new_expr.callee.get_inner_expression() {
        Expression::Identifier(id) => {
            let symbol = referenced_symbol(visitor, id)?;
            if is_imported_symbol(visitor, symbol) {
                let name = id.name.as_str();
                (!visitor.namespace_imports.contains_key(name)
                    && !is_queue_package_import(visitor, name))
                .then(|| PendingReceiver::ConstructedImported(name.to_string()))
            } else if is_class(visitor, symbol) {
                Some(PendingReceiver::ConstructedLocalClass(symbol))
            } else if !is_function(visitor, symbol) {
                Some(PendingReceiver::ConstructedLocalBinding(symbol))
            } else {
                None
            }
        }
        Expression::StaticMemberExpression(member) => {
            let (source, exported) = namespace_member(visitor, member)?;
            (!is_queue_package(&source))
                .then_some(PendingReceiver::ConstructedNamespace { source, exported })
        }
        _ => None,
    }
}

fn classify_call_receiver(
    visitor: &QueueVisitor<'_>,
    call_expr: &CallExpression<'_>,
) -> Option<PendingReceiver> {
    if is_worker_factory_call(visitor, call_expr) {
        return Some(PendingReceiver::WorkerSpan(call_expr.span.start));
    }
    if is_queue_factory_call(visitor, call_expr) {
        return None;
    }
    match call_expr.callee.get_inner_expression() {
        Expression::Identifier(id) => {
            let symbol = referenced_symbol(visitor, id)?;
            let name = id.name.as_str();
            (is_imported_symbol(visitor, symbol)
                && !visitor.namespace_imports.contains_key(name)
                && !is_queue_package_import(visitor, name))
            .then(|| PendingReceiver::CallImported(name.to_string()))
        }
        Expression::StaticMemberExpression(member) => {
            let (source, exported) = namespace_member(visitor, member)?;
            (!is_queue_package(&source))
                .then_some(PendingReceiver::CallNamespace { source, exported })
        }
        _ => None,
    }
}

fn classify_ident_receiver(
    visitor: &QueueVisitor<'_>,
    id: &IdentifierReference<'_>,
) -> Option<PendingReceiver> {
    let symbol = referenced_symbol(visitor, id)?;
    if is_imported_symbol(visitor, symbol) {
        let name = id.name.as_str();
        return (!visitor.namespace_imports.contains_key(name)
            && !is_queue_package_import(visitor, name))
        .then(|| PendingReceiver::Imported(name.to_string()));
    }
    if is_class_or_function(visitor, symbol) {
        return None;
    }
    Some(PendingReceiver::Local(symbol))
}

pub(super) fn classify_local_init(
    visitor: &QueueVisitor<'_>,
    init: &Expression<'_>,
) -> Option<LocalInit> {
    let expr = unwrap_worker_init(init);
    if is_worker_ctor_expr(visitor, expr) {
        return Some(LocalInit::WorkerClass);
    }
    if is_non_worker_expr(expr) {
        return Some(LocalInit::NonWorker);
    }
    match expr {
        Expression::NewExpression(new_expr) => {
            Some(match classify_new_receiver(visitor, new_expr) {
                Some(PendingReceiver::WorkerSpan(span)) => LocalInit::WorkerSpan(span),
                Some(PendingReceiver::ConstructedLocalClass(sym)) => {
                    LocalInit::ConstructedLocalClass(sym)
                }
                Some(PendingReceiver::ConstructedLocalBinding(sym)) => {
                    LocalInit::ConstructedLocalBinding(sym)
                }
                Some(PendingReceiver::ConstructedImported(local)) => {
                    LocalInit::ConstructedImported(local)
                }
                Some(PendingReceiver::ConstructedNamespace { source, exported }) => {
                    LocalInit::ConstructedNamespace { source, exported }
                }
                _ => LocalInit::NonWorker,
            })
        }
        Expression::CallExpression(call_expr) => match classify_call_receiver(visitor, call_expr) {
            Some(PendingReceiver::WorkerSpan(span)) => Some(LocalInit::WorkerSpan(span)),
            Some(PendingReceiver::CallImported(local)) => Some(LocalInit::CallImported(local)),
            Some(PendingReceiver::CallNamespace { source, exported }) => {
                Some(LocalInit::CallNamespace { source, exported })
            }
            _ => None,
        },
        Expression::Identifier(id) => {
            let symbol = referenced_symbol(visitor, id)?;
            if is_imported_symbol(visitor, symbol) {
                let name = id.name.as_str();
                return Some(
                    if visitor.namespace_imports.contains_key(name)
                        || is_queue_package_import(visitor, name)
                    {
                        LocalInit::NonWorker
                    } else {
                        LocalInit::Imported(name.to_string())
                    },
                );
            }
            Some(LocalInit::Alias(symbol))
        }
        Expression::StaticMemberExpression(member) => {
            let (source, exported) = namespace_member(visitor, member)?;
            Some(if is_queue_package(&source) {
                LocalInit::NonWorker
            } else {
                LocalInit::Namespace { source, exported }
            })
        }
        _ => None,
    }
}
