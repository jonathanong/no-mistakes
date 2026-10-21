use super::*;
use crate::codebase::dependencies::extract::{ValueBinding, ValueFlowFacts};
use crate::fx::{fx_map, FxHashMap, FxHashSet};

/// Borrowed syntactic indexes; neither value resolution nor another graph.
pub(crate) struct Bindings<'a> {
    pub(super) facts: &'a CheckFactMap,
    values: FxHashMap<(&'a Path, usize, &'a str), Vec<&'a ValueBinding>>,
    calls: FxHashMap<&'a Path, FxHashSet<(u32, u32)>>,
    iterations: FxHashMap<(&'a Path, usize, &'a str), Vec<&'a ValueBinding>>,
}
impl<'a> Bindings<'a> {
    pub(crate) fn new(facts: &'a CheckFactMap) -> Self {
        let mut result = Self {
            facts,
            calls: fx_map(),
            values: fx_map(),
            iterations: fx_map(),
        };
        for file in facts.graph_file_universe() {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            result.add(file, &ts.value_flow);
        }
        for values in result
            .values
            .values_mut()
            .chain(result.iterations.values_mut())
        {
            values.sort_by_key(|binding| binding.declared_at);
        }
        result
    }
    pub(crate) fn with_calls(facts: &'a CheckFactMap, calls: &'a [SqlCall<'_>]) -> Self {
        let mut index = Self::new(facts);
        for call in calls {
            index.calls.entry(&call.file).or_default().insert(call.span);
        }
        index
    }
    fn add(&mut self, file: &'a Path, flow: &'a ValueFlowFacts) {
        for binding in &flow.bindings {
            self.values
                .entry((file, binding.scope, &binding.name))
                .or_default()
                .push(binding);
        }
        for binding in &flow.iteration_bindings {
            self.iterations
                .entry((file, binding.scope, &binding.name))
                .or_default()
                .push(binding);
        }
    }
    pub(super) fn value(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
    ) -> Option<&'a ValueBinding> {
        let values = self.values.get(&(file, scope, name))?;
        let index = values.partition_point(|value| value.declared_at <= at);
        let binding = *values.get(index.checked_sub(1)?)?;
        binding
            .invalidated_at
            .is_none_or(|write| write > at)
            .then_some(binding)
    }
    pub(crate) fn env_alias(&self, file: &Path, scope: usize, name: &str, at: u32) -> bool {
        let Some(binding) = self.value(file, scope, name, at) else {
            return false;
        };
        self.env(file, &binding.value, &[]).is_some()
    }
    pub(super) fn env(
        &self,
        file: &Path,
        value: &ValueExpression,
        members: &[String],
    ) -> Option<String> {
        self.search_env(file, value, members, &mut super::search::Search::default())
    }
    pub(crate) fn query_projection(&self, file: &Path, scope: usize, name: &str, at: u32) -> bool {
        self.row_named(file, scope, name, at, &mut crate::fx::fx_set())
    }
    fn row_named(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        seen: &mut crate::fx::FxHashSet<(usize, String, u32)>,
    ) -> bool {
        let value = self.value(file, scope, name, at).or_else(|| {
            let values = self.iterations.get(&(file, scope, name))?;
            let index = values.partition_point(|value| value.declared_at <= at);
            values.get(index.checked_sub(1)?).copied()
        });
        value.is_some_and(|binding| {
            binding.invalidated_at.is_none_or(|end| at < end)
                && seen.insert((scope, name.to_owned(), binding.declared_at))
                && self.row(file, &binding.value, seen)
        })
    }
    fn row(
        &self,
        file: &Path,
        value: &ValueExpression,
        seen: &mut crate::fx::FxHashSet<(usize, String, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Call(start, end) => self
                .calls
                .get(file)
                .is_some_and(|calls| calls.contains(&(*start, *end))),
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.row_named(file, scope.unwrap_or(0), name, *offset, seen),
            ValueExpression::Derived(parts) => parts.iter().any(|part| self.row(file, part, seen)),
            ValueExpression::Either(left, right)
            | ValueExpression::Nullish(left, right)
            | ValueExpression::Default(left, right) => {
                self.row(file, left, seen) || self.row(file, right, seen)
            }
            ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => {
                self.row(file, base, seen)
            }
            _ => false,
        }
    }
}
