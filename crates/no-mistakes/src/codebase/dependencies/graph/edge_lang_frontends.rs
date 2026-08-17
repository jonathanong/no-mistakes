use crate::codebase::lang_frontends::{
    collect_all_lang_facts, scan_kafka_file, topic_identity, LangFactMap, LangFileFacts,
    LangFrontendConfig,
};
use crate::codebase::ts_source::SourceStore;

fn collect_language_frontend_edges(
    root: &Path,
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
    sources: &SourceStore,
) -> Vec<Edge> {
    let Some(options) = config_options else {
        return Vec::new();
    };
    let config = lang_config_from_options(options);
    if config_is_empty(&config)
        && options.queue_enqueues.is_empty()
        && options.queue_workers.is_empty()
    {
        return Vec::new();
    }
    let enqueue_globs = compile_queue_globs(&options.queue_enqueues);
    let worker_globs = compile_queue_globs(&options.queue_workers);
    let mut edges = Vec::new();
    if !config_is_empty(&config) {
        let facts = collect_all_lang_facts(root, all_files, &config, sources);
        emit_lang_edges(
            &facts.python,
            EdgeKind::PythonImport,
            EdgeKind::PythonReference,
            &mut edges,
        );
        emit_lang_edges(
            &facts.go,
            EdgeKind::GoImport,
            EdgeKind::GoReference,
            &mut edges,
        );
        emit_lang_edges(
            &facts.rust,
            EdgeKind::RustUse,
            EdgeKind::RustUse,
            &mut edges,
        );
        emit_mod_edges(&facts.rust, EdgeKind::RustMod, &mut edges);
        emit_package_edges(&facts.rust, EdgeKind::RustPackage, &mut edges);
        emit_lang_edges(
            &facts.ruby,
            EdgeKind::RubyRequire,
            EdgeKind::RubyReference,
            &mut edges,
        );
        emit_lang_edges(&facts.php, EdgeKind::PhpUse, EdgeKind::PhpUse, &mut edges);
        emit_package_edges(&facts.php, EdgeKind::PhpPackage, &mut edges);
        emit_queue_edges(
            root,
            &facts.python,
            &enqueue_globs,
            &worker_globs,
            options,
            &mut edges,
        );
        emit_queue_edges(
            root,
            &facts.go,
            &enqueue_globs,
            &worker_globs,
            options,
            &mut edges,
        );
        emit_queue_edges(
            root,
            &facts.ruby,
            &enqueue_globs,
            &worker_globs,
            options,
            &mut edges,
        );
        emit_queue_edges(
            root,
            &facts.php,
            &enqueue_globs,
            &worker_globs,
            options,
            &mut edges,
        );
        emit_route_edges(root, &facts.python, options, &mut edges);
        emit_route_edges(root, &facts.ruby, options, &mut edges);
        emit_route_edges(root, &facts.php, options, &mut edges);
    }
    if !options.queue_enqueues.is_empty() || !options.queue_workers.is_empty() {
        emit_kafka_edges(
            root,
            all_files,
            &enqueue_globs,
            &worker_globs,
            options,
            sources,
            &mut edges,
        );
    }
    edges
}

fn lang_config_from_options(options: &GraphConfigOptions) -> LangFrontendConfig {
    LangFrontendConfig {
        python_packages: options.python_packages.clone(),
        go_modules: options.go_modules.clone(),
        rust_packages: options.rust_packages.clone(),
        rails_apps: options.rails_apps.clone(),
        php_apps: options.php_apps.clone(),
        php_framework: options.php_framework.clone(),
    }
}

fn config_is_empty(config: &LangFrontendConfig) -> bool {
    config.python_packages.is_empty()
        && config.go_modules.is_empty()
        && config.rust_packages.is_empty()
        && config.rails_apps.is_empty()
        && config.php_apps.is_empty()
}
