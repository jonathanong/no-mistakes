pub(super) struct CallAdmission {
    // (Some(owner), None) blocks every group of that file-local owner.
    pub(super) blocked: FxHashSet<(Option<CallableId>, Option<usize>)>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) checks: usize,
    #[cfg(feature = "test-instrumentation")]
    pub(super) steps: usize,
    #[cfg(feature = "test-instrumentation")]
    pub(super) region_searches: usize,
}

impl CallAdmission {
    pub(super) fn blocks(&self, owner: Option<CallableId>, region: Option<usize>) -> bool {
        self.blocked.contains(&(owner, region))
            || owner.is_some() && self.blocked.contains(&(owner, None))
    }
}
