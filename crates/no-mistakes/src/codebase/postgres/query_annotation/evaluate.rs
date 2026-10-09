mod alternatives;
mod callbacks;
mod calls;
mod concat;
mod member;
use concat::concat;
mod delete;
mod effects;
mod index;
mod mapped;
mod modules;
mod result;
mod run;
mod statements;
mod tagged;
use super::{Expr, Function, QueryAnnotationFileFacts};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Eq, PartialEq)]
pub(super) enum Value {
    Prefix(String, bool, Option<u64>),
    Promise(Box<Value>),
    Evaluated(Box<Value>, bool),
    Aggregate(Vec<Value>),
    Arguments(u64),
    Function(Function, PathBuf, Environment),
    Unknown,
    Unsupported,
    SlotDeletion,
}
impl Value {
    pub(super) fn exposed(self) -> Self {
        let mut value = self;
        while let Self::Evaluated(inner, _) = value {
            value = *inner;
        }
        value
    }
}
pub(super) type Environment = usize;
pub(super) struct File<'a> {
    pub facts: &'a QueryAnnotationFileFacts,
    pub ts: &'a TsFileFacts,
    pub executors: FxHashSet<u32>,
    pub imports: FxHashMap<String, usize>,
    pub exports: FxHashMap<String, usize>,
}
pub(super) struct Evaluator<'a, F> {
    pub files: &'a FxHashMap<PathBuf, File<'a>>,
    pub resolve: F,
    pub events: BTreeMap<(PathBuf, u32), Vec<(bool, Value)>>,
    pub scopes: Vec<FxHashMap<String, Value>>,
    pub modules: FxHashMap<PathBuf, Environment>,
    pub next_builder: u64,
    pub invalidated_builders: FxHashSet<u64>,
    pub builder_updates: FxHashMap<u64, Value>,
    pub captured_bindings: FxHashMap<Environment, FxHashMap<String, Environment>>,
    pub deleted_argument_slots: FxHashSet<(u64, Option<usize>)>,
    pub argument_objects: FxHashMap<u64, Vec<Value>>,
    pub definite_deleted_argument_slots: FxHashSet<(u64, usize)>,
    pub fresh_mapped_parameters: FxHashMap<Environment, FxHashSet<String>>,
    pub mapped_arguments: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn environment(
        &mut self,
        values: FxHashMap<String, Value>,
    ) -> Environment {
        let id = self.scopes.len();
        self.scopes.push(values);
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
            return Value::Unknown;
        };
        let value = match expr {
            Expr::Unknown | Expr::Primitive => Value::Unknown,
            Expr::Unsupported => Value::Unsupported,
            Expr::Text(value) => Value::Prefix(value.clone(), true, None),
            Expr::Name(name) => self.mapped_parameter_value(*env, name).unwrap_or_else(|| {
                self.scopes[*env]
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| self.name(path, name, depth, generic))
            }),
            Expr::Function(function) => {
                // Request-owned scope IDs retain live bindings without recursive
                // closure copies or reference cycles between sibling functions.
                Value::Function(function.clone(), path.to_path_buf(), *env)
            }
            Expr::Template(parts) => {
                let mut prefix = Value::Prefix(String::new(), true, None);
                for part in parts {
                    prefix = concat(prefix, self.expr(part, path, env, depth, generic));
                }
                prefix
            }
            Expr::Append(base, tail) => {
                let base = self.expr(base, path, env, depth, generic).exposed();
                let tail = self.expr(tail, path, env, depth, generic);
                let base = if matches!(&base, Value::Prefix(_, _, Some(id)) if self.invalidated_builders.contains(id))
                {
                    Value::Unknown
                } else {
                    base
                };
                let value = concat(base, tail);
                self.replace_builder(&value);
                value
            }
            Expr::Tagged(tag, parts, effects) => {
                self.tagged(tag, parts, effects, path, env, (depth, generic))
            }
            Expr::Sequence(parts) => self.sequence(parts, path, env, (depth, generic)),
            Expr::Discard(expr) => self.discard(expr, path, env, (depth, generic)),
            Expr::Alternatives(arms) => self.alternatives(arms, path, env, depth, generic),
            Expr::Spread(expr) => {
                Value::Aggregate(vec![self.expr(expr, path, env, depth, generic)])
            }
            Expr::OpaqueCallback(expr) => {
                match self.expr(expr, path, env, depth, generic).exposed() {
                    Value::Function(mut function, path, captured) => {
                        function.supported = false;
                        Value::Function(function, path, captured)
                    }
                    _ => Value::Unknown,
                }
            }
            Expr::Await(expr) => {
                let mut value = self.expr(expr, path, env, depth, generic).exposed();
                while let Value::Promise(inner) = value {
                    value = inner.exposed();
                }
                value
            }
            Expr::Call {
                callee,
                args,
                start,
            } => self.call(callee, args, *start, path, env, (depth, generic)),
            Expr::Delete(children, index) => {
                self.deleted(children, *index, path, env, (depth, generic))
            }
            Expr::Opaque(children) => {
                self.invalidate_opaque_mapped_names(children, *env);
                let values = children
                    .iter()
                    .map(|child| self.expr(child, path, env, depth, generic))
                    .collect::<Vec<_>>();
                self.invalidate_builders(&values);
                self.opaque_callbacks(&values, depth);
                Value::Unknown
            }
            Expr::Member(object, name) => {
                let value = self.expr(object, path, env, depth, generic).exposed();
                self.member(value, name)
            }
            Expr::Index(object, index) => {
                let value = self.expr(object, path, env, depth, generic).exposed();
                self.index(value, *index)
            }
            Expr::Children(children) => Value::Aggregate(
                children
                    .iter()
                    .map(|child| self.expr(child, path, env, depth, generic))
                    .collect(),
            ),
        };
        if matches!(&value, Value::Prefix(_, _, Some(id)) if self.invalidated_builders.contains(id))
        {
            Value::Unknown
        } else {
            value
        }
    }
}
