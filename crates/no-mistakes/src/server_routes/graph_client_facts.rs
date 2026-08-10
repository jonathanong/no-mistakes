/// Borrow the request's collected facts for client-call sources without
/// applying the server-route-root glob that selects route definitions.
fn client_facts_from_prepared(
    prepared: &PreparedServerAnalysis,
    filter: Option<&GlobSet>,
    test_filter: Option<&crate::codebase::test_filter::TestFileFilter>,
) -> crate::codebase::ts_source::facts::TsFactMap {
    let root = &prepared.root;
    crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
        prepared
            .source_files
            .iter()
            .filter(|path| !test_filter.is_some_and(|filter| filter.is_match(root, path)))
            .filter(|path| {
                filter
                    .is_none_or(|filter| filter.is_match(path.strip_prefix(root).unwrap_or(path)))
            })
            .filter_map(|path| {
                prepared
                    .facts
                    .get(path)
                    .cloned()
                    .map(|facts| (path.clone(), facts))
            }),
        prepared.facts.plan(),
    )
}
