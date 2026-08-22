#[test]
fn language_frontend_edges_keep_go_imports_across_modules() {
    let go = lang_fixture("go-asynq");
    let mut options = lang_options();
    options.go_modules = vec!["worker".into(), "nested".into()];
    let go_edges = collect_language_frontend_edges_for_test(&go, &lang_files(&go), Some(&options));
    assert!(go_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::GoImport
            && from
                .as_file()
                .is_some_and(|path| path.ends_with("enqueue.go"))
            && to
                .as_file()
                .is_some_and(|path| path.ends_with("nested/mail.go"))
    }));
}

fn matches_any_naive(rel: &Path, globs: &[String]) -> bool {
    globs.iter().any(|glob| {
        globset::Glob::new(glob)
            .ok()
            .is_some_and(|compiled| compiled.compile_matcher().is_match(rel))
    })
}

#[test]
fn compiled_queue_globs_agree_with_per_file_glob_new() {
    let kafka = lang_fixture("kafka-topics");
    let files = lang_files(&kafka);
    // "[" is an invalid glob; compile_queue_globs must ignore it like Glob::new.
    let globs = vec!["**/*".into(), "[".into(), "producer.ts".into()];
    let compiled = compile_queue_globs(&globs);
    for path in &files {
        let rel = path.strip_prefix(&kafka).unwrap_or(path);
        let naive = matches_any_naive(rel, &globs);
        let compiled_hit = compiled
            .matchers
            .iter()
            .any(|(matcher, _)| matcher.is_match(rel));
        assert_eq!(
            naive,
            compiled_hit,
            "compiled glob match must agree with Glob::new per file for {}",
            path.display()
        );
    }
}

#[test]
fn empty_language_config_still_emits_kafka_queue_edges() {
    let kafka = lang_fixture("kafka-topics");
    let options = GraphConfigOptions {
        queue_enqueues: vec!["**/*".into()],
        queue_workers: vec!["**/*".into()],
        queue_cluster: Some("orders".into()),
        ..GraphConfigOptions::default()
    };
    let edges = collect_language_frontend_edges_for_test(&kafka, &lang_files(&kafka), Some(&options));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::QueueEnqueue));
}

#[test]
fn rust_path_deps_emit_package_and_mod_edges() {
    let root = lang_fixture("rust-path-deps");
    let options = GraphConfigOptions {
        rust_packages: vec!["app".into(), "helper".into()],
        ..GraphConfigOptions::default()
    };
    let edges = collect_language_frontend_edges_for_test(&root, &lang_files(&root), Some(&options));
    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RustMod
            && from.as_file().is_some_and(|path| path.ends_with("app/src/lib.rs"))
            && to.as_file().is_some_and(|path| path.ends_with("app/src/alt.rs"))
    }));
    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RustPackage
            && from.as_file().is_some_and(|path| path.ends_with("app/src/lib.rs"))
            && to
                .as_file()
                .is_some_and(|path| path.ends_with("helper/src/lib.rs"))
    }));
    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RustPackage
            && from.as_file().is_some_and(|path| path.ends_with("app/src/lib.rs"))
            && to
                .as_file()
                .is_some_and(|path| path.ends_with("tests/integration.rs"))
    }));
}

#[test]
fn rails_sidekiq_emits_queue_enqueue_and_worker_edges() {
    let root = lang_fixture("rails-sidekiq");
    let edges =
        collect_language_frontend_edges_for_test(&root, &lang_files(&root), Some(&lang_options()));
    assert!(edges.iter().any(|(from, _, kind)| {
        *kind == EdgeKind::QueueEnqueue
            && from
                .as_file()
                .is_some_and(|path| path.ends_with("users_controller.rb"))
    }));
    assert!(edges.iter().any(|(_, to, kind)| {
        *kind == EdgeKind::QueueWorker
            && to.as_file().is_some_and(|path| path.ends_with("mail_worker.rb"))
    }));
    assert!(edges.iter().all(|(from, _, kind)| {
        *kind != EdgeKind::QueueEnqueue
            || from
                .as_file()
                .is_none_or(|path| !path.ends_with("dynamic.rb"))
    }));
}

#[test]
fn dart_exact_imports_cross_configured_packages() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/dart-cross-package/fixture"),
    );
    let options = GraphConfigOptions {
        dart_packages: vec!["libs/shared".into(), "services/app".into()],
        ..GraphConfigOptions::default()
    };
    let edges = collect_language_frontend_edges_for_test(&root, &lang_files(&root), Some(&options));
    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::DartImport
            && from.as_file().is_some_and(|path| path.ends_with("app.dart"))
            && to.as_file().is_some_and(|path| path.ends_with("user.dart"))
    }));
}

#[test]
fn dart_http_calls_match_configured_ts_backend_routes() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/dart-flutter-http/fixture"),
    );
    let options = super::graph_config_options(&root).expect("dart fixture config");
    let files = lang_files(&root);
    let edges = super::collect_http_call_edges(
        &root,
        None,
        &[],
        &files,
        &files,
        Some(&options),
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::HttpCall
            && from.as_file().is_some_and(|path| path.ends_with("api.dart"))
            && to.as_file().is_some_and(|path| path.ends_with("server.ts"))
    }));
    assert!(edges.iter().all(|(_, to, kind)| {
        *kind != EdgeKind::HttpCall
            || to.as_file().is_none_or(|path| !path.ends_with("admin.ts"))
    }));
}
