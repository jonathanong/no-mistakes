impl DepGraph {
    pub(crate) fn assertion_call_sites(&self) -> &[ResolvedCallSite] {
        &self.assertion_call_sites
    }
}
