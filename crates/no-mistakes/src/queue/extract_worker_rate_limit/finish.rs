use super::predicates::is_class_or_function;
use super::{LocalInit, ModuleInit, PendingReceiver, State};
use crate::queue::extract_model::{
    ExportedWorkerBinding, WorkerRateLimitCall, WorkerRateLimitTarget,
};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_syntax::symbol::SymbolId;
use std::collections::HashSet;

pub(super) fn finish(visitor: &mut QueueVisitor<'_>) {
    let mut state = std::mem::take(&mut visitor.worker_rate_limit);
    finish_module_exports(visitor, &state);
    for call in std::mem::take(&mut state.pending_calls) {
        if let Some(target) = resolve_pending(visitor, &state, call.receiver) {
            visitor.facts.worker_rate_limits.push(WorkerRateLimitCall {
                line: call.line,
                target,
            });
        }
    }
}

fn finish_module_exports(visitor: &mut QueueVisitor<'_>, state: &State) {
    for name in state.module_bindings.keys() {
        let mut visited = HashSet::new();
        match resolve_module_binding(name, state, &mut visited) {
            Some(ResolvedModule::Binding(binding)) => {
                if name != "default" {
                    visitor.facts.worker_bindings.insert(name.clone(), binding);
                }
                visitor.facts.worker_exports.insert(name.clone(), binding);
            }
            Some(ResolvedModule::Imported(local)) => {
                visitor
                    .facts
                    .worker_import_aliases
                    .insert(name.clone(), local);
            }
            None => {}
        }
    }
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
}

enum ResolvedModule {
    Binding(ExportedWorkerBinding),
    Imported(String),
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
            .map(|idx| ResolvedModule::Binding(ExportedWorkerBinding::Worker(idx))),
        ModuleInit::NonWorker => Some(ResolvedModule::Binding(ExportedWorkerBinding::NonWorker)),
        ModuleInit::Imported(local) => Some(ResolvedModule::Imported(local.clone())),
        ModuleInit::Alias(next) => resolve_module_binding(next, state, visited),
    }
}

fn resolve_pending(
    visitor: &QueueVisitor<'_>,
    state: &State,
    receiver: PendingReceiver,
) -> Option<WorkerRateLimitTarget> {
    match receiver {
        PendingReceiver::WorkerSpan(span) => Some(WorkerRateLimitTarget::LocalWorker(
            state.worker_call_indices.get(&span).copied(),
        )),
        PendingReceiver::Imported(local) => Some(WorkerRateLimitTarget::Imported { local }),
        PendingReceiver::Namespace { source, exported } => {
            Some(WorkerRateLimitTarget::Namespace { source, exported })
        }
        PendingReceiver::Unresolved => Some(WorkerRateLimitTarget::Unresolved),
        PendingReceiver::Local(symbol) => resolve_local_symbol(visitor, state, symbol),
    }
}

fn resolve_local_symbol(
    visitor: &QueueVisitor<'_>,
    state: &State,
    mut current: SymbolId,
) -> Option<WorkerRateLimitTarget> {
    let mut visited = HashSet::new();
    while visited.insert(current) {
        let Some(init) = state.local_bindings.get(&current) else {
            return Some(WorkerRateLimitTarget::Unresolved);
        };
        match init {
            LocalInit::WorkerSpan(span) => {
                return Some(WorkerRateLimitTarget::LocalWorker(
                    state.worker_call_indices.get(span).copied(),
                ));
            }
            LocalInit::NonWorker => return None,
            LocalInit::Imported(local) => {
                return Some(WorkerRateLimitTarget::Imported {
                    local: local.clone(),
                });
            }
            LocalInit::Namespace { source, exported } => {
                return Some(WorkerRateLimitTarget::Namespace {
                    source: source.clone(),
                    exported: exported.clone(),
                });
            }
            LocalInit::Alias(next) => {
                if is_class_or_function(visitor, *next) {
                    return None;
                }
                current = *next;
            }
        }
    }
    Some(WorkerRateLimitTarget::Unresolved)
}
