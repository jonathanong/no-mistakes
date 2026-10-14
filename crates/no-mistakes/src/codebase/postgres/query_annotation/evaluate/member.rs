use super::{Evaluator, Value};
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn namespace_member(
        &mut self,
        object: &Expr,
        name: &str,
        path: &Path,
        env: super::Environment,
        depth: u8,
        generic: bool,
    ) -> Option<Value> {
        let Expr::Name(local) = object else {
            return None;
        };
        if self.scopes[env].contains_key(local) || self.mapped_parameter_value(env, local).is_some()
        {
            return None;
        }
        let file = &self.files[path];
        let import = file
            .module
            .imports
            .get(local)
            .map(|index| &file.module.ts.imported_bindings[*index])?;
        if import.kind != ImportedBindingKind::Namespace {
            return None;
        }
        let target = file.module.target(&import.specifier, path, &self.resolve)?;
        Some(
            self.lookup_export(target, name, depth, generic)
                .unwrap_or(Value::Unknown),
        )
    }

    pub(super) fn member(&self, object: Value, name: &str, path: &Path) -> Value {
        match object {
            // The intrinsic length is a primitive; passing it cannot expose
            // an argument slot or a callback captured by the container.
            Value::Arguments(id) if name == "length" => {
                if self.invalidated_builders.contains(&id) {
                    Value::Unknown
                } else {
                    Value::Primitive
                }
            }
            Value::Record(values, keys)
                if self.files[path].facts.precise_records && !name.is_empty() =>
            {
                keys.iter()
                    .rposition(|key| key == name)
                    .and_then(|index| values.get(index).cloned())
                    .unwrap_or(Value::Unknown)
            }
            // Preserve conservative member effects for other object shapes.
            value => member_candidates(value),
        }
    }
}

pub(super) fn member_candidates(value: Value) -> Value {
    match value.exposed() {
        Value::Record(values, _) => Value::Aggregate(values),
        Value::Object(values) => Value::Aggregate(values),
        Value::Aggregate(values) | Value::Joined(values) | Value::Possible(values) => {
            Value::Aggregate(
                values
                    .into_iter()
                    .map(member_candidates)
                    .collect::<Vec<_>>()
                    .into(),
            )
        }
        value => Value::Aggregate(vec![value].into()),
    }
}
