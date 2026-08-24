fn collect_swift_import_edges(
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for file in facts.files.values() {
        let allowed_roots = swift_allowed_roots(facts, file);
        for import in &file.imports {
            if let Some(target_files) = facts.files_by_target.get(import) {
                let target_files = target_files
                    .iter()
                    .filter(|target| allowed_roots.iter().any(|root| target.starts_with(root)))
                    .cloned()
                    .collect();
                push_swift_file_edges(
                    edges,
                    &file.path,
                    &target_files,
                    EdgeKind::SwiftImport,
                    interner,
                );
            }
        }
    }
}

fn collect_swift_reference_edges(
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for file in facts.files.values() {
        let allowed_roots = swift_allowed_roots(facts, file);
        for reference in &file.references {
            if let Some(target_files) = facts.declarations.get(reference) {
                let target_files = target_files
                    .iter()
                    .filter(|target| allowed_roots.iter().any(|root| target.starts_with(root)))
                    .cloned()
                    .collect();
                push_swift_file_edges(
                    edges,
                    &file.path,
                    &target_files,
                    EdgeKind::SwiftReference,
                    interner,
                );
            }
        }
    }
}

fn swift_allowed_roots(
    facts: &crate::codebase::swift::SwiftFactMap,
    file: &crate::codebase::swift::SwiftFileFacts,
) -> Vec<PathBuf> {
    let Some(package) = swift_owning_package(facts, &file.path) else {
        return Vec::new();
    };
    file.target
        .as_ref()
        .and_then(|target| package.targets.get(target))
        .map(|target| swift_dependency_roots(facts, package, target))
        // A Swift file can be in a custom or unsupported target layout. Keep
        // imports within its package causal instead of dropping every edge.
        .unwrap_or_else(|| vec![package.package_root.clone()])
}
