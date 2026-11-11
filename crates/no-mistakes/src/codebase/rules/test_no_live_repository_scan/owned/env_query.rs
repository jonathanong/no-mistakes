use super::{env, path_provenance, Owners};
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;

impl Owners<'_> {
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
