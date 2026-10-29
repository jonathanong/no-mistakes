/// A relative specifier that names an existing non-code file imports no JS/TS
/// export. File discovery skips directories such as `build/`, so the file may
/// be absent from the graph universe while still being present on disk. A
/// missing file, a directory, a query suffix, or a code file stays unknown.
fn existing_asset(inputs: &ExportEdgeInputs<'_>, source: &str) -> bool {
    (source.starts_with("./") || source.starts_with("../"))
        && inputs.path.parent().is_some_and(|directory| {
            let candidate = crate::codebase::ts_resolver::normalize_path(&directory.join(source));
            !is_indexable(&candidate) && candidate.is_file()
        })
}

/// Require loads made through a `createRequire(import.meta.url)` binding. The
/// ordinary extractor only sees the global `require`, so these loads would
/// otherwise be invisible to the selector projection.
fn collect_created_require_selectors(
    inputs: &ExportEdgeInputs<'_>,
    file_facts: &crate::codebase::ts_source::facts::TsFileFacts,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
) {
    let (Some(module), Some(source)) = (&file_facts.module_bindings, file_facts.source.as_deref())
    else {
        return;
    };
    let line_of = |start: u32| crate::codebase::ts_source::byte_offset_to_line(source, start as usize);
    let known: FxHashSet<_> = file_facts
        .imports
        .iter()
        .filter(|import| import.kind == ImportKind::Require)
        .map(|import| (import.specifier.as_str(), import.line))
        .collect();
    for load in module.loads.iter().filter(|load| load.kind == "require") {
        let line = line_of(load.span.start);
        if !known.contains(&(load.specifier.as_str(), line)) {
            selector_unknown(
                inputs,
                &load.specifier,
                line as usize,
                "dynamic or require selector",
                edges,
                sites,
            );
        }
    }
}
