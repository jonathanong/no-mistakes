fn typed_mock_import_sites(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> FxHashSet<(&str, u32)> {
    let (Some(module), Some(source)) = (&facts.module_bindings, facts.source.as_deref()) else {
        return fx_set();
    };
    if module.mock_import_spans.is_empty() {
        return fx_set();
    }
    let starts: FxHashSet<_> = module
        .mock_import_spans
        .iter()
        .map(|span| span.start)
        .collect();
    let mut sites = fx_map();
    for load in module
        .loads
        .iter()
        .filter(|load| load.kind == "dynamicImport")
    {
        let line =
            crate::codebase::ts_source::byte_offset_to_line(source, load.span.start as usize);
        // Imports do not carry byte offsets; retain unknown when a same-line,
        // same-specifier runtime occurrence coexists with a mock type carrier.
        *sites.entry((load.specifier.as_str(), line)).or_insert(true) &=
            starts.contains(&load.span.start);
    }
    sites
        .into_iter()
        .filter_map(|(site, mock_only)| mock_only.then_some(site))
        .collect()
}
