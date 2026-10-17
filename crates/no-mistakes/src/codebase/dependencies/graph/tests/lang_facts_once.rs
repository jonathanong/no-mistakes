fn lang_facts_once_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends"),
    )
}

fn lang_facts_once_config() -> crate::config::v2::NoMistakesConfig {
    serde_yaml::from_str(
        r#"
tests:
  python:
    packages:
      - python-celery-django/app
      - python-flask-fastapi
  go:
    modules:
      - go-asynq
      - go-asynq/worker
      - go-http
  rust:
    packages:
      - rust-mods
      - rust-mods/src
      - rust-http
  rails:
    apps:
      - rails-jobs
  php:
    framework: laravel
    apps:
      - php-laravel
  java:
    packages:
      - java-spring
  kotlin:
    packages:
      - kotlin-spring
  elixir:
    apps:
      - phoenix-routes
projects:
  api:
    type: server
    queues:
      enqueues:
        - "**/*"
      workers:
        - "**/*"
      cluster: orders
"#,
    )
    .expect("lang frontend fixture config")
}

fn lang_facts_once_options(config: &crate::config::v2::NoMistakesConfig) -> GraphConfigOptions {
    let tests = &config.tests;
    GraphConfigOptions {
        python_packages: tests.python.packages.clone(),
        go_modules: tests.go.modules.clone(),
        rust_packages: tests.rust.packages.clone(),
        rails_apps: tests.rails.apps.clone(),
        php_apps: tests.php.apps.clone(),
        php_framework: tests.php.framework.clone(),
        java_packages: tests.java.packages.clone(),
        kotlin_packages: tests.kotlin.packages.clone(),
        elixir_apps: tests.elixir.apps.clone(),
        dart_packages: tests.dart.packages.clone(),
        queue_enqueues: vec!["**/*".into()],
        queue_workers: vec!["**/*".into()],
        queue_cluster: Some("orders".into()),
        ..GraphConfigOptions::default()
    }
}

fn lang_facts_once_extractions(observer: &crate::diagnostics::InvocationObserver) -> u64 {
    observer
        .snapshot()
        .work
        .get("lang_frontends.file_extractions")
        .copied()
        .unwrap_or(0)
}

fn lang_facts_once_scanned(facts: &crate::codebase::lang_frontends::CollectedLangFacts) -> u64 {
    crate::codebase::lang_frontends::each_lang_map(facts)
        .iter()
        .map(|map| map.scanned_files)
        .sum::<usize>() as u64
}

#[test]
fn lang_facts_once_queue_routes_and_graph_extract_each_file_once() {
    let root = lang_facts_once_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let config = lang_facts_once_config();
    let dataset = session.dataset(&root);
    let files = dataset.paths_for(&root);
    let sources = dataset.sources_for(&root);
    assert!(files.len() > 1, "fixture must contain multiple files");

    crate::queue::language_queue_sites_for_test(&root, &session, &config);
    let once = lang_facts_once_extractions(&observer);
    assert!(once > 1, "extraction must scan more than one file");

    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let prepared = crate::server_routes::PreparedServerAnalysis {
        root: root.clone(),
        source_files: std::sync::Arc::new(Vec::new()),
        tsconfig: tsconfig.clone(),
        config: Some(config.clone()),
        facts: crate::codebase::ts_source::facts::TsFactMap::default(),
        client_relationships: crate::server_routes::PreparedClientRelationships::new(
            &[],
            &tsconfig,
            &session,
        ),
        session: std::sync::Arc::clone(&session),
    };
    let mut route_facts = std::collections::HashMap::new();
    crate::server_routes::merge_language_route_facts(&prepared, &mut route_facts, None);
    assert_eq!(lang_facts_once_extractions(&observer), once);

    let options = lang_facts_once_options(&config);
    let _edges = super::collect_language_frontend_edges(
        &root,
        &files,
        Some(&options),
        &sources,
        &crate::codebase::analysis_session::PathInterner::new(),
        &session,
    );
    assert_eq!(lang_facts_once_extractions(&observer), once);

    let lang = crate::codebase::lang_frontends::lang_config_from_v2(&config);
    let facts = session.lang_frontend_facts(&root, &files, &lang, &sources);
    assert_eq!(once, lang_facts_once_scanned(&facts));
    assert_eq!(lang_facts_once_extractions(&observer), once);

    let same_files = std::sync::Arc::new(files.iter().cloned().collect::<Vec<_>>());
    let shared = session.lang_frontend_facts(&root, &same_files, &lang, &sources);
    assert!(std::sync::Arc::ptr_eq(&facts, &shared));
    assert_eq!(lang_facts_once_extractions(&observer), once);

    let mut changed = lang.clone();
    changed.python_packages.push("extra-package".into());
    let changed_facts = session.lang_frontend_facts(&root, &files, &changed, &sources);
    let after_config = lang_facts_once_extractions(&observer);
    assert_eq!(after_config, once + lang_facts_once_scanned(&changed_facts));

    let mut fewer = (*files).clone();
    fewer.pop();
    let fewer = std::sync::Arc::new(fewer);
    let fewer_facts = session.lang_frontend_facts(&root, &fewer, &lang, &sources);
    let after_files = lang_facts_once_extractions(&observer);
    assert_eq!(
        after_files,
        after_config + lang_facts_once_scanned(&fewer_facts)
    );

    let other_sources = std::sync::Arc::new(crate::codebase::ts_source::SourceStore::new(
        std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
            files.as_slice(),
        )),
    ));
    let other_facts = session.lang_frontend_facts(&root, &files, &lang, &other_sources);
    assert_eq!(
        lang_facts_once_extractions(&observer),
        after_files + lang_facts_once_scanned(&other_facts)
    );
}

#[test]
fn lang_facts_once_skips_the_work_counter_without_verbose_diagnostics() {
    let root = lang_facts_once_root();
    let observer = crate::diagnostics::InvocationObserver::new(false);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let dataset = session.dataset(&root);
    let files = dataset.paths_for(&root);
    let sources = dataset.sources_for(&root);
    let config = crate::codebase::lang_frontends::LangFrontendConfig {
        python_packages: vec!["python-celery-django/app".into()],
        ..Default::default()
    };
    let facts = session.lang_frontend_facts(&root, &files, &config, &sources);
    assert!(lang_facts_once_scanned(&facts) > 0);
    assert!(!observer
        .snapshot()
        .work
        .contains_key("lang_frontends.file_extractions"));

    let disabled = crate::codebase::analysis_session::AnalysisSession::disabled();
    let disabled_facts = disabled.lang_frontend_facts(&root, &files, &config, &sources);
    assert!(lang_facts_once_scanned(&disabled_facts) > 0);
    assert!(!observer
        .snapshot()
        .work
        .contains_key("lang_frontends.file_extractions"));
}
