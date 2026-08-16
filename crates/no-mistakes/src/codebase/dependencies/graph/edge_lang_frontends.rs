use crate::codebase::lang_frontends::{
    collect_all_lang_facts, scan_kafka_file, topic_identity, LangFactMap, LangFrontendConfig,
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
    emit_lang_edges(&facts.rust, EdgeKind::RustUse, EdgeKind::RustMod, &mut edges);
    emit_lang_edges(&facts.ruby, EdgeKind::RubyRequire, EdgeKind::RubyReference, &mut edges);
    emit_lang_edges(&facts.php, EdgeKind::PhpUse, EdgeKind::PhpPackage, &mut edges);
    emit_queue_edges(&facts.python, options.queue_cluster.as_deref(), &mut edges);
    emit_queue_edges(&facts.go, options.queue_cluster.as_deref(), &mut edges);
    emit_queue_edges(&facts.ruby, options.queue_cluster.as_deref(), &mut edges);
    emit_queue_edges(&facts.php, options.queue_cluster.as_deref(), &mut edges);
    emit_route_edges(&facts.python, &mut edges);
    emit_route_edges(&facts.ruby, &mut edges);
    emit_route_edges(&facts.php, &mut edges);
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
                        file.package.is_none()
                            || facts.files.get(*target).and_then(|other| other.package.as_ref())
                                == file.package.as_ref()
                    })
                    .cloned()
                    .collect();
                push_file_edges(edges, &file.path, &scoped, ref_kind);
            }
        }
    }
}

fn emit_queue_edges(facts: &LangFactMap, cluster: Option<&str>, edges: &mut Vec<Edge>) {
    let mut workers: std::collections::HashMap<String, std::collections::BTreeSet<PathBuf>> =
        std::collections::HashMap::new();
    for file in facts.files.values() {
        for job in &file.queue_workers {
            workers
                .entry(topic_identity(cluster, job))
                .or_default()
                .insert(file.path.clone());
        }
    }
    for file in facts.files.values() {
        for job in &file.queue_enqueues {
            let identity = topic_identity(cluster, job);
            let node = NodeId::QueueJob {
                queue_file: file.path.clone(),
                job: identity.clone(),
            };
            edges.push((
                NodeId::File(file.path.clone()),
                node.clone(),
                EdgeKind::QueueEnqueue,
            ));
            if let Some(targets) = workers.get(&identity) {
                for worker in targets {
                    edges.push((
                        node.clone(),
                        NodeId::File(worker.clone()),
                        EdgeKind::QueueWorker,
                    ));
                }
            }
        }
    }
}

fn push_file_edges(
    edges: &mut Vec<Edge>,
    source: &Path,
    targets: &std::collections::BTreeSet<PathBuf>,
    kind: EdgeKind,
) {
    for target in targets {
        if target != source {
            edges.push((
                NodeId::File(source.to_path_buf()),
                NodeId::File(target.clone()),
                kind,
            ));
        }
    }
}
