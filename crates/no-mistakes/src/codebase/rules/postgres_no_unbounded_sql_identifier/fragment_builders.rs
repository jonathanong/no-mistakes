use super::*;
use crate::codebase::dependencies::extract::{ValueCall, ValueFlowFacts};
use crate::fx::{FxHashMap, FxHashSet};

#[path = "fragment_builder_helpers.rs"]
mod helpers;
#[path = "fragment_builder_lifecycle.rs"]
mod lifecycle;

#[derive(Clone, Hash, PartialEq, Eq)]
enum Key {
    Binding(usize, String, u32, bool),
    Call(u32, u32, bool),
}

#[cfg(test)]
#[path = "fragment_builder_boundary_tests.rs"]
mod boundary_tests;

pub(super) struct Proof<'a, 'facts> {
    graph: &'a DepGraph,
    opts: &'a CompiledOptions,
    bindings: &'a Bindings<'facts>,
    file: &'a Path,
    flow: &'a ValueFlowFacts,
    calls: FxHashMap<(u32, u32), &'a ValueCall>,
    active: FxHashSet<Key>,
    done: FxHashMap<Key, bool>,
}

impl<'a, 'facts> Proof<'a, 'facts> {
    pub(super) fn new(
        graph: &'a DepGraph,
        opts: &'a CompiledOptions,
        bindings: &'a Bindings<'facts>,
        file: &'a Path,
        flow: &'a ValueFlowFacts,
    ) -> Self {
        Self {
            graph,
            opts,
            bindings,
            file,
            flow,
            calls: flow
                .calls
                .iter()
                .map(|call| ((call.offset, call.end), call))
                .collect(),
            active: FxHashSet::default(),
            done: FxHashMap::default(),
        }
    }

    pub(super) fn binding(
        &mut self,
        scope: usize,
        name: &str,
        at: u32,
        default_only: bool,
    ) -> bool {
        let Some(binding) = self.bindings.value(self.file, scope, name, at) else {
            return false;
        };
        let key = Key::Binding(scope, name.into(), binding.declared_at, default_only);
        self.memoized(key, |proof| {
            binding.immutable
                && binding.invalidated_at.is_none()
                && !proof
                    .flow
                    .registry_mutations
                    .iter()
                    .any(|write| write.scope == scope && write.name == name)
                && !proof
                    .flow
                    .identifier_writes
                    .iter()
                    .any(|write| write.scope == scope && write.name == name)
                && proof.expression(&binding.value, default_only)
                && proof.lifecycle(binding)
        })
    }

    fn expression(&mut self, expression: &ValueExpression, default_only: bool) -> bool {
        match expression {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.binding(scope.unwrap_or(0), name, *offset, default_only),
            ValueExpression::Call(start, end) => {
                let Some(call) = self.calls.get(&(*start, *end)).copied() else {
                    return false;
                };
                self.memoized(Key::Call(*start, *end, default_only), |proof| {
                    if proof.flow.tagged_templates.contains_key(&(*start, *end)) {
                        let tags = if default_only {
                            &[][..]
                        } else {
                            &proof.opts.embedded.trusted_sql_tags
                        };
                        return super::super::super::postgres_sql_cross_file::is_text_tag(
                            proof.bindings,
                            proof.file,
                            call,
                            tags,
                        );
                    }
                    if let ValueExpression::Member(base, method) = &call.callee {
                        return method == "append"
                            && proof.expression(base, true)
                            && proof.append_argument(call);
                    }
                    helpers::returned_builder(proof, call)
                })
            }
            _ => false,
        }
    }

    fn append_argument(&mut self, call: &ValueCall) -> bool {
        !call.bound
            && call.spread_at.is_none()
            && call.arguments.len() == 1
            && self.expression(&call.arguments[0], true)
    }

    fn memoized(&mut self, key: Key, evaluate: impl FnOnce(&mut Self) -> bool) -> bool {
        if let Some(result) = self.done.get(&key) {
            return *result;
        }
        if !self.active.insert(key.clone()) {
            return false;
        }
        let result = evaluate(self);
        self.active.remove(&key);
        self.done.insert(key, result);
        result
    }
}
