fn resolve_exported_namespace_reexport_member(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    export: (&CallableFileIndex, &str, &str),
    indexes: &CallableResolutionIndexes,
    visited: &mut Vec<(std::path::PathBuf, String)>,
) -> ExportedCallableResolution {
    let (file, namespace, member) = export;
    if member.contains('.') {
        return ExportedCallableResolution::Absent;
    }
    let Some(binding) = file.exported.get(namespace).filter(|binding| {
        binding.local == "*" && binding.specifier.is_some()
    }) else {
        return ExportedCallableResolution::Absent;
    };
    let specifier = binding.specifier.as_ref().expect("checked above");
    let visible_target = resolver
        .resolve(specifier, path)
        .and_then(|target_path| edge_inputs.graph_files.visible_path(&target_path));
    if let Some(target_path) = visible_target {
        return resolve_exported_callable(
            edge_inputs,
            facts,
            resolver,
            target_path,
            member,
            indexes,
            visited,
        );
    }
    if external_module_specifier(specifier) {
        ExportedCallableResolution::ExternalModuleExport(specifier.clone(), member.to_string())
    } else {
        ExportedCallableResolution::Unknown
    }
}

fn resolve_exported_namespace_member_alias(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    alias: (&CallableFileIndex, &str),
    indexes: &CallableResolutionIndexes,
    visited: &mut Vec<(std::path::PathBuf, String)>,
) -> Option<ExportedCallableResolution> {
    let (file, local) = alias;
    let (namespace, export) = local.split_once('.')?;
    if export.contains('.') {
        return None;
    }
    let imported = file.imported.get(namespace).filter(|imported| {
        imported.kind == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
    })?;
    let visible_target = resolver
        .resolve(&imported.specifier, path)
        .and_then(|target_path| edge_inputs.graph_files.visible_path(&target_path));
    if let Some(target_path) = visible_target {
        return Some(resolve_exported_callable(
            edge_inputs,
            facts,
            resolver,
            target_path,
            export,
            indexes,
            visited,
        ));
    }
    Some(if external_module_specifier(&imported.specifier) {
        ExportedCallableResolution::ExternalModuleExport(
            imported.specifier.clone(),
            export.to_string(),
        )
    } else {
        ExportedCallableResolution::Unknown
    })
}

fn exported_local_callable(
    file: &CallableFileIndex,
    path: &std::path::Path,
    local: String,
    resolved: Option<&ResolvedLocalCallee>,
) -> ExportedCallableResolution {
    let callable_id = resolved
        .and_then(|resolved| resolved.callable_id)
        .or_else(|| file.resolve_local_callable_id(Some(0), &local));
    ExportedCallableResolution::Callable(path.to_path_buf(), local, callable_id)
}
