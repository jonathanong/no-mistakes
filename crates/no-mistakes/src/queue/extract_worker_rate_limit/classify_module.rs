use super::predicates::{
    is_non_worker_expr, is_queue_factory_call, is_queue_package_import, is_worker_factory_call,
    is_worker_new, namespace_member, unwrap_worker_init,
};
use super::ModuleInit;
use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::Expression;

pub(super) fn classify_module_init(
    visitor: &QueueVisitor<'_>,
    init: &Expression<'_>,
) -> Option<ModuleInit> {
    let expr = unwrap_worker_init(init);
    if is_non_worker_expr(expr) {
        return Some(ModuleInit::NonWorker);
    }
    match expr {
        Expression::NewExpression(new_expr) => {
            if is_worker_new(visitor, new_expr) {
                return Some(ModuleInit::WorkerSpan(new_expr.span.start));
            }
            match new_expr.callee.get_inner_expression() {
                Expression::Identifier(id) => {
                    let name = id.name.as_str();
                    if visitor.namespace_imports.contains_key(name)
                        || is_queue_package_import(visitor, name)
                    {
                        Some(ModuleInit::NonWorker)
                    } else if visitor.facts.imports.iter().any(|imp| imp.local == name) {
                        Some(ModuleInit::ConstructedImported(name.to_string()))
                    } else {
                        Some(ModuleInit::ConstructedLocalClass(name.to_string()))
                    }
                }
                Expression::StaticMemberExpression(member) => {
                    let (source, exported) = namespace_member(visitor, member)?;
                    Some(if is_queue_package(&source) {
                        ModuleInit::NonWorker
                    } else {
                        ModuleInit::ConstructedNamespace { source, exported }
                    })
                }
                _ => None,
            }
        }
        Expression::CallExpression(call_expr) => {
            if is_worker_factory_call(visitor, call_expr) {
                Some(ModuleInit::WorkerSpan(call_expr.span.start))
            } else if is_queue_factory_call(visitor, call_expr) {
                Some(ModuleInit::NonWorker)
            } else {
                None
            }
        }
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            Some(
                if visitor.namespace_imports.contains_key(name)
                    || is_queue_package_import(visitor, name)
                {
                    ModuleInit::NonWorker
                } else if visitor.facts.imports.iter().any(|imp| imp.local == name) {
                    ModuleInit::Imported(name.to_string())
                } else {
                    ModuleInit::Alias(name.to_string())
                },
            )
        }
        Expression::StaticMemberExpression(member) => {
            let (source, exported) = namespace_member(visitor, member)?;
            Some(if is_queue_package(&source) {
                ModuleInit::NonWorker
            } else {
                ModuleInit::Namespace { source, exported }
            })
        }
        _ => None,
    }
}
