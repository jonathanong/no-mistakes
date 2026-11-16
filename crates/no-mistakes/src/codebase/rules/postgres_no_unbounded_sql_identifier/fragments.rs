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
        let Some((owner, false)) = super::partial::text_owner(self.bindings, call) else {
            return false;
        };
        match (owner, source) {
            (
                TextOwner::Local(owner, owner_scope),
                Provenance::Reference {
                    name,
                    scope,
                    offset,
                    members,
                },
            ) if members.as_slice() == ["text"] && owner == name && owner_scope == *scope => {
                self.proof.binding(&call.file, *scope, name, *offset, false)
                    && self.proof.binds_at(&call.file, *scope, name, *offset)
            }
            (
                TextOwner::Namespace(owner, export),
                Provenance::Namespace {
                    local,
                    exported,
                    members,
                },
            ) if members.as_slice() == [exported.as_str(), "text"]
                && owner == local
                && export == exported =>
            {
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
