use super::predicates::{
    is_class, is_imported_symbol, is_queue_package_import, is_worker_ctor_expr, namespace_member,
    referenced_symbol,
};
use super::{LocalClassInit, ModuleClassInit, State};
use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_model::WorkerRateLimitTarget;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{Class, Expression};
use oxc_syntax::symbol::SymbolId;
use std::collections::HashSet;

pub(super) fn classify_local_class(
    visitor: &QueueVisitor<'_>,
    class: &Class<'_>,
) -> LocalClassInit {
    let Some(heritage) = &class.heritage else {
        return LocalClassInit::NonWorker;
    };
    let expr = heritage.expression.get_inner_expression();
    if is_worker_ctor_expr(visitor, expr) {
        return LocalClassInit::WorkerClass;
    }
    match expr {
        Expression::Identifier(id) => {
            let Some(symbol) = referenced_symbol(visitor, id) else {
                return LocalClassInit::Dynamic;
            };
            if is_imported_symbol(visitor, symbol) {
                let name = id.name.as_str();
                if visitor.namespace_imports.contains_key(name)
                    || is_queue_package_import(visitor, name)
                {
                    LocalClassInit::NonWorker
                } else {
                    LocalClassInit::ExtendsImported(name.to_string())
                }
            } else if is_class(visitor, symbol) {
                LocalClassInit::ExtendsLocal(symbol)
            } else {
                LocalClassInit::Dynamic
            }
        }
        Expression::StaticMemberExpression(member) => {
            if let Some((source, exported)) = namespace_member(visitor, member) {
                if is_queue_package(&source) {
                    LocalClassInit::NonWorker
                } else {
                    LocalClassInit::ExtendsNamespace { source, exported }
                }
            } else {
                LocalClassInit::Dynamic
            }
        }
        _ => LocalClassInit::Dynamic,
    }
}

pub(super) fn classify_module_class(
    visitor: &QueueVisitor<'_>,
    class: &Class<'_>,
) -> ModuleClassInit {
    let Some(heritage) = &class.heritage else {
        return ModuleClassInit::NonWorker;
    };
    let expr = heritage.expression.get_inner_expression();
    if is_worker_ctor_expr(visitor, expr) {
        return ModuleClassInit::WorkerClass;
    }
    match expr {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            if visitor.namespace_imports.contains_key(name)
                || is_queue_package_import(visitor, name)
            {
                ModuleClassInit::NonWorker
            } else if visitor.facts.imports.iter().any(|imp| imp.local == name) {
                ModuleClassInit::ExtendsImported(name.to_string())
            } else {
                ModuleClassInit::ExtendsLocal(name.to_string())
            }
        }
        Expression::StaticMemberExpression(member) => {
            if let Some((source, exported)) = namespace_member(visitor, member) {
                if is_queue_package(&source) {
                    ModuleClassInit::NonWorker
                } else {
                    ModuleClassInit::ExtendsNamespace { source, exported }
                }
            } else {
                ModuleClassInit::Dynamic
            }
        }
        _ => ModuleClassInit::Dynamic,
    }
}

pub(super) fn resolve_local_class(
    visitor: &QueueVisitor<'_>,
    state: &State,
    mut current: SymbolId,
) -> Option<WorkerRateLimitTarget> {
    let mut visited = HashSet::new();
    while visited.insert(current) {
        match state.local_classes.get(&current)? {
            LocalClassInit::WorkerClass => return Some(WorkerRateLimitTarget::LocalWorker(None)),
            LocalClassInit::NonWorker => return None,
            LocalClassInit::ExtendsLocal(next) => current = *next,
            LocalClassInit::ExtendsImported(local) => {
                return Some(WorkerRateLimitTarget::ConstructedImported {
                    local: local.clone(),
                });
            }
            LocalClassInit::ExtendsNamespace { source, exported } => {
                return Some(WorkerRateLimitTarget::ConstructedNamespace {
                    source: source.clone(),
                    exported: exported.clone(),
                });
            }
            LocalClassInit::Dynamic => {
                return visitor
                    .facts
                    .has_queue_package_import
                    .then_some(WorkerRateLimitTarget::Unresolved);
            }
        }
    }
    None
}

pub(super) enum ResolvedModuleClass {
    WorkerClass,
    NonWorker,
    Imported(String),
    Namespace(String, String),
}

pub(super) fn resolve_module_class(
    name: &str,
    state: &State,
    visited: &mut HashSet<String>,
) -> Option<ResolvedModuleClass> {
    if !visited.insert(name.to_string()) {
        return None;
    }
    match state.module_classes.get(name)? {
        ModuleClassInit::WorkerClass => Some(ResolvedModuleClass::WorkerClass),
        ModuleClassInit::NonWorker => Some(ResolvedModuleClass::NonWorker),
        ModuleClassInit::ExtendsImported(local) => {
            Some(ResolvedModuleClass::Imported(local.clone()))
        }
        ModuleClassInit::ExtendsNamespace { source, exported } => Some(
            ResolvedModuleClass::Namespace(source.clone(), exported.clone()),
        ),
        ModuleClassInit::ExtendsLocal(next) => resolve_module_class(next, state, visited),
        ModuleClassInit::Dynamic => None,
    }
}
