use super::names::identifier_name;
use super::{plain_call, RegistryMutation};
use crate::codebase::dependencies::extract::ImportedBindingKind;
use oxc_ast::ast::{AssignmentTarget, CallExpression, Expression};

const MUTATING_METHODS: &[&str] = &[
    "set", "add", "push", "pop", "shift", "unshift", "splice", "delete", "clear", "fill",
];

impl super::super::super::ImportCollector {
    pub(in super::super) fn note_closed_registry_call(&mut self, call: &CallExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        if let Some(name) = mutating_receiver(call).or_else(|| assign_target(call)) {
            self.record_registry_mutation(&name);
        }
    }

    pub(super) fn bounded_source(&self, init: &Expression<'_>) -> Option<Option<(String, usize)>> {
        if super::names::stripped_temp_name(init, &|call| self.trusted_uuid_call(call)) {
            return Some(None);
        }
        if let Some(map) = super::names::map_get_receiver(init) {
            return self.registry_source(&map, super::RegistryKind::Map);
        }
        if let Some(object) = super::names::member_object(init) {
            return self
                .registry_source(&object, super::RegistryKind::Object)
                .or_else(|| self.registry_source(&object, super::RegistryKind::Array));
        }
        None
    }

    fn registry_source(
        &self,
        name: &str,
        kind: super::RegistryKind,
    ) -> Option<Option<(String, usize)>> {
        super::unique_closed_collection(&self.value_flow, name, kind).then(|| {
            let scope = self.callee_binding_scope(name).unwrap_or(0);
            Some((name.to_string(), scope))
        })
    }

    pub(in super::super::super) fn note_closed_registry_write(
        &mut self,
        target: &AssignmentTarget<'_>,
        _at: u32,
    ) {
        if !self.collect_value_flow {
            return;
        }
        let Some(name) = member_write_receiver(target) else {
            return;
        };
        self.record_registry_mutation(&name);
    }

    pub(in super::super) fn trusted_uuid_call(&self, expr: &Expression<'_>) -> bool {
        let Some(call) = plain_call(expr) else {
            return false;
        };
        match crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(identifier) if identifier.name == "randomUUID" => {
                self.imported_random_uuid()
            }
            Expression::StaticMemberExpression(member)
                if member.property.name == "randomUUID"
                    && identifier_name(&member.object).as_deref() == Some("crypto") =>
            {
                self.trusted_crypto_receiver()
            }
            _ => false,
        }
    }

    pub(super) fn record_registry_mutation(&mut self, name: &str) {
        let Some(scope) = self.callee_binding_scope(name) else {
            return;
        };
        let seen = self
            .value_flow
            .registry_mutations
            .iter()
            .any(|mutation| mutation.name == name && mutation.scope == scope);
        if seen {
            return;
        }
        self.value_flow.registry_mutations.push(RegistryMutation {
            name: name.to_string(),
            scope,
        });
    }

    fn imported_random_uuid(&self) -> bool {
        if self.non_import_local("randomUUID") {
            return false;
        }
        self.call_import_bindings.iter().any(|binding| {
            !binding.is_type_only
                && binding.kind == ImportedBindingKind::Named
                && binding.local == "randomUUID"
                && binding.imported == "randomUUID"
                && crypto_module(&binding.specifier)
        })
    }

    fn trusted_crypto_receiver(&self) -> bool {
        if self.non_import_local("crypto") {
            return false;
        }
        let mut imports = self
            .call_import_bindings
            .iter()
            .filter(|binding| !binding.is_type_only && binding.local == "crypto");
        match imports.next() {
            None => true,
            Some(binding) => {
                imports.next().is_none()
                    && crypto_module(&binding.specifier)
                    && matches!(
                        binding.kind,
                        ImportedBindingKind::Default | ImportedBindingKind::Namespace
                    )
            }
        }
    }

    fn non_import_local(&self, name: &str) -> bool {
        let Some(scope) = self.callee_binding_scope(name) else {
            return false;
        };
        if scope != 0 {
            return true;
        }
        let imported = self
            .call_import_bindings
            .iter()
            .any(|binding| !binding.is_type_only && binding.local == name);
        if imported {
            return false;
        }
        self.value_flow
            .bindings
            .iter()
            .any(|binding| binding.scope == scope && binding.name == name)
            || self
                .value_flow
                .parameters
                .iter()
                .any(|parameter| parameter.scope == scope && parameter.name == name)
            || self.callable_binding_id(name).is_some()
    }
}

fn crypto_module(specifier: &str) -> bool {
    specifier == "node:crypto" || specifier == "crypto"
}

fn mutating_receiver(call: &CallExpression<'_>) -> Option<String> {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    if !MUTATING_METHODS.contains(&member.property.name.as_str()) {
        return None;
    }
    identifier_name(&member.object)
}

fn assign_target(call: &CallExpression<'_>) -> Option<String> {
    let name = super::super::super::simple_callee_name(&call.callee)?;
    if !matches!(
        name.as_str(),
        "Object.assign"
            | "Object.defineProperty"
            | "Object.defineProperties"
            | "Object.setPrototypeOf"
            | "Reflect.set"
            | "Reflect.defineProperty"
            | "Reflect.deleteProperty"
    ) {
        return None;
    }
    identifier_name(call.arguments.first()?.as_expression()?)
}

fn member_write_receiver(target: &AssignmentTarget<'_>) -> Option<String> {
    let object = match target {
        AssignmentTarget::StaticMemberExpression(member) => &member.object,
        AssignmentTarget::ComputedMemberExpression(member) => &member.object,
        _ => return None,
    };
    identifier_name(object)
}
