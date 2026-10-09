mod calls;
mod statements;
use super::{Expr, Function, QueryAnnotationFileFacts};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub(super) enum Value {
    Prefix(String, bool),
    Function(Function, PathBuf, Arc<Environment>),
    Unknown,
    Unsupported,
}
pub(super) type Environment = BTreeMap<String, Value>;
pub(super) struct File<'a> {
    pub facts: &'a QueryAnnotationFileFacts,
    pub ts: &'a TsFileFacts,
    pub executors: Vec<(u32, String)>,
}
pub(super) struct Evaluator<'a, F> {
    pub files: BTreeMap<PathBuf, File<'a>>,
    pub resolve: F,
    pub events: BTreeMap<(PathBuf, u32, String), Vec<(bool, Value)>>,
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
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
            self.steps(&roots, &path, &mut Environment::new(), 16, true);
            // Function declarations describe possible entrypoints; contextual
            // callback invocations below take precedence over this unknown input.
            for expr in globals.values() {
                if let Expr::Function(function) = expr {
                    if !function.supported {
                        continue;
                    }
                    let mut env: Environment = function
                        .params
                        .iter()
                        .map(|name| (name.clone(), Value::Unknown))
                        .collect();
                    if let Some(name) = &function.self_name {
                        env.insert(name.clone(), Value::Unknown);
                    }
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
            Expr::Name(name) => env
                .get(name)
                .cloned()
                .unwrap_or_else(|| self.name(path, name, depth, generic)),
            Expr::Function(function) => {
                // Closures retain immutable snapshots; recursively copying
                // previously captured closures makes sibling helpers exponential.
                Value::Function(function.clone(), path.to_path_buf(), Arc::new(env.clone()))
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
            Expr::Tagged(tag, text) => {
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
                if !env.contains_key(name) && !file.facts.globals.contains_key(name) && trusted {
                    Value::Prefix(text.clone(), true)
                } else {
                    Value::Unknown
                }
            }
            Expr::Call {
                callee,
                args,
                line,
                spelling,
            } => self.call(callee, args, (*line, spelling), path, env, (depth, generic)),
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
