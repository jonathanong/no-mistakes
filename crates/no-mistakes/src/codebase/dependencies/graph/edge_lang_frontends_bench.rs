/// Test-instrumentation wrappers around the production language-frontend
/// collectors. They exist so Criterion can measure the real in-process path
/// without making `GraphConfigOptions` part of the supported API.
pub(crate) struct LanguageFrontendEdgeRequest<'a> {
    pub root: &'a Path,
    pub all_files: &'a [PathBuf],
    pub languages: &'a crate::codebase::lang_frontends::LangFrontendConfig,
    pub queue_enqueues: &'a [String],
    pub queue_workers: &'a [String],
    pub queue_cluster: Option<String>,
}

pub(crate) fn collect_language_frontend_edges_for_bench(
    request: LanguageFrontendEdgeRequest<'_>,
) -> Vec<Edge> {
    let options = GraphConfigOptions {
        python_packages: request.languages.python_packages.clone(),
        go_modules: request.languages.go_modules.clone(),
        rust_packages: request.languages.rust_packages.clone(),
        rails_apps: request.languages.rails_apps.clone(),
        php_apps: request.languages.php_apps.clone(),
        php_framework: request.languages.php_framework.clone(),
        queue_enqueues: request.queue_enqueues.to_vec(),
        queue_workers: request.queue_workers.to_vec(),
        queue_cluster: request.queue_cluster,
        ..GraphConfigOptions::default()
    };
    let interner = PathInterner::new();
    collect_language_frontend_edges(
        request.root,
        request.all_files,
        Some(&options),
        None,
        &interner,
    )
}

pub(crate) fn count_queue_glob_matches(
    root: &Path,
    files: &[PathBuf],
    enqueue_globs: &[String],
    worker_globs: &[String],
) -> usize {
    let options = GraphConfigOptions {
        queue_enqueues: enqueue_globs.to_vec(),
        queue_workers: worker_globs.to_vec(),
        ..GraphConfigOptions::default()
    };
    let enqueue = compile_queue_globs(enqueue_globs);
    let worker = compile_queue_globs(worker_globs);
    files
        .iter()
        .filter(|path| {
            let enqueue = matching_queue_cluster(root, path, &enqueue, &options).is_some();
            let worker = matching_queue_cluster(root, path, &worker, &options).is_some();
            enqueue || worker
        })
        .count()
}
