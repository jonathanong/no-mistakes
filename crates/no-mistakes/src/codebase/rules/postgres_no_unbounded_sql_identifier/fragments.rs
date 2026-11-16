use super::*;
use crate::codebase::rules::postgres_sql_cross_file::CompiledOptions;

#[path = "fragment_builders.rs"]
mod builders;

pub(super) struct Proofs<'a, 'facts> {
    proof: builders::Proof<'a, 'facts>,
    bindings: &'a Bindings<'facts>,
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
            bindings,
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
        use super::partial::TextOwner;
        let owner = match source {
            Provenance::Reference {
                name,
                scope,
                members,
                ..
            } if members.as_slice() == ["text"] => TextOwner::Local(name, *scope),
            Provenance::Namespace {
                local,
                exported,
                members,
            } if members.as_slice() == [exported.as_str(), "text"] => {
                TextOwner::Namespace(local, exported)
            }
            _ => return false,
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
}

#[cfg(test)]
#[path = "fragment_builders_tests.rs"]
mod builder_tests;

#[cfg(test)]
#[path = "fragment_import_tests.rs"]
mod import_tests;
