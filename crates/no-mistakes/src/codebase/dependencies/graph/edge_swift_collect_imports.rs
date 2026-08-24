fn collect_swift_import_edges(
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for file in facts.files.values() {
        let source_package = swift_owning_package(facts, &file.path);
        for import in &file.imports {
            if let Some(target_files) = facts.files_by_target.get(import) {
                let allowed_roots = source_package
                    .and_then(|package| {
                        file.target
                            .as_ref()
                            .and_then(|target| package.targets.get(target))
                            .map(|target| swift_dependency_roots(facts, package, target))
                    })
                    .unwrap_or_default();
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
        let source_package = swift_owning_package(facts, &file.path);
        let allowed_roots = source_package
            .and_then(|package| {
                file.target
                    .as_ref()
                    .and_then(|target| package.targets.get(target))
                    .map(|target| swift_dependency_roots(facts, package, target))
            })
            .unwrap_or_default();
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
