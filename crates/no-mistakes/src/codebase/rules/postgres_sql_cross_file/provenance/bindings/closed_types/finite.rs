use super::*;

impl Bindings<'_> {
    pub(crate) fn finite_string_parameter(&self, file: &Path, scope: usize, name: &str) -> bool {
        self.numeric_parameter_bounds_only
            && !self.non_required_parameter(file, scope, name)
            && self
                .scoped_parameters(file, scope, name)
                .iter()
                .any(|parameter| match &parameter.type_reference {
                    Some(shape @ ValueType::FiniteStringLiterals)
                    | Some(shape @ ValueType::Reference(_))
                    | Some(shape @ ValueType::Element(_)) => self.finite_string_type(file, shape),
                    _ => false,
                })
    }

    pub(super) fn non_required_parameter(&self, file: &Path, scope: usize, name: &str) -> bool {
        self.facts.get_ts_facts(file).is_some_and(|ts| {
            ts.value_flow
                .non_required_parameters
                .get(&scope)
                .is_some_and(|names| names.contains(name))
        })
    }

    pub(in super::super) fn parameter_at(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
    ) -> bool {
        self.parameter(file, scope, name)
            || self.widened_finite_string_flow(
                file,
                scope,
                name,
                at,
                &mut super::super::flow::Seen {
                    bindings: crate::fx::fx_set(),
                    calls: crate::fx::fx_set(),
                },
            )
    }

    pub(super) fn finite_string_type(&self, file: &Path, shape: &ValueType) -> bool {
        self.finite_string_type_inner(file, shape, &mut Default::default())
    }

    fn finite_string_type_inner(
        &self,
        file: &Path,
        shape: &ValueType,
        visited: &mut crate::fx::FxHashSet<(std::path::PathBuf, String)>,
    ) -> bool {
        match shape {
            ValueType::FiniteStringLiterals => true,
            ValueType::Element(name) => self.finite_tuple(file, name),
            ValueType::Reference(name) => {
                let key = (file.to_path_buf(), name.clone());
                if !visited.insert(key.clone()) {
                    return false;
                }
                let local = self
                    .facts
                    .get_ts_facts(file)
                    .and_then(|ts| ts.value_flow.type_shapes.get(name));
                let resolved =
                    local.is_some_and(|shape| self.finite_string_type_inner(file, shape, visited));
                visited.remove(&key);
                resolved
            }
            _ => false,
        }
    }

    fn finite_tuple(&self, file: &Path, name: &str) -> bool {
        use crate::codebase::dependencies::extract::{unique_closed_collection, RegistryKind};
        self.facts.get_ts_facts(file).is_some_and(|ts| {
            unique_closed_collection(&ts.value_flow, name, RegistryKind::Array)
                && ts
                    .value_flow
                    .closed_string_registries
                    .iter()
                    .any(|registry| {
                        registry.name == name && registry.scope == 0 && registry.const_tuple
                    })
        })
    }

    pub(in super::super) fn widened_finite_string_flow(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        seen: &mut super::super::flow::Seen,
    ) -> bool {
        if !self.widened_string_assertion_at(file, at) {
            return false;
        }
        self.finite_string_source(file, scope, name, at, seen)
    }

    fn widened_string_assertion_at(&self, file: &Path, at: u32) -> bool {
        self.facts
            .get_ts_facts(file)
            .and_then(|ts| ts.value_flow.widened_string_assertions.get(&at))
            .is_some_and(|shapes| {
                shapes
                    .iter()
                    .any(|shape| !self.finite_string_type(file, shape))
            })
    }

    fn finite_string_source(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        seen: &mut super::super::flow::Seen,
    ) -> bool {
        if self.finite_string_parameter(file, scope, name) {
            return true;
        }
        let Some(binding) = self.value(file, scope, name, at) else {
            return false;
        };
        if !binding.immutable {
            return false;
        }
        let key = (binding.scope, binding.name.clone(), binding.declared_at);
        if !seen.bindings.insert(key.clone()) {
            return false;
        }
        let result = match &binding.value {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.finite_string_source(file, scope.unwrap_or(0), name, *offset, seen),
            ValueExpression::Derived(parts) => parts
                .iter()
                .any(|part| self.finite_string_expression_source(file, part, seen)),
            ValueExpression::Awaited(inner) => {
                self.finite_string_expression_source(file, inner, seen)
            }
            _ => false,
        };
        seen.bindings.remove(&key);
        result
    }

    fn finite_string_expression_source(
        &self,
        file: &Path,
        expression: &ValueExpression,
        seen: &mut super::super::flow::Seen,
    ) -> bool {
        match expression {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.finite_string_source(file, scope.unwrap_or(0), name, *offset, seen),
            ValueExpression::Derived(parts) => parts
                .iter()
                .any(|part| self.finite_string_expression_source(file, part, seen)),
            ValueExpression::Awaited(inner) => {
                self.finite_string_expression_source(file, inner, seen)
            }
            _ => false,
        }
    }
}
