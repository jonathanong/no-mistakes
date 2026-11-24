use super::classes::{resolve_module_class, ResolvedModuleClass};
use super::{ModuleInit, State};
use crate::queue::extract_model::ExportedWorkerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use std::collections::{HashMap, HashSet};

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
        let Some(resolved) = resolve_module_binding(name, state, &mut visited) else {
            continue;
        };
        let facts = &mut visitor.facts;
        match resolved {
            ResolvedModule::Binding(binding) => record_binding(visitor, name, binding),
            ResolvedModule::Imported(local) => {
                facts.worker_import_aliases.insert(name.clone(), local);
            }
            ResolvedModule::Namespace(s, e) => {
                facts.worker_namespace_exports.insert(name.clone(), (s, e));
            }
            ResolvedModule::ClassImported(local) => {
                facts.worker_class_imports.insert(name.clone(), local);
            }
            ResolvedModule::ClassNamespace(s, e) => {
                facts.worker_class_namespaces.insert(name.clone(), (s, e));
            }
            ResolvedModule::ConstructedImported(local) => {
                facts.worker_constructed_aliases.insert(name.clone(), local);
            }
            ResolvedModule::ConstructedNamespace(s, e) => {
                facts
                    .worker_constructed_namespaces
                    .insert(name.clone(), (s, e));
            }
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
    let reexports = &visitor.facts.reexports;
    let copied_bindings: Vec<_> = reexports
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
    visitor.facts.worker_exports.extend(copied_bindings);
    copy_reexport_in_place(reexports, &mut visitor.facts.worker_import_aliases);
    copy_reexport_in_place(reexports, &mut visitor.facts.worker_namespace_exports);
    copy_reexport_in_place(reexports, &mut visitor.facts.worker_class_imports);
    copy_reexport_in_place(reexports, &mut visitor.facts.worker_class_namespaces);
    copy_reexport_in_place(reexports, &mut visitor.facts.worker_constructed_aliases);
    copy_reexport_in_place(reexports, &mut visitor.facts.worker_constructed_namespaces);
}

fn copy_reexport_in_place<V: Clone>(reexports: &[(String, String)], map: &mut HashMap<String, V>) {
    let copied: Vec<_> = reexports
        .iter()
        .filter_map(|(exported, local)| map.get(local).cloned().map(|v| (exported.clone(), v)))
        .collect();
    map.extend(copied);
}

enum ResolvedModule {
    Binding(ExportedWorkerBinding),
    Imported(String),
    Namespace(String, String),
    ClassImported(String),
    ClassNamespace(String, String),
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
            resolve_constructed_local(class_name, state, visited)
        }
        ModuleInit::Alias(next) => resolve_module_binding(next, state, visited)
            .or_else(|| resolve_module_class_binding(next, state)),
    }
}

fn resolve_module_class_binding(name: &str, state: &State) -> Option<ResolvedModule> {
    let mut visited = HashSet::new();
    match resolve_module_class(name, state, &mut visited)? {
        ResolvedModuleClass::WorkerClass => {
            Some(ResolvedModule::Binding(ExportedWorkerBinding::WorkerClass))
        }
        ResolvedModuleClass::NonWorker => {
            Some(ResolvedModule::Binding(ExportedWorkerBinding::NonWorker))
        }
        ResolvedModuleClass::Imported(local) => Some(ResolvedModule::ClassImported(local)),
        ResolvedModuleClass::Namespace(source, exported) => {
            Some(ResolvedModule::ClassNamespace(source, exported))
        }
    }
}

fn resolve_constructed_local(
    class_name: &str,
    state: &State,
    visited: &mut HashSet<String>,
) -> Option<ResolvedModule> {
    let resolved = resolve_module_class_binding(class_name, state)
        .or_else(|| resolve_module_binding(class_name, state, visited))?;
    match resolved {
        ResolvedModule::Binding(ExportedWorkerBinding::WorkerClass) => {
            Some(ResolvedModule::Binding(ExportedWorkerBinding::Worker(None)))
        }
        ResolvedModule::Binding(ExportedWorkerBinding::NonWorker) => {
            Some(ResolvedModule::Binding(ExportedWorkerBinding::NonWorker))
        }
        ResolvedModule::Imported(local) | ResolvedModule::ClassImported(local) => {
            Some(ResolvedModule::ConstructedImported(local))
        }
        ResolvedModule::Namespace(source, exported)
        | ResolvedModule::ClassNamespace(source, exported) => {
            Some(ResolvedModule::ConstructedNamespace(source, exported))
        }
        _ => None,
    }
}
