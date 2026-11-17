use super::*;
use crate::codebase::rules::postgres_sql_cross_file::CompiledOptions;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[path = "fragment_builders.rs"]
mod builders;
#[path = "fragment_recursion.rs"]
mod recursion;

pub(super) struct Proofs<'a, 'facts> {
    proof: builders::Proof<'a, 'facts>,
    graph: &'a DepGraph,
    facts: &'a CheckFactMap,
    bindings: &'a Bindings<'facts>,
    risky_helpers: FxHashMap<(PathBuf, usize, String, u32), bool>,
}

impl<'a, 'facts> Proofs<'a, 'facts> {
    pub(super) fn new(
        graph: &'a DepGraph,
        opts: &'a CompiledOptions,
        facts: &'a CheckFactMap,
        bindings: &'a Bindings<'facts>,
    ) -> Self {
        Self {
            proof: builders::Proof::new(graph, opts, facts, bindings),
            graph,
            facts,
            bindings,
            risky_helpers: FxHashMap::default(),
        }
    }

    pub(super) fn proven(&mut self, call: &SqlCall<'_>, source: &Provenance) -> bool {
        match source {
            Provenance::Reference {
                name,
                scope,
                offset,
                members,
            } if members.as_slice() == ["text"] => {
                self.proof.binding(&call.file, *scope, name, *offset, false)
                    && !self.missing_values(call, source)
            }
            Provenance::Namespace {
                local,
                exported,
                members,
            } if members.as_slice() == [exported.as_str(), "text"] => {
                self.proof.namespace(&call.file, local, exported)
                    && !self.missing_values(call, source)
            }
            _ => false,
        }
    }

    pub(super) fn missing_values(&mut self, call: &SqlCall<'_>, source: &Provenance) -> bool {
        let Some(owner) = super::partial::source_owner(source) else {
            return false;
        };
        if super::partial::text_owner_for(self.bindings, call, owner) != Some(false) {
            return false;
        }
        match source {
            Provenance::Reference {
                name,
                scope,
                offset,
                ..
            } => {
                self.proof.binding(&call.file, *scope, name, *offset, false)
                    && self.proof.binds_at(&call.file, *scope, name, *offset)
            }
            Provenance::Namespace {
                local, exported, ..
            } => {
                self.proof.namespace(&call.file, local, exported)
                    && self.proof.binds_namespace(&call.file, local, exported)
            }
            _ => false,
        }
    }

    pub(super) fn report_text_source(&mut self, call: &SqlCall<'_>, source: &Provenance) -> bool {
        self.missing_values(call, source)
            || self.invalidated_builder(call, source)
            || self.risky_helper_source(call, source)
    }

    fn risky_helper_source(&mut self, call: &SqlCall<'_>, source: &Provenance) -> bool {
        let Provenance::Reference {
            name,
            scope,
            offset,
            ..
        } = source
        else {
            return false;
        };
        let Some(binding) = self.bindings.value(&call.file, *scope, name, *offset) else {
            return false;
        };
        let key = (
            call.file.clone(),
            binding.scope,
            binding.name.clone(),
            binding.declared_at,
        );
        if let Some(found) = self.risky_helpers.get(&key) {
            return *found;
        }
        let found = recursion::unsafe_helper(
            &mut self.proof,
            self.graph,
            self.facts,
            self.bindings,
            call,
            source,
        );
        self.risky_helpers.insert(key, found);
        found
    }

    fn invalidated_builder(&mut self, call: &SqlCall<'_>, source: &Provenance) -> bool {
        let Provenance::Reference {
            name,
            scope,
            offset,
            ..
        } = source
        else {
            return false;
        };
        let Some(binding) = self
            .bindings
            .declared_value(&call.file, *scope, name, *offset)
        else {
            return false;
        };
        binding.invalidated_at.is_some_and(|write| write <= *offset)
            && self.proof.expression(&call.file, &binding.value, false)
    }
}

#[cfg(test)]
#[path = "fragment_builders_tests.rs"]
mod builder_tests;

#[cfg(test)]
#[path = "fragment_import_tests.rs"]
mod import_tests;
