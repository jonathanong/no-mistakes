fn project_call_module_files(
    inputs: &GraphEdgeBuildInputs<'_>,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    file: &crate::codebase::ts_source::facts::TsFileFacts,
) -> Vec<(String, std::path::PathBuf)> {
    let imports = file
        .imported_bindings
        .iter()
        .filter(|binding| !binding.is_type_only)
        .map(|binding| binding.specifier.as_str());
    let reexports = file
        .imports
        .iter()
        .filter(|import| {
            import.re_export
                && !import.computed
                && import.kind == crate::codebase::dependencies::extract::ImportKind::Static
        })
        .map(|import| import.specifier.as_str());
    let mut seen = fx_set();
    imports
        .chain(reexports)
        .filter(|specifier| seen.insert(*specifier))
        .filter_map(|specifier| {
            let target = resolver.resolve(specifier, path)?;
            let target = inputs.graph_files.visible_path(&target)?;
            Some((specifier.to_string(), target.to_path_buf()))
        })
        .collect()
}
