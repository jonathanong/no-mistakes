mod alternatives;
mod atomic;
mod callbacks;
mod calls;
mod collections;
mod composition;
mod concat;
mod control_flow;
mod member;
use {concat::concat, scope::Scope};
mod delete;
mod effects;
mod frame_arena;
mod index;
mod mapped;
mod modules;
mod opaque;
mod reclamation;
mod result;
mod run;
mod scope;
mod slot_write;
mod slots;
mod statements;
mod tagged;
mod temporaries;
mod transactions;
mod value;
use super::modules::Module;
use super::{Expr, Function, QueryAnnotationFileFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, sync::Arc};
pub(super) use value::Value;

pub(super) type Environment = usize;
pub(super) type CallSpan = (u32, u32);
pub(super) type ExecutionEvents = BTreeMap<(PathBuf, CallSpan), Vec<(bool, Value)>>;
pub(super) struct File<'a> {
    pub facts: &'a QueryAnnotationFileFacts,
    pub module: Arc<Module>,
    pub executors: FxHashSet<CallSpan>,
}
pub(super) struct Evaluator<'a, F> {
    pub files: &'a FxHashMap<PathBuf, File<'a>>,
    pub resolve: F,
    // Immutable request-wide trust; branch snapshots cannot restore intrinsics.
    pub untrusted_array_intrinsics: bool,
    pub events: ExecutionEvents,
    pub scopes: frame_arena::ScopeArena,
    pub transactions: transactions::Transactions,
    pub modules: FxHashMap<PathBuf, Environment>,
    pub active_module_initials: Vec<alternatives::modules::Initials>,
    pub transient_roots: Vec<temporaries::TransientRoot>,
    pub active_callback_functions: Option<FxHashSet<callbacks::CallbackIdentity>>,
    pub active_callback_executions: Option<FxHashSet<callbacks::CallbackIdentity>>,
    pub next_builder: u64,
    pub invalidated_builders: FxHashSet<u64>,
    pub builder_updates: FxHashMap<u64, Value>,
    pub captured_bindings: FxHashMap<Environment, FxHashMap<String, Environment>>,
    pub captured_binding_readers: FxHashMap<Environment, FxHashMap<String, FxHashSet<Environment>>>,
    pub mapped_argument_owners: FxHashMap<u64, Environment>,
    pub mapped_parameter_indices: FxHashMap<u64, FxHashMap<String, usize>>,
    pub deleted_argument_slots: FxHashSet<(u64, Option<usize>)>,
    pub argument_objects: FxHashMap<u64, Vec<Value>>,
    pub argument_extra_slots: FxHashMap<u64, BTreeMap<usize, Value>>,
    pub definite_deleted_argument_slots: FxHashSet<(u64, usize)>,
    pub recreated_argument_slots: FxHashSet<(u64, usize)>,
    pub disconnected_argument_slots: FxHashSet<(u64, usize)>,
    pub fresh_mapped_parameters: FxHashMap<Environment, FxHashSet<String>>,
    pub fresh_mapped_argument_bindings: FxHashMap<u64, FxHashMap<Environment, FxHashSet<String>>>,
    pub mapped_arguments: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
}

mod dispatch;

#[cfg(test)]
mod reclamation_test_setup;
