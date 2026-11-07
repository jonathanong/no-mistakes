use super::{path_provenance, Owners};
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::codebase::rules::test_no_live_repository_scan::provenance::{classify, Verdict};
use crate::codebase::ts_source::module_facts::arguments::PathOrigin;
use crate::fx::FxHashSet;

impl Owners<'_> {
    pub(super) fn param_owned(
        &self,
        index: usize,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> bool {
        let Some(source) = self.parameter_source(origin, site) else {
            return false;
        };
        let mut pending = vec![(source, index, origin.param_member.clone())];
        let mut seen = FxHashSet::default();
        let mut seeded = false;
        while let Some((source, index, member)) = pending.pop() {
            if !seen.insert((source.clone(), index, member.clone())) {
                continue;
            }
            if seen.len() > 128 || self.escape_unknown || !self.proven_target(&source) {
                return false;
            }
            let Some(callers) = self
                .callers
                .get(&source)
                .filter(|callers| !callers.is_empty())
            else {
                return false;
            };
            for caller in callers {
                let Some(paths) = facts.get_ts_facts(&caller.file).and_then(path_provenance) else {
                    return false;
                };
                let argument = match &member {
                    Some(member) => paths
                        .arg_member_paths
                        .get(&caller.offset)
                        .and_then(|paths| {
                            paths
                                .iter()
                                .find(|(position, key, _)| *position == index && key == member)
                                .map(|(_, _, origin)| origin)
                        }),
                    None => paths.arg_paths.get(&caller.offset).and_then(|paths| {
                        paths
                            .iter()
                            .find(|(position, _)| *position == index)
                            .map(|(_, origin)| origin)
                    }),
                };
                let Some(argument) = argument else {
                    return false;
                };
                if argument.segments.iter().any(|part| part == "..") {
                    return false;
                }
                if classify(Some(argument), fixture_dirs) == Verdict::Allowed {
                    seeded = true;
                } else if let Some(index) = argument.param {
                    let Some(source) = self.parameter_source(argument, caller) else {
                        return false;
                    };
                    pending.push((source, index, argument.param_member.clone()));
                } else {
                    return false;
                }
            }
        }
        // A recursive component needs an independently owned entry argument.
        seeded
    }

    fn parameter_source(&self, origin: &PathOrigin, site: &ResolvedCallSite) -> Option<NodeId> {
        match origin.param_owner {
            Some(offset) => self.nodes.get(&site.file)?.get(&offset).cloned(),
            None => Some(site_source_node(site)),
        }
    }
}
