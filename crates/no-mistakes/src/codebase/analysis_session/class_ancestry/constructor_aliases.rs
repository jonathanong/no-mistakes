//! Narrow identity proofs from the already-prepared lexical value facts.
use super::*;
use crate::codebase::dependencies::extract::{FunctionCall, ValueExpression};

impl Builder<'_> {
    pub(super) fn aliased_constructor(
        &mut self,
        path: &Path,
        file: &TsFileFacts,
        call: &FunctionCall,
    ) -> Option<(NodeId, bool)> {
        let scope = call.callee_binding_scope?;
        let aliases: Vec<_> = file
            .callable_aliases
            .iter()
            .filter(|alias| {
                alias.binding_scope == scope
                    && alias.local == call.callee
                    && call.offset >= alias.declared_at
                    && alias
                        .invalidated_at
                        .is_none_or(|offset| call.offset < offset)
            })
            .collect();
        if aliases.len() == 1 {
            let alias = aliases[0];
            if !unmutated_reference(file, &alias.target, alias.declared_at, alias.scope_id) {
                return None;
            }
            return self
                .reference(path, &alias.target)
                .map(|target| (target, false));
        }
        if !aliases.is_empty() {
            return None;
        }
        let mut bindings = file
            .value_flow
            .bindings
            .iter()
            .filter(|binding| binding.scope == scope && binding.name == call.callee);
        let binding = bindings.next()?;
        if bindings.next().is_some()
            || !binding.immutable
            || binding.invalidated_at.is_some()
            || !binding.mutation_contexts.is_empty()
            || binding.declared_at > call.offset
        {
            return None;
        }
        let (name, nonnullish) = binding_reference(file, &binding.value)?;
        if !unmutated_reference(file, &name, binding.declared_at, binding.owner) {
            return None;
        }
        self.reference(path, &name)
            .map(|target| (target, nonnullish))
    }
}

pub(super) fn binding_reference(
    file: &TsFileFacts,
    value: &ValueExpression,
) -> Option<(String, bool)> {
    // A proved class on the left cannot be nullish. Conditional/opaque values
    // supply no corresponding proof and never erase an unknown alternative.
    match value {
        ValueExpression::Nullish(left, _) => {
            imported_reference(left, file).map(|name| (name, true))
        }
        value => imported_reference(value, file).map(|name| (name, false)),
    }
}

fn imported_reference(value: &ValueExpression, file: &TsFileFacts) -> Option<String> {
    match value {
        ValueExpression::Named {
            name,
            scope: None | Some(0),
            ..
        } if file
            .imported_bindings
            .iter()
            .filter(|binding| binding.local == *name && !binding.is_type_only)
            .count()
            == 1 =>
        {
            Some(name.clone())
        }
        ValueExpression::Member(base, member) => {
            Some(format!("{}.{}", imported_reference(base, file)?, member))
        }
        _ => None,
    }
}

pub(super) fn unmutated_reference(
    file: &TsFileFacts,
    name: &str,
    offset: u32,
    context: Option<CallableId>,
) -> bool {
    let root = name.split('.').next().unwrap_or(name);
    [
        file.value_flow.receiver_mutations.as_deref(),
        file.value_flow.possible_receiver_mutations.as_deref(),
    ]
    .into_iter()
    .flatten()
    .flat_map(|scopes| scopes.values())
    .filter_map(|names| names.get(root))
    .all(|history| {
        history.summary.as_ref().is_none_or(|summary| {
            summary.offset > offset && summary.contexts.iter().all(|writer| *writer == context)
        })
    })
}
