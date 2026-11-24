use super::classes::{resolve_module_class, ResolvedModuleClass};
use super::{ModuleInit, State};
use crate::queue::extract_model::ExportedWorkerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use std::collections::HashSet;

pub(super) fn finish_module_classes(visitor: &mut QueueVisitor<'_>, state: &State) {
    for name in state.module_classes.keys() {
        let mut visited = HashSet::new();
        match resolve_module_class(name, state, &mut visited) {
            Some(ResolvedModuleClass::WorkerClass) => {
                record_binding(visitor, name, ExportedWorkerBinding::WorkerClass);
            }
            Some(ResolvedModuleClass::NonWorker) => {
                record_binding(visitor, name, ExportedWorkerBinding::NonWorker);
            }
            Some(ResolvedModuleClass::Imported(local)) => {
                visitor
                    .facts
                    .worker_class_imports
                    .insert(name.clone(), local);
            }
            Some(ResolvedModuleClass::Namespace(source, exported)) => {
                visitor
                    .facts
                    .worker_class_namespaces
                    .insert(name.clone(), (source, exported));
            }
            None => {}
        }
    }
}

pub(super) fn finish_module_exports(visitor: &mut QueueVisitor<'_>, state: &State) {
    for name in state.module_bindings.keys() {
        let mut visited = HashSet::new();
        match resolve_module_binding(name, state, &mut visited) {
            Some(ResolvedModule::Binding(binding)) => record_binding(visitor, name, binding),
            Some(ResolvedModule::Imported(local)) => {
                visitor
                    .facts
                    .worker_import_aliases
                    .insert(name.clone(), local);
            }
            Some(ResolvedModule::Namespace(source, exported)) => {
                visitor
                    .facts
                    .worker_namespace_exports
                    .insert(name.clone(), (source, exported));
            }
            Some(ResolvedModule::ConstructedImported(local)) => {
                visitor
                    .facts
                    .worker_constructed_aliases
                    .insert(name.clone(), local);
            }
            Some(ResolvedModule::ConstructedNamespace(source, exported)) => {
                visitor
                    .facts
                    .worker_constructed_namespaces
                    .insert(name.clone(), (source, exported));
            }
            None => {}
        }
    }
    copy_reexports(visitor);
}

fn record_binding(visitor: &mut QueueVisitor<'_>, name: &str, binding: ExportedWorkerBinding) {
    if name != "default" {
        visitor
            .facts
            .worker_bindings
            .insert(name.to_string(), binding);
    }
    visitor
        .facts
        .worker_exports
        .insert(name.to_string(), binding);
}

fn copy_reexports(visitor: &mut QueueVisitor<'_>) {
    let copied: Vec<_> = visitor
        .facts
        .reexports
        .iter()
        .filter_map(|(exported, local)| {
            visitor
                .facts
                .worker_bindings
                .get(local)
                .copied()
                .map(|binding| (exported.clone(), binding))
        })
        .collect();
    for (exported, binding) in copied {
        visitor.facts.worker_exports.insert(exported, binding);
    }
    let copied_ns: Vec<_> = visitor
        .facts
        .reexports
        .iter()
        .filter_map(|(exported, local)| {
            visitor
                .facts
                .worker_namespace_exports
                .get(local)
                .cloned()
                .map(|target| (exported.clone(), target))
        })
        .collect();
    for (exported, target) in copied_ns {
        visitor
            .facts
            .worker_namespace_exports
            .insert(exported, target);
    }
    let copied_class: Vec<_> = visitor
        .facts
        .reexports
        .iter()
        .filter_map(|(exported, local)| {
            visitor
                .facts
                .worker_class_imports
                .get(local)
                .cloned()
                .map(|target| (exported.clone(), target))
        })
        .collect();
    for (exported, target) in copied_class {
        visitor.facts.worker_class_imports.insert(exported, target);
    }
}

enum ResolvedModule {
    Binding(ExportedWorkerBinding),
    Imported(String),
    Namespace(String, String),
    ConstructedImported(String),
    ConstructedNamespace(String, String),
}

fn resolve_module_binding(
    name: &str,
    state: &State,
    visited: &mut HashSet<String>,
) -> Option<ResolvedModule> {
    if !visited.insert(name.to_string()) {
        return None;
    }
    match state.module_bindings.get(name)? {
        ModuleInit::WorkerSpan(span) => state
            .worker_call_indices
            .get(span)
            .copied()
            .map(|idx| ResolvedModule::Binding(ExportedWorkerBinding::Worker(Some(idx)))),
        ModuleInit::NonWorker => Some(ResolvedModule::Binding(ExportedWorkerBinding::NonWorker)),
        ModuleInit::Imported(local) => Some(ResolvedModule::Imported(local.clone())),
        ModuleInit::Namespace { source, exported } => {
            Some(ResolvedModule::Namespace(source.clone(), exported.clone()))
        }
        ModuleInit::ConstructedImported(local) => {
            Some(ResolvedModule::ConstructedImported(local.clone()))
        }
        ModuleInit::ConstructedNamespace { source, exported } => Some(
            ResolvedModule::ConstructedNamespace(source.clone(), exported.clone()),
        ),
        ModuleInit::ConstructedLocalClass(class_name) => {
            let mut class_visited = HashSet::new();
            match resolve_module_class(class_name, state, &mut class_visited) {
                Some(ResolvedModuleClass::WorkerClass) => {
                    Some(ResolvedModule::Binding(ExportedWorkerBinding::Worker(None)))
                }
                Some(ResolvedModuleClass::NonWorker) => {
                    Some(ResolvedModule::Binding(ExportedWorkerBinding::NonWorker))
                }
                Some(ResolvedModuleClass::Imported(local)) => {
                    Some(ResolvedModule::ConstructedImported(local))
                }
                Some(ResolvedModuleClass::Namespace(source, exported)) => {
                    Some(ResolvedModule::ConstructedNamespace(source, exported))
                }
                None => None,
            }
        }
        ModuleInit::Alias(next) => resolve_module_binding(next, state, visited),
    }
}
