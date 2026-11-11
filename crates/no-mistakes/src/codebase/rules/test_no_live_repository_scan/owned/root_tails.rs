use super::parameters::{argument_path, argument_present, unproven_argument};
use super::{path_provenance, Owners};
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};

impl Owners<'_> {
    pub(super) fn safe_suffix(
        &self,
        tail: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        depth: usize,
    ) -> bool {
        if depth >= 16 || tail.segments.iter().any(|part| part == "..") {
            return false;
        }
        if tail.anchor == PathAnchor::Relative && !tail.dynamic {
            return true;
        }
        let Some(index) = tail.param else {
            return false;
        };
        if tail.param_member.is_some() {
            return false;
        }
        let Some(source) = self.parameter_source(tail, site) else {
            return false;
        };
        if !self.parameter_proven(&source) || self.closure_unknown(&site.file) {
            return false;
        }
        let Some(callers) = self.callers.get(&source).filter(|calls| !calls.is_empty()) else {
            return false;
        };
        callers.iter().all(|caller| {
            let Some(paths) = facts.get_ts_facts(&caller.file).and_then(path_provenance) else {
                return false;
            };
            argument_path(paths, caller.offset, index, None)
                .or_else(|| unproven_argument(paths, caller.offset, index))
                .or_else(|| {
                    if argument_present(paths, caller.offset, index) {
                        return None;
                    }
                    facts
                        .get_ts_facts(&site.file)
                        .and_then(path_provenance)
                        .and_then(|declared| declared.parameter_defaults.get(&tail.param_owner?))
                        .and_then(|defaults| defaults.get(&index))
                })
                .is_some_and(|argument| self.safe_suffix(argument, caller, facts, depth + 1))
        })
    }
}
