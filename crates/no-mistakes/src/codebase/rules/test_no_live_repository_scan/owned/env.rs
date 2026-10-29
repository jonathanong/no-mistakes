//! Which `process.env` variables are only ever set to an owned temp root.
use super::path_provenance;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathProvenance, WriteScope};
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[derive(Default)]
pub(super) struct EnvWrites {
    /// A write this pass cannot attribute to a variable, or a file it cannot read.
    poisoned: bool,
    /// Per variable: whether every write anywhere is an owned temp root.
    owned: FxHashMap<String, bool>,
}

impl EnvWrites {
    pub(super) fn new(facts: &dyn TsFactLookup, graph_files: &[PathBuf]) -> Self {
        let mut writes = Self::default();
        // A file with no path facts was outside the rule's scope; a file whose
        // facts failed to build may hide writes.
        let analyzed = graph_files
            .iter()
            .filter_map(|file| facts.get_ts_facts(file))
            .filter(|ts| {
                ts.module_bindings
                    .as_deref()
                    .is_some_and(|module| module.path_provenance.is_some())
            });
        for ts in analyzed {
            let provenance = path_provenance(ts);
            writes.poisoned |= provenance.is_none_or(|found| found.env_unknown_write);
            for write in provenance.iter().flat_map(|found| &found.env_writes) {
                let entry = writes.owned.entry(write.name.clone()).or_insert(true);
                *entry &= write.origin.anchor == PathAnchor::Temp
                    && !write.origin.segments.iter().any(|segment| segment == "..");
            }
        }
        writes
    }

    pub(super) fn all_owned(&self, name: &str) -> bool {
        !self.poisoned && self.owned.get(name).copied().unwrap_or(false)
    }
}

/// An unconditional owned write of `name` that must have run before `offset`.
pub(in super::super) fn dominated(provenance: &PathProvenance, name: &str, offset: u32) -> bool {
    provenance
        .env_writes
        .iter()
        .filter(|write| write.name == name && write.origin.anchor == PathAnchor::Temp)
        .any(|write| match write.scope {
            WriteScope::Program => write.end <= offset,
            WriteScope::Function(start, end) => {
                start <= offset && offset < end && write.end <= offset
            }
            // A hook runs before every test body registered in the same scope.
            WriteScope::Hook(start, end) => provenance
                .test_callbacks
                .iter()
                .any(|(from, to)| start <= *from && *to <= end && *from <= offset && offset < *to),
            WriteScope::Conditional => false,
        })
}
