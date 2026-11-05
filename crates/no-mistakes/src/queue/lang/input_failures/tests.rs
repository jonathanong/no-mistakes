use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use globset::Glob;
use std::sync::Arc;

#[test]
fn selected_go_sources_include_module_manifest_and_share_read_failures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends/go-asynq")
        .canonicalize()
        .unwrap();
    let files = vec![
        root.join("worker/enqueue.go"),
        root.join("go.mod"),
        root.join("nested/go.mod"),
        root.join("nested/mail.go"),
    ];
    let matcher = Glob::new("worker/**/*.go").unwrap().compile_matcher();
    let globs = QueueGlobMatchers {
        enqueues: vec![(matcher, "worker/**/*.go".into())],
        workers: vec![],
        clusters: Default::default(),
        default_cluster: None,
    };
    let config = LangFrontendConfig {
        go_modules: vec![".".into()],
        ..Default::default()
    };
    let selected = selected_inputs(&root, &files, &config, &globs);
    assert_eq!(
        selected,
        vec![root.join("go.mod"), root.join("worker/enqueue.go")]
    );
    let source = crate::test_support::materialize_saved_fixture(&root);
    let removed = source.path().join("worker/enqueue.go");
    let manifest = source.path().join("go.mod");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[
        removed.clone(),
        manifest.clone(),
    ])));
    std::fs::remove_file(&removed).unwrap();
    let failures = read_failures(&[removed.clone(), manifest], &sources);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].0, removed);
    assert!(failures[0].1.contains("failed to read"));
    assert_eq!(read_failures(&[removed], &sources), failures);
}
