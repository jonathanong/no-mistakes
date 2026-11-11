use super::{path_provenance, Consumed, ImportedWrites, TsFactLookup, WriteScope};
use crate::fx::FxHashSet;
use std::path::Path;

#[derive(Default)]
pub(super) struct Lifecycle {
    cleanup_names: FxHashSet<String>,
    cleanup_all: bool,
}

impl Lifecycle {
    pub(super) fn record(&mut self, file: &Path, facts: &dyn TsFactLookup) {
        let Some(paths) = facts.get_ts_facts(file).and_then(path_provenance) else {
            return;
        };
        self.cleanup_all |= !paths.env_resets.is_empty();
        self.cleanup_names.extend(
            paths
                .env_writes
                .iter()
                .filter(|write| {
                    matches!(
                        write.scope,
                        WriteScope::AfterEach(..) | WriteScope::AfterAll(..)
                    )
                })
                .map(|write| write.name.clone()),
        );
    }
}

impl ImportedWrites<'_> {
    pub(in super::super) fn has_cleanup(
        &self,
        file: &Path,
        name: &str,
        facts: &dyn TsFactLookup,
        consumed: &Consumed,
        fixture_dirs: &[String],
    ) -> bool {
        let mut summaries = self.summaries.borrow_mut();
        let summary = summaries
            .entry(file.to_path_buf())
            .or_insert_with(|| self.collect(file, facts, consumed, fixture_dirs));
        summary.lifecycle.cleanup_all || summary.lifecycle.cleanup_names.contains(name)
    }
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;
