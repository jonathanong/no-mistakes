#[test]
fn language_frontend_edges_keep_go_imports_across_modules() {
    let go = lang_fixture("go-asynq");
    let mut options = lang_options();
    options.go_modules = vec!["worker".into(), "nested".into()];
    let go_edges = collect_language_frontend_edges(&go, &lang_files(&go), Some(&options));
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
    let edges = collect_language_frontend_edges(&kafka, &lang_files(&kafka), Some(&options));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::QueueEnqueue));
}
