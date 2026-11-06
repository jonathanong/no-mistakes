use super::*;
use crate::codebase::dependencies::extract::PerItemCall;
use crate::config::v2::schema::EffectKindConfig;

pub(super) struct CandidatePolicy<'a> {
    pub options: &'a Options,
    pub effect: &'a EffectKindConfig,
    pub filter: &'a super::super::path_filter::RulePathFilter,
    pub excluded: &'a [NodeId],
}

impl CandidatePolicy<'_> {
    pub(super) fn includes(&self, site: &ResolvedCallSite, position: &PerItemCall) -> bool {
        if (self.options.exempt_capped_cursor_page_reads && position.capped_cursor_page_read)
            || !self.filter.is_match(&site.file)
            || exempt(self.effect, &site.source_callee)
            || self.excluded.binary_search(&source_node(site)).is_ok()
            || (site.targets().len() > 0
                && site.targets().all(|candidate| {
                    candidate
                        .node
                        .is_some_and(|node| self.excluded.binary_search(node).is_ok())
                }))
        {
            return false;
        }
        true
    }
}
