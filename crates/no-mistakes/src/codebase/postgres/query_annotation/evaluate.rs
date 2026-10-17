mod alternatives;
mod atomic;
mod callbacks;
mod capture;
use capture::Capture;
mod calls;
mod collections;
mod composition;
mod concat;
mod control_flow;
mod member;
use {concat::concat, scope::Scope};
mod delete;
mod effects;
mod index;
mod initial_template;
mod mapped;
mod modules;
mod new;
pub(super) use new::new;
mod opaque;
mod result;
mod run;
mod scope;
mod slot_write;
mod slots;
mod statements;
mod tagged;
mod temporaries;
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
    pub initial_template: std::sync::OnceLock<Option<Scope>>,
}
pub(super) struct Evaluator<'a, F> {
    // Own one request observer; activation counters borrow it without TLS/Arc clones.
    pub initial_template_observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    pub files: &'a FxHashMap<PathBuf, File<'a>>,
    pub resolve: F,
    // Immutable request-wide trust; branch snapshots cannot restore intrinsics.
    pub untrusted_array_intrinsics: bool,
    pub events: ExecutionEvents,
    pub scopes: Vec<Scope>,
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

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn environment(
        &mut self,
        values: FxHashMap<String, Value>,
    ) -> Environment {
        let id = self.scopes.len();
        self.scopes.push(values.into());
        id
    }
    pub(super) fn expr(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        let Some(depth) = depth.checked_sub(1) else {
            return Value::Failure("evaluation-depth-limit");
        };
        let value = if let Some(value) = self.atomic(expr, path, env, (depth, generic)) {
            value
        } else {
            match expr {
                Expr::Binary(op, left, right) => {
                    self.binary(op, left, right, path, env, (depth, generic))
                }
                Expr::Flow(steps) => self.steps(steps, path, &mut env.clone(), depth, generic),
                Expr::Template(parts) => self.template(parts, path, env, (depth, generic)),
                Expr::Append(base, tail) => self.append(base, tail, path, env, (depth, generic)),
                Expr::Tagged(tag, parts, effects) => {
                    self.tagged(tag, parts, effects, path, env, (depth, generic))
                }
                Expr::Sequence(parts) => self.sequence(parts, path, env, (depth, generic)),
                Expr::Discard(expr) => self.discard(expr, path, env, (depth, generic)),
                Expr::Alternatives(arms) => self.alternatives(arms, path, env, depth, generic),
                Expr::Spread(expr) => {
                    Value::Aggregate(vec![self.expr(expr, path, env, depth, generic)].into())
                }
                Expr::OpaqueCallback(expr) => {
                    match self.expr(expr, path, env, depth, generic).exposed() {
                        Value::Function(mut function, path, captured) => {
                            Arc::make_mut(&mut function).supported = false;
                            Value::Function(function, path, captured)
                        }
                        _ => Value::Unknown,
                    }
                }
                Expr::Await(expr) => self.await_expr(expr, path, env, depth, generic),
                Expr::Call {
                    callee,
                    args,
                    start,
                    end,
                } => self.call(callee, args, (*start, *end), path, env, (depth, generic)),
                Expr::ModuleLoad(specifier) => self.load_dynamic_import(specifier, path),
                Expr::Delete(children, index) => {
                    self.deleted(children, *index, path, env, (depth, generic))
                }
                Expr::Opaque(children) => self.opaque(children, &[], path, env, (depth, generic)),
                Expr::OptionalCall { call, children } => {
                    self.optional_call(call, children, path, env, (depth, generic))
                }
                Expr::OpaqueWrite { children, targets } => {
                    self.opaque(children, targets, path, env, (depth, generic))
                }
                Expr::SlotWrite {
                    receiver,
                    index,
                    value,
                } => self.slot_write(receiver, *index, value, path, env, (depth, generic)),
                Expr::Member(object, name) => {
                    self.member_expr(object, name, path, env, (depth, generic))
                }
                Expr::ComputedMember(object, key) => {
                    let value = self.operand(object, path, env, (depth, generic)).exposed();
                    self.expr(key, path, env, depth, generic);
                    self.member(value, "", path)
                }
                Expr::Index(object, index) => {
                    let value = self.expr(object, path, env, depth, generic).exposed();
                    self.index(value, *index)
                }
                Expr::Children(children)
                | Expr::Array(children)
                | Expr::Container(children)
                | Expr::Object(children)
                | Expr::Record(children, _) => {
                    self.container(expr, children, path, env, (depth, generic))
                }
                Expr::Unknown
                | Expr::Primitive
                | Expr::Number(_)
                | Expr::Boolean(_)
                | Expr::Unsupported
                | Expr::Text(_)
                | Expr::Name(_)
                | Expr::Function(_) => unreachable!("atomic expression"),
            }
        };
        if matches!(&value, Value::Prefix(_, _, Some(id)) if self.invalidated_builders.contains(id))
        {
            if self.files[path].facts.precise_records {
                Value::Failure("opaque-builder-effect")
            } else {
                Value::Unknown
            }
        } else {
            value
        }
    }
}
