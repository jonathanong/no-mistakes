use crate::codebase::lang_frontends::{
    collect_all_lang_facts, scan_kafka_file, topic_identity, LangFactMap, LangFileFacts,
    LangFrontendConfig,
};

fn collect_language_frontend_edges(
    root: &Path,
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
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
    let facts = collect_all_lang_facts(root, all_files, &config);
    let mut edges = Vec::new();
    emit_lang_edges(&facts.python, EdgeKind::PythonImport, EdgeKind::PythonReference, &mut edges);
    emit_lang_edges(&facts.go, EdgeKind::GoImport, EdgeKind::GoReference, &mut edges);
    emit_lang_edges(&facts.rust, EdgeKind::RustUse, EdgeKind::RustUse, &mut edges);
    emit_mod_edges(&facts.rust, EdgeKind::RustMod, &mut edges);
    emit_package_edges(&facts.rust, EdgeKind::RustPackage, &mut edges);
    emit_lang_edges(&facts.ruby, EdgeKind::RubyRequire, EdgeKind::RubyReference, &mut edges);
    emit_lang_edges(&facts.php, EdgeKind::PhpUse, EdgeKind::PhpUse, &mut edges);
    emit_package_edges(&facts.php, EdgeKind::PhpPackage, &mut edges);
    emit_queue_edges(root, &facts.python, options, &mut edges);
    emit_queue_edges(root, &facts.go, options, &mut edges);
    emit_queue_edges(root, &facts.ruby, options, &mut edges);
    emit_queue_edges(root, &facts.php, options, &mut edges);
    emit_route_edges(root, &facts.python, options, &mut edges);
    emit_route_edges(root, &facts.ruby, options, &mut edges);
    emit_route_edges(root, &facts.php, options, &mut edges);
    emit_kafka_edges(root, all_files, options, &mut edges);
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

fn emit_lang_edges(
    facts: &LangFactMap,
    import_kind: EdgeKind,
    ref_kind: EdgeKind,
    edges: &mut Vec<Edge>,
) {
    for file in facts.files.values() {
        for import in &file.imports {
            if let Some(targets) = facts.files_by_module.get(import) {
                push_file_edges(edges, &file.path, targets, import_kind);
            }
        }
        for reference in &file.references {
            if let Some(targets) = facts.declarations.get(reference) {
                let scoped: std::collections::BTreeSet<_> = targets
                    .iter()
                    .filter(|target| {
                        facts
                            .files
                            .get(*target)
                            .is_some_and(|other| reference_target_allowed(file, other, reference))
                    })
                    .cloned()
                    .collect();
                push_file_edges(edges, &file.path, &scoped, ref_kind);
            }
        }
    }
}

fn emit_mod_edges(facts: &LangFactMap, kind: EdgeKind, edges: &mut Vec<Edge>) {
    for file in facts.files.values() {
        for name in &file.mods {
            let qualified = match file.module.as_deref() {
                Some(parent) => format!("{parent}.{name}"),
                None => name.clone(),
            };
            let targets = facts
                .files_by_module
                .get(&qualified)
                .or_else(|| facts.files_by_module.get(name));
            if let Some(targets) = targets {
                push_file_edges(edges, &file.path, targets, kind);
            }
        }
    }
}

fn emit_package_edges(facts: &LangFactMap, kind: EdgeKind, edges: &mut Vec<Edge>) {
    for files in facts.files_by_package.values() {
        let Some(root) = package_root_file(files) else {
            continue;
        };
        push_file_edges(edges, root, files, kind);
    }
}

fn package_root_file(files: &std::collections::BTreeSet<PathBuf>) -> Option<&Path> {
    let named = |want: &str| {
        files
            .iter()
            .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(want))
    };
    named("lib.rs")
        .or_else(|| named("main.rs"))
        .or_else(|| named("composer.json"))
        .or_else(|| named("mod.rs"))
        .or_else(|| files.iter().next())
        .map(PathBuf::as_path)
}

fn push_file_edges(
    edges: &mut Vec<Edge>,
    source: &Path,
    targets: &std::collections::BTreeSet<PathBuf>,
    kind: EdgeKind,
) {
    for target in targets {
        if target != source {
            edges.push((NodeId::file(source), NodeId::file(target), kind));
        }
    }
}
