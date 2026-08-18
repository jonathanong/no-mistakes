use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn empty_config_returns_no_language_queue_sites() {
    let session = AnalysisSession::disabled();
    let (producers, workers) = language_queue_sites(
        Path::new("/repo"),
        &session,
        &NoMistakesConfig::default(),
        None,
    );
    assert!(producers.is_empty());
    assert!(workers.is_empty());
}

#[test]
fn kafka_skips_paths_the_source_store_cannot_read() {
    let root = PathBuf::from("/repo");
    let missing = root.join("producer.ts");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
projects:
  mail:
    type: server
    root: .
    queues:
      enqueues: ["**/*"]
      workers: ["**/*"]
"#,
    )
    .unwrap();
    let globs = queue_globs_from_v2(&config);
    let mut producers = Vec::new();
    let mut workers = Vec::new();
    extend_kafka(
        &root,
        &[missing],
        &sources,
        &globs,
        None,
        &mut producers,
        &mut workers,
    );
    assert!(producers.is_empty());
    assert!(workers.is_empty());
}

#[test]
fn default_cluster_labels_language_queue_nodes() {
    let root = PathBuf::from("/repo");
    let path = root.join("enqueue.py");
    let producer = language_producer(&root, &path, "WelcomeJob", None);
    let worker = language_worker(&root, &path, "WelcomeJob", None);
    assert_eq!(producer.queue.as_ref().unwrap().queue_name, "default");
    assert_eq!(worker.queue.as_ref().unwrap().queue_name, "default");
}
