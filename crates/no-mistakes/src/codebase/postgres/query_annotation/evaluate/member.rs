use super::{Evaluator, Value};
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn member_expr(
        &mut self,
        object: &Expr,
        name: &str,
        path: &Path,
        env: &super::Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        if let Some(value) = self.namespace_member(object, name, path, *env, depth, generic) {
            value
        } else {
            let value = self.expr(object, path, env, depth, generic).exposed();
            self.member(value, name, path)
        }
    }

    pub(super) fn static_namespace_read(
        &self,
        object: &Expr,
        path: &Path,
        env: super::Environment,
    ) -> bool {
        if let Expr::Sequence(parts) = object {
            return parts
                .last()
                .is_some_and(|part| self.static_namespace_read(part, path, env));
        }
        let Expr::Name(local) = object else {
            return false;
        };
        let file = &self.files[path];
        !self.scopes[env].contains_key(local)
            && self.mapped_parameter_value(env, local).is_none()
            && file.module.imports.get(local).is_some_and(|index| {
                file.module.ts.imported_bindings[*index].kind == ImportedBindingKind::Namespace
            })
    }

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
        let target = file
            .module
            .target(&import.specifier, path, self.resolver())?;
        Some(
            self.lookup_export(target, name, depth, generic)
                .unwrap_or(Value::Unknown),
        )
    }

    pub(super) fn member(&self, object: Value, name: &str, path: &Path) -> Value {
        match object {
            // The intrinsic length is a primitive; passing it cannot expose
            // an argument slot or a callback captured by the container.
            Value::Array(id)
                if name == "length"
                    && self.files[path].facts.precise_records
                    && (self.invalidated_builders.contains(&id)
                        || self
                            .argument_extra_slots
                            .get(&id)
                            .is_some_and(|slots| !slots.is_empty())) =>
            {
                Value::Failure("sparse-or-opaque-collection-shape")
            }
            Value::Arguments(id) | Value::Array(id) if name == "length" => {
                if self.conditional_collection_shape(id) {
                    Value::Failure("conditional-collection-shape")
                } else if self.invalidated_builders.contains(&id) {
                    Value::Unknown
                } else {
                    if self.files[path].facts.precise_records {
                        Value::Number(self.argument_objects[&id].len() as i64)
                    } else {
                        Value::Primitive
                    }
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
            Value::Failure(reason) => Value::Failure(reason),
            Value::Unknown if self.files[path].facts.precise_records => {
                Value::Failure("opaque-member-or-getter")
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
