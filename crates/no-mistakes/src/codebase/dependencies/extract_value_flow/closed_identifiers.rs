use super::*;
use oxc_ast::ast::{BindingPattern, CallExpression, Expression};

#[path = "closed_identifier_guards.rs"]
mod guards;

#[path = "closed_identifier_types.rs"]
mod types;
pub(crate) use types::*;

pub(crate) fn unique_closed_collection(
    flow: &ValueFlowFacts,
    name: &str,
    kind: RegistryKind,
) -> bool {
    if flow
        .parameters
        .iter()
        .any(|parameter| parameter.name == name)
    {
        return false;
    }
    let matched: Vec<_> = flow
        .bindings
        .iter()
        .filter(|binding| binding.name == name)
        .collect();
    let [binding] = matched.as_slice() else {
        return false;
    };
    if flow
        .registry_mutations
        .iter()
        .any(|mutation| mutation.name == binding.name && mutation.scope == binding.scope)
    {
        return false;
    }
    flow.closed_string_registries.iter().any(|registry| {
        registry.kind == kind
            && registry.name == binding.name
            && registry.scope == binding.scope
            && registry.declared_at == binding.declared_at
    })
}

impl ImportCollector {
    fn registry_builtins_visible(&self, init: &Expression<'_>, kind: RegistryKind) -> bool {
        let visible = |name: &str| {
            self.callee_binding_scope(name).is_none()
                && !self
                    .call_import_bindings
                    .iter()
                    .any(|binding| !binding.is_type_only && binding.local == name)
        };
        let constructor = match kind {
            RegistryKind::Set => visible("Set"),
            RegistryKind::Map => visible("Map"),
            _ => true,
        };
        constructor
            && (registries::frozen_argument(crate::codebase::ts_source::unwrap_ts_wrappers(init))
                .is_none()
                || visible("Object"))
    }

    pub(crate) fn note_closed_identifier(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
        declared_at: u32,
    ) {
        let BindingPattern::BindingIdentifier(identifier) = pattern else {
            return;
        };
        let name = identifier.name.to_string();
        let scope = self
            .callee_binding_scope(&name)
            .unwrap_or_else(|| self.current_lexical_scope_id());
        rows::note_binding(self, &name, scope, init);
        if let Some((kind, keys)) = registries::registry(init)
            .filter(|(kind, _)| self.registry_builtins_visible(init, *kind))
        {
            if kind == RegistryKind::MetadataArray {
                self.metadata_registry_names.insert(name.clone());
            }
            self.value_flow
                .closed_string_registries
                .push(ClosedStringRegistry {
                    name: name.clone(),
                    scope,
                    declared_at,
                    keys,
                    kind,
                });
        }
        if let Some(source) = self.bounded_source(init) {
            self.value_flow.bounded_bindings.push(BoundedBinding {
                name,
                scope,
                declared_at,
                source,
                members: Vec::new(),
                requires_guard: false,
            });
        }
        metadata::note_lookup(self, pattern, init, scope, declared_at);
    }
}

#[path = "sql_fragment_members.rs"]
mod fragment_members;
#[path = "closed_identifier_hex.rs"]
mod hex;
#[path = "closed_identifier_generated.rs"]
mod generated;
pub(crate) use generated::GeneratedIdentifierReturn;
#[path = "closed_identifier_metadata.rs"]
pub(super) mod metadata;
#[path = "closed_identifier_metadata_mutations.rs"]
mod metadata_mutations;
#[path = "closed_identifier_names.rs"]
mod names;
#[path = "closed_identifier_registries.rs"]
mod registries;
#[path = "closed_identifier_escapes.rs"]
mod registry_escapes;
#[path = "closed_identifier_mutations.rs"]
mod registry_mutations;
#[path = "closed_identifier_rows.rs"]
pub(crate) mod rows;

pub(super) fn plain_call<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    let expr = crate::codebase::ts_source::unwrap_ts_wrappers(expr);
    match expr {
        Expression::CallExpression(call) if !call.optional => Some(call.as_ref()),
        _ => None,
    }
}

pub(super) fn literal_string(expr: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let mut text = String::new();
            for quasi in &template.quasis {
                text.push_str(&crate::ast::template_quasi_text(quasi));
            }
            Some(text)
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "closed_identifiers_tests.rs"]
mod tests;
