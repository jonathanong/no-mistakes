fn write_output_results(
    format: Format,
    root_strs: &[String],
    result: &TraversalResult,
    out: &mut dyn Write,
) -> Result<()> {
    match format {
        Format::Json if result.projection.is_paths() => output::write_import_closure_json(
            &result.entries,
            &result.root,
            &result.diagnostics,
            out,
        ),
        Format::Json => output::write_json_with_diagnostics(
            root_strs,
            &result.entries,
            &result.root,
            &result.diagnostics,
            &result.tsconfig_provenance,
            out,
        ),
        Format::Yml => output::write_yml_with_diagnostics(
            root_strs,
            &result.entries,
            &result.root,
            &result.diagnostics,
            &result.tsconfig_provenance,
            out,
        ),
        _ => write_entries(format, root_strs, &result.entries, &result.root, out),
    }
}
