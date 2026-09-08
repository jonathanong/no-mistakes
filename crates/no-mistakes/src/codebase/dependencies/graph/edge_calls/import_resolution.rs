/// Resolves a direct runtime import binding (including one static namespace
/// member) to a callable in a visible local module.
fn resolve_imported_call_scope(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    file: &CallableFileIndex,
    callee: &str,
    indexes: &CallableResolutionIndexes,
) -> Option<(std::path::PathBuf, String)> {
    let (local, requested_export) = match callee.split_once('.') {
        Some((local, member)) if !member.contains('.') => (local, Some(member)),
        Some(_) => return None,
        None => (callee, None),
    };
    let binding = file
        .imported
        .get(local)
        .filter(|binding| match requested_export {
            Some(_) => {
                binding.kind
                    == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
            }
            None => {
                binding.kind
                    != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
            }
        })?;
    let target_path = resolver.resolve(&binding.specifier, path)?;
    let target_path = edge_inputs.graph_files.visible_path(&target_path)?;
    let export = requested_export.unwrap_or(&binding.imported);
    resolve_exported_callable(
        edge_inputs,
        facts,
        resolver,
        target_path,
        export,
        indexes,
        &mut Vec::new(),
    )
    .callable()
}
