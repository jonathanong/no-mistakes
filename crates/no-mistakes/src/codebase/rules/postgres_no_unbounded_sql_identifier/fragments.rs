use super::*;
use crate::codebase::rules::postgres_sql_cross_file::CompiledOptions;
use crate::fx::FxHashMap;

#[path = "fragment_builders.rs"]
mod builders;

pub(super) struct Proofs<'a, 'facts> {
    graph: &'a DepGraph,
    opts: &'a CompiledOptions,
    facts: &'a CheckFactMap,
    bindings: &'a Bindings<'facts>,
    files: FxHashMap<&'a Path, builders::Proof<'a, 'facts>>,
}

impl<'a, 'facts> Proofs<'a, 'facts> {
    pub(super) fn new(
        graph: &'a DepGraph,
        opts: &'a CompiledOptions,
        facts: &'a CheckFactMap,
        bindings: &'a Bindings<'facts>,
    ) -> Self {
        Self {
            graph,
            opts,
            facts,
            bindings,
            files: FxHashMap::default(),
        }
    }

    pub(super) fn proven(&mut self, call: &'a SqlCall<'_>, source: &Provenance) -> bool {
        let Provenance::Reference {
            name,
            scope,
            offset,
            members,
        } = source
        else {
            return false;
        };
        if members.as_slice() != ["text"] {
            return false;
        }
        let Some(ts) = self.facts.get_ts_facts(&call.file) else {
            return false;
        };
        self.files
            .entry(&call.file)
            .or_insert_with(|| {
                builders::Proof::new(
                    self.graph,
                    self.opts,
                    self.bindings,
                    &call.file,
                    &ts.value_flow,
                )
            })
            .binding(*scope, name, *offset, false)
    }
}

#[cfg(test)]
pub(super) fn proven(
    graph: &DepGraph,
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    source: &Provenance,
) -> bool {
    Proofs::new(graph, opts, facts, bindings).proven(call, source)
}

#[cfg(test)]
#[path = "fragment_builders_tests.rs"]
mod builder_tests;
