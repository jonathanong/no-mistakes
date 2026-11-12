use super::*;
use crate::codebase::dependencies::extract::{IdentifierAssignment, ValueBinding, ValueFlowFacts};

mod assignments;
mod closed_types;
mod flow;
mod rows;
pub(crate) use closed_types::literal_array;

use crate::fx::{fx_map, FxHashMap, FxHashSet};
type RowMemberKey<'a> = (&'a Path, usize, &'a str, &'a str);
type AssignmentIndex<'a> = FxHashMap<(&'a Path, usize, &'a str), Vec<&'a IdentifierAssignment>>;
/// Borrowed syntactic indexes; neither value resolution nor another graph.
pub(crate) struct Bindings<'a> {
    pub(super) identifier_returns: Option<&'a super::IdentifierReturns>,
    pub(super) numeric_parameter_bounds_only: bool,
    pub(super) collection_safety: std::cell::RefCell<FxHashMap<(std::path::PathBuf, String), bool>>,
    pub(super) trusted_tags: &'a [crate::codebase::postgres::TrustedSqlTag],
    pub(crate) routes: super::super::graph::RegistryRoutes<'a>,
    pub(super) facts: &'a CheckFactMap,
    pub(super) graph: Option<&'a DepGraph>,
    values: FxHashMap<(&'a Path, usize, &'a str), Vec<&'a ValueBinding>>,
    assignments: std::sync::OnceLock<AssignmentIndex<'a>>,
    calls: FxHashMap<&'a Path, FxHashSet<(u32, u32)>>,
    overwrites: FxHashMap<RowMemberKey<'a>, FxHashSet<(u32, u32)>>,
    iterations: FxHashMap<(&'a Path, usize, &'a str), Vec<&'a ValueBinding>>,
}
impl<'a> Bindings<'a> {
    pub(crate) fn with_trusted_tags(
        mut self,
        tags: &'a [crate::codebase::postgres::TrustedSqlTag],
    ) -> Self {
        self.trusted_tags = tags;
        self
    }
    pub(crate) fn new(facts: &'a CheckFactMap) -> Self {
        let mut result = Self {
            facts,
            identifier_returns: None,
            numeric_parameter_bounds_only: false,
            collection_safety: Default::default(),
            trusted_tags: &[],
            graph: None,
            routes: super::super::graph::RegistryRoutes::new(facts),
            calls: fx_map(),
            values: fx_map(),
            assignments: Default::default(),
            iterations: fx_map(),
            overwrites: fx_map(),
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
    pub(crate) fn with_graph(mut self, graph: &'a DepGraph) -> Self {
        self.graph = Some(graph);
        self
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
        for (start, end, scope, name, member) in &flow.row_literal_overwrites {
            self.overwrites
                .entry((file, *scope, name, member))
                .or_default()
                .insert((*start, *end));
        }
        for binding in &flow.iteration_bindings {
            self.iterations
                .entry((file, binding.scope, &binding.name))
                .or_default()
                .push(binding);
        }
    }
    pub(crate) fn value(
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
}

#[cfg(test)]
#[path = "bindings/reassigned_identifier_tests.rs"]
mod reassigned_identifier_tests;

#[cfg(test)]
#[path = "bindings/assignment_scalar_tests.rs"]
mod assignment_scalar_tests;

#[cfg(test)]
#[path = "bindings/assignment_initializers_tests.rs"]
mod assignment_initializers_tests;

#[cfg(test)]
#[path = "bindings/assignment_boundary_fixture_tests.rs"]
mod assignment_boundary_fixture_tests;
