struct FileMockDraft {
    boundaries: RootMockBoundaries,
    candidates: FxHashMap<PathBuf, bool>,
    external: FxHashMap<MockedModule, bool>,
}

impl DepGraph {
    fn finish_file_mocks(
        &self,
        path: &Path,
        draft: FileMockDraft,
        facts: &dyn TsFactLookup,
        resolver: &dyn ImportResolution,
        files: &GraphFiles,
    ) -> Option<RootMockBoundaries> {
        let FileMockDraft {
            mut boundaries,
            candidates,
            external,
        } = draft;
        boundaries.blocked.extend(
            candidates
                .into_iter()
                .filter_map(|(target, full)| full.then_some(target)),
        );
        boundaries
            .modules
            .extend(boundaries.blocked.iter().cloned().map(MockedModule::File));
        for (module_id, full) in external {
            if full {
                boundaries.modules.insert(module_id);
            }
        }
        if !boundaries.modules.is_empty() {
            boundaries.suppressed = self.calls_through_mocked_modules(
                path,
                &boundaries.modules,
                facts,
                resolver,
                files,
            );
        }
        boundaries.factories.sort();
        boundaries.factories.dedup();
        (!boundaries.blocked.is_empty()
            || !boundaries.factories.is_empty()
            || !boundaries.modules.is_empty()
            || !boundaries.suppressed.is_empty())
        .then_some(boundaries)
    }
}
