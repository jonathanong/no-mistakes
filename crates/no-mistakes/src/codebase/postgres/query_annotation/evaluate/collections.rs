use super::{concat, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn collection_call(
        &mut self,
        receiver: Option<&Value>,
        name: &str,
        args: &[Value],
        path: &Path,
        context: (u8, bool),
    ) -> Option<Value> {
        if !matches!(name, "map" | "join" | "push" | "forEach") {
            return None;
        }
        let (depth, generic) = context;
        if self.untrusted_array_intrinsics {
            self.invalidate_live_builders();
            self.opaque_callbacks(args, depth);
            return Some(Value::Failure("untrusted-array-intrinsics"));
        }
        let Some(Value::Array(id)) = receiver else {
            self.invalidate_live_builders();
            self.opaque_callbacks(args, depth);
            return Some(match receiver {
                Some(Value::Failure(reason)) => Value::Failure(reason),
                _ => Value::Failure("unbounded-collection-shape"),
            });
        };
        if self.conditional_collection_shape(*id) {
            self.invalidate_live_builders();
            self.opaque_callbacks(args, depth);
            return Some(Value::Failure("conditional-collection-shape"));
        }
        if self.invalidated_builders.contains(id)
            || self
                .deleted_argument_slots
                .iter()
                .any(|(other, _)| other == id)
            || self.argument_extra_slots.contains_key(id)
        {
            return Some(Value::Failure("opaque-collection-effect"));
        }
        let values = self.argument_objects[id].clone();
        let result = match name {
            "push" => {
                self.append_dense_slots(*id, args);
                Value::Number(self.argument_objects[id].len() as i64)
            }
            "join" => {
                let separator = match args {
                    [] => ",".to_string(),
                    [Value::Prefix(text, true, None)] => text.clone(),
                    _ => return Some(Value::Failure("unknown-collection-separator")),
                };
                let mut result = Value::Prefix(String::new(), true, None);
                for (index, value) in values.into_iter().enumerate() {
                    if index > 0 {
                        result = concat(result, Value::Prefix(separator.clone(), true, None));
                    }
                    if let Value::Failure(reason) = value {
                        return Some(Value::Failure(reason));
                    }
                    if !string_element(&value) {
                        return Some(Value::Failure("non-string-collection-element"));
                    }
                    result = concat(result, value);
                }
                result
            }
            _ => {
                let [callback @ Value::Function(function, _, _)] = args else {
                    self.invalidate_live_builders();
                    self.opaque_callbacks(args, depth);
                    return Some(Value::Failure("unknown-collection-callback"));
                };
                if !function.supported || function.asynchronous {
                    self.invalidate_live_builders();
                    self.opaque_callbacks(args, depth);
                    return Some(Value::Failure("unsupported-collection-callback"));
                }
                let mapped = self.next_builder;
                self.next_builder += 1;
                self.create_argument_object(mapped, Vec::new());
                for index in 0..values.len() {
                    let value = self.index(Value::Array(*id), index);
                    // Invoke through the ordinary callback/effect machinery, never
                    // inspect a callback's SQL text without executing its summary.
                    let scope = self.environment(crate::fx::FxHashMap::from_iter([
                        ("callback".into(), callback.clone()),
                        ("element".into(), value),
                        ("index".into(), Value::Number(index as i64)),
                        ("array".into(), Value::Array(*id)),
                        // Keep collected aliases rooted across callback branch joins.
                        ("mapped".into(), Value::Array(mapped)),
                    ]));
                    let call = Expr::Call {
                        callee: Box::new(Expr::Name("callback".into())),
                        args: vec![
                            Expr::Name("element".into()),
                            Expr::Name("index".into()),
                            Expr::Name("array".into()),
                        ],
                        start: 0,
                        end: 0,
                    };
                    let value = self.expr(&call, path, &scope, depth, generic);
                    if !matches!(value, Value::Evaluated(_, true)) {
                        self.invalidate_live_builders();
                        return Some(Value::Failure("opaque-collection-callback-effect"));
                    }
                    self.append_dense_slots(mapped, &[value.exposed()]);
                }
                if name == "map" {
                    Value::Array(mapped)
                } else {
                    Value::Primitive
                }
            }
        };
        Some(Value::Evaluated(Box::new(result), true))
    }

    pub(super) fn conditional_collection_shape(&self, id: u64) -> bool {
        self.argument_objects[&id] == [Value::Failure("conditional-collection-shape")]
    }
}

pub(super) fn string_element(value: &Value) -> bool {
    match value {
        Value::Prefix(_, _, None) | Value::Number(_) | Value::Boolean(_) => true,
        Value::Evaluated(value, true) => string_element(value),
        Value::Joined(values) => values.iter().all(string_element),
        _ => false,
    }
}
