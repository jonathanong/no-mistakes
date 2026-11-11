use super::owned_origin;
use crate::codebase::ts_source::module_facts::arguments::{EnvWrite, PathProvenance, WriteScope};
use crate::fx::FxHashSet;

#[derive(Default)]
pub(super) struct CleanupProof {
    each: crate::fx::FxHashMap<(u32, u32), FxHashSet<String>>,
    all: crate::fx::FxHashMap<(u32, u32), FxHashSet<String>>,
}

impl CleanupProof {
    pub(super) fn new(provenance: &PathProvenance, fixture_dirs: &[String]) -> Self {
        let mut proof = Self::default();
        for write in &provenance.env_writes {
            if !owned_origin(&write.origin, fixture_dirs) {
                continue;
            }
            match write.scope {
                WriteScope::BeforeEach(start, end) => {
                    proof
                        .each
                        .entry((start, end))
                        .or_default()
                        .insert(write.name.clone());
                    proof
                        .all
                        .entry((start, end))
                        .or_default()
                        .insert(write.name.clone());
                }
                WriteScope::Hook(start, end) => {
                    proof
                        .all
                        .entry((start, end))
                        .or_default()
                        .insert(write.name.clone());
                }
                _ => {}
            }
        }
        proof
    }

    fn names(&self, scope: WriteScope) -> Option<&FxHashSet<String>> {
        match scope {
            WriteScope::AfterEach(start, end) => self.each.get(&(start, end)),
            WriteScope::AfterAll(start, end) => self.all.get(&(start, end)),
            _ => None,
        }
    }

    pub(super) fn paired_cleanup(&self, cleanup: &EnvWrite) -> bool {
        self.names(cleanup.scope)
            .is_some_and(|names| names.contains(&cleanup.name))
    }

    pub(super) fn constrain_resets(
        &self,
        provenance: &PathProvenance,
        allowed: &mut Option<FxHashSet<String>>,
    ) {
        for reset in &provenance.env_resets {
            let names = self.names(*reset);
            if let Some(previous) = allowed {
                previous.retain(|name| names.is_some_and(|names| names.contains(name)));
            } else {
                *allowed = Some(names.cloned().unwrap_or_default());
            }
        }
    }
}

pub(in super::super::super) fn cleanup_read(
    provenance: &PathProvenance,
    name: &str,
    offset: u32,
) -> bool {
    provenance
        .cleanup_callbacks
        .iter()
        .any(|(start, end)| *start <= offset && offset < *end)
        && (!provenance.env_resets.is_empty()
            || provenance.env_writes.iter().any(|write| {
                write.name == name
                    && matches!(
                        write.scope,
                        WriteScope::AfterEach(..) | WriteScope::AfterAll(..)
                    )
            }))
}

/// An unconditional owned write of `name` that must have run before `offset`.
pub(in super::super::super) fn dominated(
    provenance: &PathProvenance,
    name: &str,
    offset: u32,
    fixture_dirs: &[String],
) -> bool {
    provenance
        .env_writes
        .iter()
        .filter(|write| write.name == name && owned_origin(&write.origin, fixture_dirs))
        .any(|write| match write.scope {
            WriteScope::Program => write.end <= offset,
            WriteScope::Function(start, end) => {
                start <= offset && offset < end && write.end <= offset
            }
            WriteScope::Hook(start, end) | WriteScope::BeforeEach(start, end) => provenance
                .test_callbacks
                .iter()
                .any(|(from, to)| start <= *from && *to <= end && *from <= offset && offset < *to),
            WriteScope::Conditional | WriteScope::AfterEach(..) | WriteScope::AfterAll(..) => false,
        })
}
