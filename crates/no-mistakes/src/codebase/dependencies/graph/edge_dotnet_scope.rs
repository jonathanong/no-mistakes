fn dotnet_project_scope(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    source_project: &Path,
) -> std::collections::BTreeSet<PathBuf> {
    let mut allowed = std::collections::BTreeSet::from([source_project.to_path_buf()]);
    let mut pending = vec![source_project.to_path_buf()];
    while let Some(project) = pending.pop() {
        let Some(project_facts) = facts.projects.get(&project) else {
            continue;
        };
        for reference in &project_facts.project_references {
            if allowed.insert(reference.clone()) {
                pending.push(reference.clone());
            }
        }
    }
    allowed
}

fn scoped_dotnet_target_files(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    targets: &std::collections::BTreeSet<PathBuf>,
    allowed_projects: &std::collections::BTreeSet<PathBuf>,
) -> std::collections::BTreeSet<PathBuf> {
    targets
        .iter()
        .filter(|target| {
            facts
                .files
                .get(*target)
                .and_then(|file| file.project.as_ref())
                .is_some_and(|project| allowed_projects.contains(project))
        })
        .cloned()
        .collect()
}
