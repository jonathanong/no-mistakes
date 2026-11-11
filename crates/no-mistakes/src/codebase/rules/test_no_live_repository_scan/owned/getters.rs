use super::{path_provenance, Owners};
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::PathOrigin;

impl Owners<'_> {
    pub(super) fn getter_owned(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> bool {
        if origin.segments.iter().any(|part| part == "..")
            || !origin
                .root_tails
                .iter()
                .all(|tail| self.safe_suffix(tail, site, facts, 0))
        {
            return false;
        }
        let Some(offset) = origin.getter else {
            return false;
        };
        let sites = self.graph.call_sites_in_file(&site.file);
        let index = sites.partition_point(|call| call.offset < offset);
        let Some(call) = sites.get(index).filter(|call| call.offset == offset) else {
            return false;
        };
        self.getter_call_owned(call, site, facts)
    }

    fn getter_call_owned(
        &self,
        call: &ResolvedCallSite,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> bool {
        let Some(
            target @ NodeId::Symbol {
                file,
                callable_id: Some(id),
                ..
            },
        ) = &call.target_node
        else {
            return false;
        };
        if !call.alternatives.is_empty()
            || !self.parameter_proven(target)
            || self.closure_unknown(file)
        {
            return false;
        }
        let Some(root) = facts
            .get_ts_facts(file)
            .and_then(path_provenance)
            .and_then(|paths| paths.env_getters.get(&id.0))
        else {
            return false;
        };
        !root.segments.iter().any(|part| part == "..")
            && root
                .env
                .as_deref()
                .is_some_and(|name| self.env_owned(name, site, facts))
    }
}

#[cfg(test)]
#[path = "getters/tests.rs"]
mod tests;
