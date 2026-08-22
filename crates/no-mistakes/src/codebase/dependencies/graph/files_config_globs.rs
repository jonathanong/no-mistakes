fn prefixed_queue_globs_enqueues(project: &crate::config::v2::schema::Project) -> Vec<String> {
    prefix_project_globs(project.root.as_deref(), &project.queues.enqueues)
}

fn prefixed_queue_globs_workers(project: &crate::config::v2::schema::Project) -> Vec<String> {
    prefix_project_globs(project.root.as_deref(), &project.queues.workers)
}

fn flatten_queue_globs(
    v2_config: &crate::config::v2::NoMistakesConfig,
    select: fn(&crate::config::v2::schema::Project) -> Vec<String>,
) -> Vec<String> {
    v2_config.projects.values().flat_map(select).collect()
}

fn flatten_queue_glob_clusters(
    v2_config: &crate::config::v2::NoMistakesConfig,
) -> HashMap<String, Option<String>> {
    let mut clusters = HashMap::new();
    for project in v2_config.projects.values() {
        let cluster = project.queues.cluster.clone();
        for glob in prefixed_queue_globs_enqueues(project)
            .into_iter()
            .chain(prefixed_queue_globs_workers(project))
        {
            clusters.entry(glob).or_insert_with(|| cluster.clone());
        }
    }
    clusters
}

fn flatten_trpc_routers(v2_config: &crate::config::v2::NoMistakesConfig) -> Vec<String> {
    v2_config
        .projects
        .values()
        .flat_map(|project| prefix_project_globs(project.root.as_deref(), &project.trpc.routers))
        .collect()
}

fn prefix_project_globs(root: Option<&str>, globs: &[String]) -> Vec<String> {
    let prefix = root
        .map(str::trim)
        .filter(|root| !root.is_empty() && *root != ".");
    globs
        .iter()
        .map(|glob| match prefix {
            Some(root) if glob_has_root_prefix(glob, root) => glob.clone(),
            Some(root) => format!("{}/{glob}", root.trim_end_matches('/')),
            None => glob.clone(),
        })
        .collect()
}

fn glob_has_root_prefix(glob: &str, root: &str) -> bool {
    let root = root.trim_end_matches('/');
    glob == root || glob.starts_with(&format!("{root}/"))
}
