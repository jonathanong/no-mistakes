use super::env;
use super::{path_provenance, Owners};
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::codebase::ts_source::module_facts::arguments::WriteScope;

impl Owners<'_> {
    pub(super) fn local_env_owned(
        &self,
        name: &str,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> bool {
        let source = site_source_node(site);
        if !self.env.local_known()
            || self.closure_unknown(&site.file)
            || !self.proven_target(&source)
        {
            return false;
        }
        let immediate = facts
            .get_ts_facts(&site.file)
            .and_then(path_provenance)
            .is_some_and(|paths| paths.immediate_env_reads.contains(&site.offset));
        if !immediate {
            return false;
        }
        let Some(callers) = self
            .callers
            .get(&source)
            .filter(|callers| !callers.is_empty())
        else {
            return false;
        };
        callers.iter().all(|caller| {
            let Some(paths) = facts.get_ts_facts(&caller.file).and_then(path_provenance) else { return false };
            let Some(end) = paths.test_env_calls.get(&caller.offset) else { return false };
            paths.env_writes.iter().any(|write| {
                write.end == *end && write.name == name
                    && env::owned_origin(&write.origin, &self.fixture_dirs)
                    && matches!(write.scope, WriteScope::Function(start, end) if start <= caller.offset && caller.offset < end)
            })
        })
    }
}
