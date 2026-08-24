fn collect_dotnet_project_edges(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for project in facts.projects.values() {
        // ProjectReference is a project-level relationship, not an inferred
        // source-symbol relationship. Keep it even when the project has no
        // parseable source files so reverse impact traversal reaches dependents.
        for reference in &project.project_references {
            edges.push((
                NodeId::file_in(interner, &project.project_path),
                NodeId::file_in(interner, reference),
                EdgeKind::DotnetProjectDependency,
            ));
        }

        let Some(source_files) = facts.files_by_project.get(&project.project_path) else {
            continue;
        };
        let test_files = source_files.iter().filter(|path| {
            facts
                .files
                .get(*path)
                .is_some_and(|file| file.has_xunit_tests)
        });
        for reference in &project.project_references {
            if let Some(target_files) = facts.files_by_project.get(reference) {
                for source in test_files.clone() {
                    push_dotnet_file_edges(
                        edges,
                        source,
                        target_files,
                        EdgeKind::DotnetProjectDependency,
                        interner,
                    );
                }
            }
        }
    }
}

fn push_dotnet_file_edges(
    edges: &mut Vec<Edge>,
    source: &Path,
    target_files: &std::collections::BTreeSet<PathBuf>,
    kind: EdgeKind,
    interner: &PathInterner,
) {
    for target in target_files {
        if target != source {
            edges.push((
                NodeId::file_in(interner, source),
                NodeId::file_in(interner, target),
                kind,
            ));
        }
    }
}
