use crate::fx::FxHashMap;
use oxc_syntax::symbol::SymbolId;
use std::collections::HashMap;

#[derive(Default)]
pub(in crate::queue) struct State {
    pub(super) worker_call_indices: HashMap<u32, usize>,
    pub(super) local_bindings: FxHashMap<SymbolId, LocalInit>,
    pub(super) mutated_origins: FxHashMap<SymbolId, Vec<LocalInit>>,
    pub(super) local_classes: FxHashMap<SymbolId, LocalClassInit>,
    pub(super) module_bindings: HashMap<String, ModuleInit>,
    pub(super) module_classes: HashMap<String, ModuleClassInit>,
    pub(super) pending_calls: Vec<PendingCall>,
}

#[derive(Clone)]
pub(in crate::queue::extract_worker_rate_limit) enum LocalInit {
    WorkerSpan(u32),
    ConstructedLocalClass(SymbolId),
    ConstructedImported(String),
    ConstructedNamespace { source: String, exported: String },
    NonWorker,
    Alias(SymbolId),
    Imported(String),
    Namespace { source: String, exported: String },
    CallImported(String),
    CallNamespace { source: String, exported: String },
}

#[derive(Clone)]
pub(in crate::queue::extract_worker_rate_limit) enum LocalClassInit {
    WorkerClass,
    NonWorker,
    ExtendsLocal(SymbolId),
    ExtendsImported(String),
    ExtendsNamespace { source: String, exported: String },
    Dynamic,
}

#[derive(Clone)]
pub(in crate::queue::extract_worker_rate_limit) enum ModuleInit {
    WorkerSpan(u32),
    ConstructedLocalClass(String),
    ConstructedImported(String),
    ConstructedNamespace { source: String, exported: String },
    NonWorker,
    Alias(String),
    Imported(String),
    Namespace { source: String, exported: String },
}

#[derive(Clone)]
pub(in crate::queue::extract_worker_rate_limit) enum ModuleClassInit {
    WorkerClass,
    NonWorker,
    ExtendsLocal(String),
    ExtendsImported(String),
    ExtendsNamespace { source: String, exported: String },
    Dynamic,
}

pub(in crate::queue::extract_worker_rate_limit) struct PendingCall {
    pub(super) line: usize,
    pub(super) receiver: PendingReceiver,
}

pub(in crate::queue::extract_worker_rate_limit) enum PendingReceiver {
    WorkerSpan(u32),
    ConstructedLocalClass(SymbolId),
    ConstructedImported(String),
    ConstructedNamespace { source: String, exported: String },
    Local(SymbolId),
    Imported(String),
    Namespace { source: String, exported: String },
    CallImported(String),
    CallNamespace { source: String, exported: String },
}

impl State {
    pub(in crate::queue) fn note_worker_span(&mut self, span_start: u32, worker_index: usize) {
        self.worker_call_indices.insert(span_start, worker_index);
    }
}
