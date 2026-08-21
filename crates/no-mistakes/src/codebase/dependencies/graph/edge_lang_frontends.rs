use crate::codebase::lang_frontends::{
    collect_all_lang_facts, scan_kafka_file, topic_identity, LangFactMap, LangFileFacts,
    LangFrontendConfig,
};

fn language_frontend_source_store(
    root: &Path,
    all_files: &[PathBuf],
    visible_paths: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
) -> Arc<crate::codebase::ts_source::SourceStore> {
    if let Some(snapshot) = visible_paths {
        return snapshot.source_store_for(root);
    }
    Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(all_files),
    )))
}

fn collect_language_frontend_edges(
    root: &Path,
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
    visible_paths: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
    interner: &PathInterner,
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
    let sources = language_frontend_source_store(root, all_files, visible_paths);
    let mut edges = Vec::new();
    if !config_is_empty(&config) {
        let facts = collect_all_lang_facts(root, all_files, &config, &sources);
        emit_lang_edges(
            &facts.python,
            EdgeKind::PythonImport,
            EdgeKind::PythonReference,
            &mut edges,
            interner,
        );
        emit_lang_edges(
            &facts.go,
            EdgeKind::GoImport,
            EdgeKind::GoReference,
            &mut edges,
            interner,
        );
        emit_lang_edges(
            &facts.rust,
            EdgeKind::RustUse,
            EdgeKind::RustUse,
            &mut edges,
            interner,
        );
        emit_mod_edges(&facts.rust, EdgeKind::RustMod, &mut edges, interner);
        emit_package_edges(&facts.rust, EdgeKind::RustPackage, &mut edges, interner);
        emit_path_dep_package_edges(&facts.rust, EdgeKind::RustPackage, &mut edges, interner);
        emit_lang_edges(
            &facts.ruby,
            EdgeKind::RubyRequire,
            EdgeKind::RubyReference,
            &mut edges,
            interner,
        );
        emit_lang_edges(
            &facts.php,
            EdgeKind::PhpUse,
            EdgeKind::PhpUse,
            &mut edges,
            interner,
        );
        emit_package_edges(&facts.php, EdgeKind::PhpPackage, &mut edges, interner);
        emit_queue_edges(root, &facts.python, options, &mut edges, interner);
        emit_queue_edges(root, &facts.go, options, &mut edges, interner);
        emit_queue_edges(root, &facts.ruby, options, &mut edges, interner);
        emit_queue_edges(root, &facts.php, options, &mut edges, interner);
        emit_route_edges(root, &facts.python, options, &mut edges, interner);
        emit_route_edges(root, &facts.go, options, &mut edges, interner);
        emit_route_edges(root, &facts.rust, options, &mut edges, interner);
        emit_route_edges(root, &facts.ruby, options, &mut edges, interner);
        emit_route_edges(root, &facts.php, options, &mut edges, interner);
    }
    if !options.queue_enqueues.is_empty() || !options.queue_workers.is_empty() {
        emit_kafka_edges(root, all_files, options, &sources, &mut edges, interner);
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

