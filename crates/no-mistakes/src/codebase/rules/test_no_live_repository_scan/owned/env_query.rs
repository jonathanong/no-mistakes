use super::{env, path_provenance, Owners};
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;

impl Owners<'_> {
    pub(super) fn overlapping_env(
        &self,
        file: &std::path::Path,
        offset: u32,
        name: &str,
        facts: &dyn TsFactLookup,
    ) -> bool {
        let overlap = self.setups.concurrent(file)
            || facts
                .get_ts_facts(file)
                .and_then(path_provenance)
                .is_some_and(|paths| {
                    paths
                        .lifecycle_overlaps
                        .iter()
                        .any(|(start, end)| *start <= offset && offset < *end)
                });
        overlap
            && (self
                .imported
                .has_cleanup(file, name, facts, &self.consumed, &self.fixture_dirs)
                || self.setups.has_cleanup(
                    file,
                    name,
                    facts,
                    &self.imported,
                    &self.consumed,
                    &self.fixture_dirs,
                ))
    }

    pub(super) fn env_owned(
        &self,
        name: &str,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> bool {
        if !self.env.all_owned(name) {
            return self.local_env_owned(name, site, facts);
        }
        let Some(entries) = self.entries(site_source_node(site)) else {
            return false;
        };
        !entries.is_empty()
            && entries.iter().all(|entry| {
                if self.overlapping_env(&entry.file, entry.offset, name, facts) {
                    return false;
                }
                facts
                    .get_ts_facts(&entry.file)
                    .and_then(path_provenance)
                    .is_some_and(|provenance| {
                        if env::cleanup_read(provenance, name, entry.offset) {
                            return false;
                        }
                        env::dominated(provenance, name, entry.offset, &self.fixture_dirs)
                            || self.imported.owns(
                                &entry.file,
                                name,
                                facts,
                                &self.consumed,
                                &self.fixture_dirs,
                            )
                            || self.setups.owns(
                                &entry.file,
                                name,
                                facts,
                                &self.imported,
                                &self.consumed,
                                &self.fixture_dirs,
                            )
                    })
            })
    }
}
