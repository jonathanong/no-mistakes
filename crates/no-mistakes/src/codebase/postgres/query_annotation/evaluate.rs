mod calls;
mod statements;
use super::{Expr, Function, QueryAnnotationFileFacts};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(super) enum Value {
    Prefix(String, bool),
    Function(Function, PathBuf, Environment),
    Unknown,
    Unsupported,
}
pub(super) type Environment = usize;
pub(super) struct File<'a> {
    pub facts: &'a QueryAnnotationFileFacts,
    pub ts: &'a TsFileFacts,
    pub executors: Vec<u32>,
}
pub(super) struct Evaluator<'a, F> {
    pub files: BTreeMap<PathBuf, File<'a>>,
    pub resolve: F,
    pub events: BTreeMap<(PathBuf, u32), Vec<(bool, Value)>>,
    pub scopes: Vec<BTreeMap<String, Value>>,
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn environment(
        &mut self,
        values: BTreeMap<String, Value>,
    ) -> Environment {
        let id = self.scopes.len();
        self.scopes.push(values);
        id
    }
    pub fn run(&mut self) {
        let roots = self
            .files
            .iter()
            .map(|(path, file)| {
                (
                    path.clone(),
                    file.facts.roots.clone(),
                    file.facts.globals.clone(),
                )
            })
            .collect::<Vec<_>>();
        for (path, roots, globals) in roots {
            let mut env = self.environment(BTreeMap::new());
            self.steps(&roots, &path, &mut env, 16, true);
            // Function declarations describe possible entrypoints; contextual
            // callback invocations below take precedence over this unknown input.
            for expr in globals.values() {
                if let Expr::Function(function) = expr {
                    let mut values: BTreeMap<String, Value> = function
                        .params
                        .iter()
                        .map(|name| (name.clone(), Value::Unknown))
                        .collect();
                    if let Some(name) = &function.self_name {
                        values.insert(name.clone(), Value::Unknown);
                    }
                    let mut env = self.environment(values);
                    self.steps(&function.body, &path, &mut env, 16, true);
                }
            }
        }
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
        match expr {
            Expr::Unknown => Value::Unknown,
            Expr::Unsupported => Value::Unsupported,
            Expr::Text(value) => Value::Prefix(value.clone(), true),
            Expr::Name(name) => self.scopes[*env]
                .get(name)
                .cloned()
                .unwrap_or_else(|| self.name(path, name, depth, generic)),
            Expr::Function(function) => {
                // Request-owned scope IDs retain live bindings without recursive
                // closure copies or reference cycles between sibling functions.
                Value::Function(function.clone(), path.to_path_buf(), *env)
            }
            Expr::Template(parts) => {
                let mut prefix = Value::Prefix(String::new(), true);
                for part in parts {
                    prefix = concat(prefix, self.expr(part, path, env, depth, generic));
                }
                prefix
            }
            Expr::Append(base, tail) => {
                let base = self.expr(base, path, env, depth, generic);
                let tail = self.expr(tail, path, env, depth, generic);
                concat(base, tail)
            }
            Expr::Tagged(tag, parts) => {
                let file = &self.files[path];
                let name = if tag == "String.raw" { "String" } else { tag };
                let trusted = if tag == "String.raw" {
                    !file
                        .ts
                        .imported_bindings
                        .iter()
                        .any(|binding| binding.local == name)
                } else {
                    file.facts.trusted_tags.contains(tag)
                };
                if !self.scopes[*env].contains_key(name)
                    && !file.facts.globals.contains_key(name)
                    && trusted
                {
                    let mut prefix = Value::Prefix(String::new(), true);
                    for part in parts {
                        prefix = concat(prefix, self.expr(part, path, env, depth, generic));
                    }
                    prefix
                } else {
                    Value::Unknown
                }
            }
            Expr::Call {
                callee,
                args,
                start,
            } => self.call(callee, args, *start, path, env, (depth, generic)),
            Expr::Children(children) => {
                for child in children {
                    self.expr(child, path, env, depth, generic);
                }
                Value::Unknown
            }
        }
    }
}

pub(super) fn concat(base: Value, tail: Value) -> Value {
    match (base, tail) {
        (Value::Unsupported, _) => Value::Unsupported,
        (Value::Prefix(base, true), Value::Unsupported) if base.trim().is_empty() => {
            Value::Unsupported
        }
        (Value::Prefix(base, false), _) => Value::Prefix(base, false),
        (Value::Prefix(mut base, true), Value::Prefix(tail, complete)) => {
            base.push_str(&tail);
            Value::Prefix(base, complete)
        }
        (Value::Prefix(base, true), _) => Value::Prefix(base, false),
        _ => Value::Unknown,
    }
}
