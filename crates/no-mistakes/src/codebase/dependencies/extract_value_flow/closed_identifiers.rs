use super::*;
use crate::codebase::dependencies::extract::CallableId;
use oxc_ast::ast::{BindingPattern, CallExpression, Expression};

#[path = "closed_identifier_guards.rs"]
mod guards;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClosedStringRegistry {
    pub(crate) name: String,
    pub(crate) scope: usize,
    pub(crate) declared_at: u32,
    pub(crate) keys: Vec<String>,
    pub(crate) kind: RegistryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistryKind {
    Object,
    Array,
    Map,
    Set,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundedBinding {
    pub(crate) name: String,
    pub(crate) scope: usize,
    pub(crate) declared_at: u32,
    pub(crate) source: Option<(String, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FiniteIdentifierGuard {
    pub(crate) context: Option<CallableId>,
    pub(crate) name: String,
    pub(crate) from: u32,
    pub(crate) until: u32,
    pub(crate) kind: GuardKind,
    pub(crate) subject: Option<GuardSubject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuardKind {
    ClosedSet {
        collection: String,
        method: SetMethod,
    },
    Assertion {
        callee: String,
        argument: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SetMethod {
    Has,
    Includes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegistryMutation {
    pub(crate) name: String,
    pub(crate) scope: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdentifierWrite {
    pub(crate) scope: usize,
    pub(crate) name: String,
    pub(crate) at: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuardSubject {
    Binding {
        scope: usize,
        declared_at: u32,
    },
    Parameter {
        scope: usize,
        owner: CallableId,
        index: usize,
    },
}

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
        if let Some((kind, keys)) = registries::registry(init) {
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
            });
        }
    }
}

#[path = "closed_identifier_names.rs"]
mod names;
#[path = "closed_identifier_registries.rs"]
mod registries;
#[path = "closed_identifier_mutations.rs"]
mod registry_mutations;

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
