fn lang_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends")
            .join(name),
    )
}

fn lang_files(root: &Path) -> Vec<PathBuf> {
    let repo = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    );
    crate::codebase::ts_source::discover_visible_paths(&repo)
        .into_iter()
        .map(|path| {
            let absolute = if path.is_absolute() {
                path
            } else {
                repo.join(path)
            };
            crate::codebase::ts_resolver::normalize_path(&absolute)
        })
        .filter(|path| path.starts_with(root))
        .collect()
}

fn lang_options() -> GraphConfigOptions {
    GraphConfigOptions {
        python_packages: vec!["app".into()],
        go_modules: vec![".".into(), "worker".into()],
        rust_packages: vec![".".into(), "src".into()],
        rails_apps: vec![".".into()],
        php_apps: vec![".".into()],
        php_framework: Some("laravel".into()),
        queue_enqueues: vec!["**/*".into()],
        queue_workers: vec!["**/*".into()],
        queue_cluster: Some("orders".into()),
        ..GraphConfigOptions::default()
    }
}


fn src(root: &std::path::Path) -> std::sync::Arc<crate::codebase::ts_source::SourceStore> {
    crate::codebase::rules::source_store_for_files(&lang_files(root))
}

fn src_files(files: &[PathBuf]) -> std::sync::Arc<crate::codebase::ts_source::SourceStore> {
    crate::codebase::rules::source_store_for_files(files)
}

#[test]
fn language_frontend_edges_cover_configured_extractors() {
    let options = lang_options();
    let python = lang_fixture("python-celery-django");
    let python_edges = collect_language_frontend_edges(&python, &lang_files(&python), Some(&options), &src(&python));
    assert!(python_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::PythonImport));
    assert!(python_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::QueueEnqueue));
    assert!(python_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from.as_file().is_some_and(|path| path.ends_with("urls.py"))
            && to.as_file().is_some_and(|path| path.ends_with("views.py"))
    }));

    let go = lang_fixture("go-asynq");
    let go_edges = collect_language_frontend_edges(&go, &lang_files(&go), Some(&options), &src(&go));
    assert!(go_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::QueueWorker));

    let rust = lang_fixture("rust-mods");
    let rust_edges = collect_language_frontend_edges(&rust, &lang_files(&rust), Some(&options), &src(&rust));
    assert!(rust_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::RustUse || *kind == EdgeKind::RustMod));
    assert!(rust_edges.iter().any(|(from, _, kind)| {
        *kind == EdgeKind::RustPackage && from.as_file().is_some_and(|path| path.ends_with("lib.rs"))
    }));
    assert!(rust_edges.iter().all(|(from, _, kind)| {
        *kind != EdgeKind::RustPackage
            || from
                .as_file()
                .is_none_or(|path| !path.ends_with("aaa/mod.rs"))
    }));

    let rails = lang_fixture("rails-jobs");
    let rails_edges = collect_language_frontend_edges(&rails, &lang_files(&rails), Some(&options), &src(&rails));
    assert!(rails_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::RouteRef));

    let php = lang_fixture("php-laravel");
    let php_edges = collect_language_frontend_edges(&php, &lang_files(&php), Some(&options), &src(&php));
    assert!(php_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::PhpUse || *kind == EdgeKind::PhpPackage));

    let kafka = lang_fixture("kafka-topics");
    let kafka_edges = collect_language_frontend_edges(&kafka, &lang_files(&kafka), Some(&options), &src(&kafka));
    assert!(kafka_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::QueueEnqueue));
    assert!(
        collect_language_frontend_edges(&kafka, &lang_files(&kafka), None, &src(&kafka))
            .is_empty()
    );
}

#[test]
fn language_frontend_edges_skip_empty_config() {
    let root = lang_fixture("python-celery-django");
    let files = lang_files(&root);
    assert!(collect_language_frontend_edges(&root, &files, Some(&GraphConfigOptions::default()), &src_files(&files))
        .is_empty());
}

#[test]
fn language_frontend_edges_cover_kafka_misses_and_empty_queue_globs() {
    let kafka = lang_fixture("kafka-topics");
    let mut files = lang_files(&kafka);
    files.push(kafka.join("missing-producer.ts"));
    let mut options = lang_options();
    options.queue_enqueues = vec!["[".into(), "producer.ts".into()];
    options.queue_workers = vec!["consumer.ts".into()];
    let edges = collect_language_frontend_edges(&kafka, &files, Some(&options), &src_files(&files));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::QueueEnqueue));

    let python = lang_fixture("python-celery-django");
    let no_queues = GraphConfigOptions {
        python_packages: vec!["app".into()],
        ..GraphConfigOptions::default()
    };
    let python_edges =
        collect_language_frontend_edges(&python, &lang_files(&python), Some(&no_queues), &src(&python));
    assert!(python_edges
        .iter()
        .all(|(_, _, kind)| *kind != EdgeKind::QueueEnqueue));

    let rails = lang_fixture("rails-jobs");
    let rails_edges = collect_language_frontend_edges(&rails, &lang_files(&rails), Some(&lang_options()), &src(&rails));
    assert!(rails_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from.as_file().is_some_and(|path| path.ends_with("routes.rb"))
            && to
                .as_file()
                .is_some_and(|path| path.ends_with("admin/users_controller.rb"))
    }));
}

#[test]
fn language_frontend_config_keeps_already_prefixed_queue_globs() {
    let root = lang_fixture("queue-prefix");
    let options = graph_config_options(&root).expect("queue-prefix config");
    assert!(options
        .queue_enqueues
        .iter()
        .any(|glob| glob == "backend/app/**/*.py"));
    assert!(options
        .queue_enqueues
        .iter()
        .any(|glob| glob == "app/application/**/*.py"));
    assert_eq!(
        options.queue_glob_clusters.get("backend/app/**/*.py"),
        Some(&Some("api".into()))
    );
    assert_eq!(
        options.queue_glob_clusters.get("app/application/**/*.py"),
        Some(&Some("other".into()))
    );
    assert_eq!(options.queue_glob_clusters.get("bare/*.py"), Some(&None));
}

#[test]
fn language_frontend_edges_scope_routes_and_go_packages() {
    let python = lang_fixture("python-celery-django");
    let python_edges =
        collect_language_frontend_edges(&python, &lang_files(&python), Some(&lang_options()), &src(&python));
    assert!(python_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from
                .as_file()
                .is_some_and(|path| path.ends_with("app/urls.py") && !path.ends_with("api/urls.py"))
            && to.as_file().is_some_and(|path| path.ends_with("api/urls.py"))
    }));
    assert!(python_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from
                .as_file()
                .is_some_and(|path| path.ends_with("app/urls.py") && !path.ends_with("api/urls.py"))
            && to
                .as_file()
                .is_some_and(|path| path.ends_with("billing/views.py"))
    }));

    let go = lang_fixture("go-asynq");
    let go_edges = collect_language_frontend_edges(&go, &lang_files(&go), Some(&lang_options()), &src(&go));
    assert!(go_edges.iter().all(|(from, to, kind)| {
        *kind != EdgeKind::GoReference
            || !from
                .as_file()
                .is_some_and(|path| path.ends_with("pkg/ping.go"))
            || !to
                .as_file()
                .is_some_and(|path| path.ends_with("mail/user.go"))
    }));
}

#[test]
fn package_root_file_falls_back_when_no_manifest_name() {
    let mut files = std::collections::BTreeSet::new();
    files.insert(PathBuf::from("z.rs"));
    files.insert(PathBuf::from("a.rs"));
    assert_eq!(package_root_file(&files), Some(Path::new("a.rs")));
    assert!(package_root_file(&std::collections::BTreeSet::new()).is_none());
}
