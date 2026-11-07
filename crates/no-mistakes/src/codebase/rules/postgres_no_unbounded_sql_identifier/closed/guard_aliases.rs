use super::*;
use crate::codebase::dependencies::extract::{CallableId, TextOrigin};
use crate::fx::FxHashSet;

pub(super) fn proven(
    graph: &DepGraph,
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    flow: &ValueFlowFacts,
    reference: (&str, usize, u32, &[String]),
) -> bool {
    let Some(context) = flow.calls.iter().find_map(|value_call| {
        ((value_call.offset, value_call.end) == call.span).then_some(value_call.context)
    }) else {
        return false;
    };
    Proof {
        graph,
        opts,
        facts,
        bindings,
        call,
        flow,
    }
    .reference(reference, context, &mut FxHashSet::default())
}

struct Proof<'a, 'b> {
    graph: &'a DepGraph,
    opts: &'a CompiledOptions,
    facts: &'a CheckFactMap,
    bindings: &'a Bindings<'b>,
    call: &'a SqlCall<'a>,
    flow: &'a ValueFlowFacts,
}

impl Proof<'_, '_> {
    fn reference(
        &self,
        reference: (&str, usize, u32, &[String]),
        context: Option<CallableId>,
        seen: &mut FxHashSet<(usize, String, u32)>,
    ) -> bool {
        let (name, scope, offset, members) = reference;
        if self.flow.finite_identifier_guards.iter().any(|guard| {
            guard.name == name
                && (if matches!(guard.kind, GuardKind::Lookup) {
                    self.bindings
                        .value(&self.call.file, scope, name, offset)
                        .is_some_and(|binding| bounded(self.flow, binding, members, true))
                } else {
                    guard.members.as_slice() == members
                })
                && guard.context == context
                && offset >= guard.from
                && offset < guard.until
                && guard_subject_holds(
                    self.flow,
                    self.bindings,
                    &self.call.file,
                    name,
                    scope,
                    offset,
                    guard,
                )
                && !written_since(self.flow, guard, offset)
                && guard_applies(
                    self.graph,
                    self.opts,
                    self.facts,
                    self.bindings,
                    &self.call.file,
                    guard,
                )
        }) {
            return true;
        }
        let Some(binding) = self.bindings.value(&self.call.file, scope, name, offset) else {
            return false;
        };
        if binding_proven(
            self.graph,
            self.facts,
            self.bindings,
            &self.call.file,
            self.flow,
            binding,
            members,
        ) {
            return true;
        }
        let key = (binding.scope, binding.name.clone(), binding.declared_at);
        if !members.is_empty() || !binding.immutable || !seen.insert(key.clone()) {
            return false;
        }
        let origin = self
            .flow
            .identifier_text_origins
            .as_ref()
            .and_then(|origins| origins.binding(binding.scope, &binding.name, binding.declared_at));
        let proven = self.expression(&binding.value, origin, seen);
        seen.remove(&key);
        proven
    }

    fn expression(
        &self,
        value: &ValueExpression,
        origin: Option<&TextOrigin>,
        seen: &mut FxHashSet<(usize, String, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Named {
                name,
                scope,
                offset,
                context,
            } => self.reference((name, scope.unwrap_or(0), *offset, &[]), *context, seen),
            ValueExpression::Derived(parts) => {
                !parts.is_empty()
                    && parts.iter().enumerate().all(|(index, part)| {
                        self.expression(
                            part,
                            origin
                                .and_then(|origin| origin.parts.get(index))
                                .and_then(Option::as_ref),
                            seen,
                        )
                    })
            }
            ValueExpression::Member(_, _) => {
                let mut inner = value;
                let mut members = Vec::new();
                while let ValueExpression::Member(base, key) = inner {
                    members.push(key.clone());
                    inner = base;
                }
                members.reverse();
                if let ValueExpression::Named {
                    name,
                    scope,
                    offset,
                    context,
                } = inner
                {
                    self.reference(
                        (name, scope.unwrap_or(0), *offset, &members),
                        *context,
                        seen,
                    )
                } else {
                    false
                }
            }
            ValueExpression::Unknown => origin.is_some_and(|origin| origin.literal),
            ValueExpression::Call(_, _) | ValueExpression::Awaited(_) => {
                self.bindings.identifier_value(&self.call.file, value, &[])
            }
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "guard_aliases_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "composition_tests.rs"]
mod composition_tests;
