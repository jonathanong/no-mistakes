/// Resolves a direct runtime import binding (including one static namespace
/// member) to a callable in a visible local module.
fn resolve_imported_call_target(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    file: &CallableFileIndex,
    callee: &str,
    indexes: &CallableResolutionIndexes,
) -> Option<ResolvedCallTarget> {
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
                matches!(
                    binding.kind,
                    crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
                        | crate::codebase::dependencies::extract::ImportedBindingKind::Named
                )
            }
            None => {
                binding.kind
                    != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
            }
        })?;
    let export = match (binding.kind, requested_export) {
        (crate::codebase::dependencies::extract::ImportedBindingKind::Namespace, Some(member)) => {
            member.to_string()
        }
        // A named import normally is a value, not a namespace object. Preserve
        // its requested member here so export resolution can accept it only
        // when the barrel concretely exported `* as thatName`.
        (crate::codebase::dependencies::extract::ImportedBindingKind::Named, Some(member)) => {
            format!("{}.{member}", binding.imported)
        }
        (_, None) => binding.imported.clone(),
        _ => return Some(ResolvedCallTarget::Unknown),
    };
    let direct_target = || {
        module_export_target(file, callee, None).expect("callee came from an imported binding")
    };
    let Some(target_path) = resolver.resolve(&binding.specifier, path) else {
        return Some(direct_target());
    };
    let Some(target_path) = edge_inputs.graph_files.visible_path(&target_path) else {
        return Some(direct_target());
    };
    Some(match resolve_exported_callable(
        edge_inputs,
        facts,
        resolver,
        target_path,
        &export,
        indexes,
        &mut Vec::new(),
    ) {
        ExportedCallableResolution::Callable(target_file, scope) => {
            module_export_target(file, callee, Some((target_file, scope)))
                .expect("callee came from an imported binding")
        }
        ExportedCallableResolution::ExternalModuleExport(specifier, export_path) => {
            ResolvedCallTarget::ModuleExport {
                specifier,
                export_path,
                repository_target: None,
            }
        }
        ExportedCallableResolution::Absent | ExportedCallableResolution::Unknown => {
            ResolvedCallTarget::Unknown
        }
    })
}
