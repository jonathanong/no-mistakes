use super::*;
use crate::codebase::dependencies::extract::{
    CallableId, ValueBinding, ValueCall, ValueExpression,
};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

mod calls;
mod state;
use state::{Frame, Key, Prepared};

pub(super) fn proves(
    graph: &DepGraph,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    file: &Path,
    binding: &ValueBinding,
) -> bool {
    Proof {
        graph,
        facts,
        bindings,
        files: FxHashMap::default(),
        active: FxHashSet::default(),
        done: FxHashMap::default(),
    }
    .binding(file, binding, None)
}

struct Proof<'a, 'b> {
    graph: &'a DepGraph,
    facts: &'a CheckFactMap,
    bindings: &'a Bindings<'b>,
    files: FxHashMap<PathBuf, Prepared<'a>>,
    active: FxHashSet<Key>,
    done: FxHashMap<Key, bool>,
}

impl Proof<'_, '_> {
    fn expression(
        &mut self,
        file: &Path,
        value: &ValueExpression,
        frame: Option<&Frame>,
        awaited: bool,
    ) -> bool {
        match value {
            ValueExpression::Named {
                name,
                scope: Some(scope),
                offset,
                ..
            } => self.named(file, *scope, name, *offset, frame),
            ValueExpression::Awaited(inner) => self.expression(file, inner, frame, true),
            ValueExpression::Call(start, end) => self.call(file, (*start, *end), frame, awaited),
            _ => false,
        }
    }

    fn binding(&mut self, file: &Path, binding: &ValueBinding, frame: Option<&Frame>) -> bool {
        let key = Key::Binding(
            file.to_path_buf(),
            binding.scope,
            binding.name.clone(),
            binding.declared_at,
            frame.cloned(),
        );
        self.memoized(key, |proof| {
            binding.immutable
                && proof
                    .facts
                    .get_ts_facts(file)
                    .is_some_and(|ts| super::bounded(&ts.value_flow, binding, &[], false))
                || binding.immutable && proof.expression(file, &binding.value, frame, false)
        })
    }

    fn named(
        &mut self,
        file: &Path,
        scope: usize,
        name: &str,
        offset: u32,
        frame: Option<&Frame>,
    ) -> bool {
        if let Some(binding) = self.bindings.value(file, scope, name, offset) {
            return self.binding(file, binding, frame);
        }
        let Some(frame) = frame.filter(|frame| frame.file == file) else {
            return false;
        };
        self.facts.get_ts_facts(file).is_some_and(|ts| {
            ts.value_flow.parameters.iter().any(|parameter| {
                parameter.owner == frame.owner
                    && parameter.scope == scope
                    && parameter.name == name
                    && parameter.projection.is_empty()
                    && parameter.invalidated_at.is_none_or(|at| offset < at)
                    && parameter.mutation_contexts.is_empty()
                    && frame.arguments.get(parameter.index) == Some(&true)
            })
        })
    }

    fn memoized(&mut self, key: Key, evaluate: impl FnOnce(&mut Self) -> bool) -> bool {
        if let Some(done) = self.done.get(&key) {
            return *done;
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

#[cfg(test)]
mod tests;
