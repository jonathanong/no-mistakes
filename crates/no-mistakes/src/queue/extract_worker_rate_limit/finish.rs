use super::classes::resolve_local_class;
use super::finish_exports::{finish_module_classes, finish_module_exports};
use super::predicates::is_class_or_function;
use super::{LocalInit, PendingReceiver, State};
use crate::queue::extract_model::{WorkerRateLimitCall, WorkerRateLimitTarget};
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_syntax::symbol::SymbolId;
use std::collections::HashSet;

pub(super) fn finish(visitor: &mut QueueVisitor<'_>) {
    let mut state = std::mem::take(&mut visitor.worker_rate_limit);
    finish_module_classes(visitor, &state);
    finish_module_exports(visitor, &state);
    for call in std::mem::take(&mut state.pending_calls) {
        if let Some((target, mutated)) = resolve_pending(visitor, &state, call.receiver) {
            if let WorkerRateLimitTarget::LocalWorker(Some(worker)) = &target {
                visitor.note_shared_rate_limit_receiver(call.start, *worker);
            }
            visitor.facts.worker_rate_limits.push(WorkerRateLimitCall {
                line: line_number(visitor.source, call.start),
                target,
                mutated,
            });
        }
    }
}

fn resolve_pending(
    visitor: &QueueVisitor<'_>,
    state: &State,
    receiver: PendingReceiver,
) -> Option<(WorkerRateLimitTarget, bool)> {
    if let PendingReceiver::Local(symbol) = receiver {
        let mut visited = HashSet::new();
        return resolve_local_symbol(visitor, state, symbol, &mut visited);
    }
    let target = match receiver {
        PendingReceiver::WorkerSpan(span) => {
            WorkerRateLimitTarget::LocalWorker(state.worker_call_indices.get(&span).copied())
        }
        PendingReceiver::ConstructedLocalClass(sym) => resolve_local_class(visitor, state, sym)?,
        PendingReceiver::ConstructedLocalBinding(sym) => resolve_local_ctor(visitor, state, sym)?,
        PendingReceiver::ConstructedImported(local) => {
            WorkerRateLimitTarget::ConstructedImported { local }
        }
        PendingReceiver::ConstructedNamespace { source, exported } => {
            WorkerRateLimitTarget::ConstructedNamespace { source, exported }
        }
        PendingReceiver::Imported(local) => WorkerRateLimitTarget::Imported { local },
        PendingReceiver::Namespace { source, exported } => {
            WorkerRateLimitTarget::Namespace { source, exported }
        }
        PendingReceiver::CallImported(local) => WorkerRateLimitTarget::CallImported { local },
        PendingReceiver::CallNamespace { source, exported } => {
            WorkerRateLimitTarget::CallNamespace { source, exported }
        }
        PendingReceiver::Local(_) => return None,
    };
    Some((target, false))
}

fn resolve_local_ctor(
    visitor: &QueueVisitor<'_>,
    state: &State,
    mut current: SymbolId,
) -> Option<WorkerRateLimitTarget> {
    let mut visited = HashSet::new();
    while visited.insert(current) {
        if state.local_classes.contains_key(&current) {
            return resolve_local_class(visitor, state, current);
        }
        match state.local_bindings.get(&current)? {
            LocalInit::WorkerClass => return Some(WorkerRateLimitTarget::LocalWorker(None)),
            LocalInit::Alias(next) => current = *next,
            LocalInit::Imported(local) => {
                return Some(WorkerRateLimitTarget::ConstructedImported {
                    local: local.clone(),
                });
            }
            LocalInit::Namespace { source, exported } => {
                return Some(WorkerRateLimitTarget::ConstructedNamespace {
                    source: source.clone(),
                    exported: exported.clone(),
                });
            }
            _ => return None,
        }
    }
    None
}

fn resolve_local_symbol(
    visitor: &QueueVisitor<'_>,
    state: &State,
    mut current: SymbolId,
    visited: &mut HashSet<SymbolId>,
) -> Option<(WorkerRateLimitTarget, bool)> {
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
) -> Option<(WorkerRateLimitTarget, bool)> {
    if let LocalInit::Alias(next) = init {
        return resolve_local_symbol(visitor, state, *next, visited);
    }
    let target = match init {
        LocalInit::WorkerSpan(span) => {
            WorkerRateLimitTarget::LocalWorker(state.worker_call_indices.get(span).copied())
        }
        LocalInit::ConstructedLocalClass(sym) => resolve_local_class(visitor, state, *sym)?,
        LocalInit::ConstructedLocalBinding(sym) => resolve_local_ctor(visitor, state, *sym)?,
        LocalInit::ConstructedImported(local) => WorkerRateLimitTarget::ConstructedImported {
            local: local.clone(),
        },
        LocalInit::ConstructedNamespace { source, exported } => {
            WorkerRateLimitTarget::ConstructedNamespace {
                source: source.clone(),
                exported: exported.clone(),
            }
        }
        LocalInit::Imported(local) => WorkerRateLimitTarget::Imported {
            local: local.clone(),
        },
        LocalInit::Namespace { source, exported } => WorkerRateLimitTarget::Namespace {
            source: source.clone(),
            exported: exported.clone(),
        },
        LocalInit::CallImported(local) => WorkerRateLimitTarget::CallImported {
            local: local.clone(),
        },
        LocalInit::CallNamespace { source, exported } => WorkerRateLimitTarget::CallNamespace {
            source: source.clone(),
            exported: exported.clone(),
        },
        LocalInit::Alias(_) | LocalInit::NonWorker | LocalInit::WorkerClass => return None,
    };
    Some((target, false))
}

fn resolve_mutated(
    visitor: &QueueVisitor<'_>,
    state: &State,
    origins: &[LocalInit],
    visited: &mut HashSet<SymbolId>,
) -> Option<(WorkerRateLimitTarget, bool)> {
    let mut deferred = None;
    for init in origins {
        let Some((target, _)) = resolve_local_init(visitor, state, init, visited) else {
            continue;
        };
        match target {
            WorkerRateLimitTarget::LocalWorker(_) | WorkerRateLimitTarget::Unresolved => {
                return Some((WorkerRateLimitTarget::Unresolved, true));
            }
            target => deferred = Some((target, true)),
        }
    }
    deferred
}
