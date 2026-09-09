fn index_sorted_call_sites_by_file(
    sites: &[ResolvedCallSite],
) -> FxHashMap<std::path::PathBuf, std::ops::Range<usize>> {
    let mut index = fx_map();
    let mut start = 0;
    while start < sites.len() {
        let file = &sites[start].file;
        let mut end = start + 1;
        while end < sites.len() && sites[end].file == *file {
            end += 1;
        }
        index.insert(file.clone(), start..end);
        start = end;
    }
    index
}

impl CallableFileIndex {
    fn unique_scope_id(
        &self,
        scope: &str,
    ) -> Option<crate::codebase::dependencies::extract::CallableId> {
        let ids = self.scope_ids_by_display.get(scope)?;
        (ids.len() == 1).then_some(ids[0])
    }
}

impl DepGraph {
    /// Call sites in `file`, or an empty slice when the file has none.
    pub fn call_sites_in_file(&self, file: &std::path::Path) -> &[ResolvedCallSite] {
        self.call_sites_by_file
            .get(file)
            .map(|range| &self.resolved_call_sites[range.clone()])
            .unwrap_or(&[])
    }
}
