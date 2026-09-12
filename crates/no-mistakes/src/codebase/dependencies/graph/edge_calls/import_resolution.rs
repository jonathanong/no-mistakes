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
            Some(_) => true,
            None => {
                binding.kind
                    != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
            }
        })?;
    let export = match (binding.kind, requested_export) {
        (crate::codebase::dependencies::extract::ImportedBindingKind::Namespace, Some(member)) => {
            member.to_string()
        }
        (crate::codebase::dependencies::extract::ImportedBindingKind::Named, Some(member)) => {
            format!("{}.{member}", binding.imported)
        }
        (crate::codebase::dependencies::extract::ImportedBindingKind::Default, Some(member)) => {
            format!("default.{member}")
        }
        (_, None) => binding.imported.clone(),
    };
    let direct_target = || {
        module_export_target(file, callee, None, None).expect("callee came from an imported binding")
    };
    let Some(target_path) = resolver.resolve(&binding.specifier, path) else {
        return Some(direct_target());
    };
    let Some(target_path) = edge_inputs.graph_files.visible_path(&target_path) else {
        return Some(direct_target());
    };
    if let Some(member) = requested_export {
        if binding.kind != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace {
            if let Some(resolved) = resolve_imported_class_static_member(
                edge_inputs,
                facts,
                resolver,
                file,
                (callee, target_path, &binding.imported, member),
                indexes,
            ) {
                return Some(resolved);
            }
        }
    }
    Some(match resolve_exported_callable(
        edge_inputs,
        facts,
        resolver,
        target_path,
        &export,
        indexes,
        &mut Vec::new(),
    ) {
        ExportedCallableResolution::Callable(target_file, scope, callable_id) => {
            module_export_target(file, callee, Some((target_file, scope)), callable_id)
                .expect("callee came from an imported binding")
        }
        ExportedCallableResolution::ExternalModuleExport(specifier, export_path) => {
            ResolvedCallTarget::ModuleExport {
                specifier,
                export_path,
                repository_target: None,
                callable_id: None,
            }
        }
        ExportedCallableResolution::Absent | ExportedCallableResolution::Unknown => {
            ResolvedCallTarget::Unknown
        }
    })
}

fn resolve_imported_class_static_member(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    file: &CallableFileIndex,
    call: (&str, &std::path::Path, &str, &str),
    indexes: &CallableResolutionIndexes,
) -> Option<ResolvedCallTarget> {
    let (callee, target_path, base_export, member) = call;
    let ExportedCallableResolution::Callable(target_file, _, Some(class_id)) =
        resolve_exported_callable(
            edge_inputs,
            facts,
            resolver,
            target_path,
            base_export,
            indexes,
            &mut Vec::new(),
        )
    else {
        return None;
    };
    let target_index = indexes.file(facts, &target_file)?;
    let (binding_scope, class) = target_index
        .class_bindings
        .iter()
        .find(|(_, class)| class.class_id == class_id)?;
    let (owner, member_id) = target_index.resolve_static_member(
        binding_scope.0,
        class,
        member,
        crate::codebase::dependencies::extract::InvocationKind::Call,
    )?;
    Some(
        module_export_target(
            file,
            callee,
            Some((target_file, format!("{}/{}", owner.scope, member))),
            Some(member_id),
        )
        .expect("callee came from an imported binding"),
    )
}
