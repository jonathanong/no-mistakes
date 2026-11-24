use super::predicates::{
    is_class_or_function, is_imported_symbol, is_known_non_worker_new, is_non_worker_expr,
    is_queue_factory_call, is_worker_factory_call, is_worker_new, namespace_member,
    referenced_symbol, unwrap_worker_init,
};
use super::{LocalInit, ModuleInit, PendingReceiver};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{CallExpression, Expression, IdentifierReference};

enum DirectKind {
    WorkerSpan(u32),
    NonWorker,
}

fn classify_direct(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> Option<DirectKind> {
    match expr {
        Expression::NewExpression(new_expr) => {
            if is_worker_new(visitor, new_expr) {
                Some(DirectKind::WorkerSpan(new_expr.span.start))
            } else if is_known_non_worker_new(visitor, new_expr) {
                Some(DirectKind::NonWorker)
            } else {
                None
            }
        }
        Expression::CallExpression(call_expr) => {
            if is_worker_factory_call(visitor, call_expr) {
                Some(DirectKind::WorkerSpan(call_expr.span.start))
            } else if is_queue_factory_call(visitor, call_expr) {
                Some(DirectKind::NonWorker)
            } else {
                None
            }
        }
        other if is_non_worker_expr(other) => Some(DirectKind::NonWorker),
        _ => None,
    }
}

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
    if let Some(direct) = classify_direct(visitor, expr) {
        return match direct {
            DirectKind::WorkerSpan(span) => Some(PendingReceiver::WorkerSpan(span)),
            DirectKind::NonWorker => None,
        };
    }
    match expr {
        Expression::Identifier(id) => classify_ident_receiver(visitor, id),
        Expression::StaticMemberExpression(member) => Some(
            namespace_member(visitor, member)
                .map(|(source, exported)| PendingReceiver::Namespace { source, exported })
                .unwrap_or(PendingReceiver::Unresolved),
        ),
        _ => Some(PendingReceiver::Unresolved),
    }
}

fn classify_ident_receiver(
    visitor: &QueueVisitor<'_>,
    id: &IdentifierReference<'_>,
) -> Option<PendingReceiver> {
    let Some(symbol) = referenced_symbol(visitor, id) else {
        return Some(PendingReceiver::Unresolved);
    };
    if is_imported_symbol(visitor, symbol) {
        return (!visitor.namespace_imports.contains_key(id.name.as_str()))
            .then(|| PendingReceiver::Imported(id.name.to_string()));
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
    if let Some(direct) = classify_direct(visitor, expr) {
        return Some(match direct {
            DirectKind::WorkerSpan(span) => LocalInit::WorkerSpan(span),
            DirectKind::NonWorker => LocalInit::NonWorker,
        });
    }
    match expr {
        Expression::Identifier(id) => {
            let symbol = referenced_symbol(visitor, id)?;
            if is_imported_symbol(visitor, symbol) {
                return Some(
                    if visitor.namespace_imports.contains_key(id.name.as_str()) {
                        LocalInit::NonWorker
                    } else {
                        LocalInit::Imported(id.name.to_string())
                    },
                );
            }
            Some(LocalInit::Alias(symbol))
        }
        Expression::StaticMemberExpression(member) => namespace_member(visitor, member)
            .map(|(source, exported)| LocalInit::Namespace { source, exported }),
        _ => None,
    }
}

pub(super) fn classify_module_init(
    visitor: &QueueVisitor<'_>,
    init: &Expression<'_>,
) -> Option<ModuleInit> {
    let expr = unwrap_worker_init(init);
    if let Some(direct) = classify_direct(visitor, expr) {
        return Some(match direct {
            DirectKind::WorkerSpan(span) => ModuleInit::WorkerSpan(span),
            DirectKind::NonWorker => ModuleInit::NonWorker,
        });
    }
    match expr {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            Some(if visitor.namespace_imports.contains_key(name) {
                ModuleInit::NonWorker
            } else if visitor.facts.imports.iter().any(|imp| imp.local == name) {
                ModuleInit::Imported(name.to_string())
            } else {
                ModuleInit::Alias(name.to_string())
            })
        }
        Expression::StaticMemberExpression(member) => namespace_member(visitor, member)
            .map(|(source, exported)| ModuleInit::Namespace { source, exported }),
        _ => None,
    }
}
