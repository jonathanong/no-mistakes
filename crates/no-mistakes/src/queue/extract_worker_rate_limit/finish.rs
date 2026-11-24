use super::classes::resolve_local_class;
use super::finish_exports::{finish_module_classes, finish_module_exports};
use super::predicates::is_class_or_function;
use super::{LocalInit, PendingReceiver, State};
use crate::queue::extract_model::{WorkerRateLimitCall, WorkerRateLimitTarget};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_syntax::symbol::SymbolId;
use std::collections::HashSet;

pub(super) fn finish(visitor: &mut QueueVisitor<'_>) {
    let mut state = std::mem::take(&mut visitor.worker_rate_limit);
    finish_module_classes(visitor, &state);
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

fn resolve_pending(
    visitor: &QueueVisitor<'_>,
    state: &State,
    receiver: PendingReceiver,
) -> Option<WorkerRateLimitTarget> {
    match receiver {
        PendingReceiver::WorkerSpan(span) => Some(WorkerRateLimitTarget::LocalWorker(
            state.worker_call_indices.get(&span).copied(),
        )),
        PendingReceiver::ConstructedLocalClass(sym) => resolve_local_class(visitor, state, sym),
        PendingReceiver::ConstructedImported(local) => {
            Some(WorkerRateLimitTarget::ConstructedImported { local })
        }
        PendingReceiver::ConstructedNamespace { source, exported } => {
            Some(WorkerRateLimitTarget::ConstructedNamespace { source, exported })
        }
        PendingReceiver::Imported(local) => Some(WorkerRateLimitTarget::Imported { local }),
        PendingReceiver::Namespace { source, exported } => {
            Some(WorkerRateLimitTarget::Namespace { source, exported })
        }
        PendingReceiver::CallImported(local) => Some(WorkerRateLimitTarget::CallImported { local }),
        PendingReceiver::CallNamespace { source, exported } => {
            Some(WorkerRateLimitTarget::CallNamespace { source, exported })
        }
        PendingReceiver::Local(symbol) => {
            let mut visited = HashSet::new();
            resolve_local_symbol(visitor, state, symbol, &mut visited)
        }
    }
}

fn resolve_local_symbol(
    visitor: &QueueVisitor<'_>,
    state: &State,
    mut current: SymbolId,
    visited: &mut HashSet<SymbolId>,
) -> Option<WorkerRateLimitTarget> {
    while visited.insert(current) {
        if let Some(origins) = state.mutated_origins.get(&current) {
            return resolve_mutated(visitor, state, origins, visited);
        }
        let init = state.local_bindings.get(&current)?;
        match init {
            LocalInit::Alias(next) => {
                if is_class_or_function(visitor, *next) {
                    return None;
                }
                current = *next;
            }
            other => return resolve_local_init(visitor, state, other, visited),
        }
    }
    None
}

fn resolve_local_init(
    visitor: &QueueVisitor<'_>,
    state: &State,
    init: &LocalInit,
    visited: &mut HashSet<SymbolId>,
) -> Option<WorkerRateLimitTarget> {
    match init {
        LocalInit::WorkerSpan(span) => Some(WorkerRateLimitTarget::LocalWorker(
            state.worker_call_indices.get(span).copied(),
        )),
        LocalInit::ConstructedLocalClass(sym) => resolve_local_class(visitor, state, *sym),
        LocalInit::ConstructedImported(local) => Some(WorkerRateLimitTarget::ConstructedImported {
            local: local.clone(),
        }),
        LocalInit::ConstructedNamespace { source, exported } => {
            Some(WorkerRateLimitTarget::ConstructedNamespace {
                source: source.clone(),
                exported: exported.clone(),
            })
        }
        LocalInit::Imported(local) => Some(WorkerRateLimitTarget::Imported {
            local: local.clone(),
        }),
        LocalInit::Namespace { source, exported } => Some(WorkerRateLimitTarget::Namespace {
            source: source.clone(),
            exported: exported.clone(),
        }),
        LocalInit::CallImported(local) => Some(WorkerRateLimitTarget::CallImported {
            local: local.clone(),
        }),
        LocalInit::CallNamespace { source, exported } => {
            Some(WorkerRateLimitTarget::CallNamespace {
                source: source.clone(),
                exported: exported.clone(),
            })
        }
        LocalInit::Alias(next) => resolve_local_symbol(visitor, state, *next, visited),
        LocalInit::NonWorker => None,
    }
}

fn resolve_mutated(
    visitor: &QueueVisitor<'_>,
    state: &State,
    origins: &[LocalInit],
    visited: &mut HashSet<SymbolId>,
) -> Option<WorkerRateLimitTarget> {
    let mut deferred = None;
    for init in origins {
        let Some(target) = resolve_local_init(visitor, state, init, visited) else {
            continue;
        };
        match target {
            WorkerRateLimitTarget::LocalWorker(_) | WorkerRateLimitTarget::Unresolved => {
                return Some(WorkerRateLimitTarget::Unresolved);
            }
            WorkerRateLimitTarget::Imported { local }
            | WorkerRateLimitTarget::ConstructedImported { local }
            | WorkerRateLimitTarget::CallImported { local } => {
                deferred = Some(WorkerRateLimitTarget::CallImported { local });
            }
            WorkerRateLimitTarget::Namespace { source, exported }
            | WorkerRateLimitTarget::ConstructedNamespace { source, exported }
            | WorkerRateLimitTarget::CallNamespace { source, exported } => {
                deferred = Some(WorkerRateLimitTarget::CallNamespace { source, exported });
            }
            _ => {}
        }
    }
    deferred
}
