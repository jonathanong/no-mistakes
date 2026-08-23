use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::lang_frontends::LangFileFacts;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::NoMistakesConfig;
use globset::{Glob, GlobSetBuilder};
use std::collections::HashMap;
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

fn all_files_globs() -> QueueGlobMatchers {
    let compiled = Glob::new("**/*").unwrap().compile_matcher();
    QueueGlobMatchers {
        enqueues: vec![(compiled.clone(), "**/*".into())],
        workers: vec![(compiled, "**/*".into())],
        clusters: HashMap::new(),
        default_cluster: Some("orders".into()),
    }
}

fn job_file(root: &Path, name: &str) -> LangFileFacts {
    LangFileFacts {
        path: root.join(name),
        queue_enqueues: vec!["WelcomeJob".into()],
        queue_workers: vec!["WelcomeJob".into()],
        ..Default::default()
    }
}

#[test]
fn language_queue_sites_scans_kafka_when_only_globs_are_configured() {
    let session = AnalysisSession::disabled();
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
    let (producers, workers) = language_queue_sites(Path::new("/repo"), &session, &config, None);
    assert!(producers.is_empty());
    assert!(workers.is_empty());
}

#[test]
fn extend_file_skips_paths_outside_the_cli_filter() {
    let root = PathBuf::from("/repo");
    let file = job_file(&root, "app.py");
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("other.py").unwrap());
    let filter = builder.build().unwrap();
    let mut producers = Vec::new();
    let mut workers = Vec::new();
    extend_file(
        &root,
        &file,
        &all_files_globs(),
        Some(&filter),
        &mut producers,
        &mut workers,
    );
    assert!(producers.is_empty());
    assert!(workers.is_empty());
}

#[test]
fn extend_file_projects_matching_enqueue_and_worker_jobs() {
    let root = PathBuf::from("/repo");
    let file = job_file(&root, "app.py");
    let mut producers = Vec::new();
    let mut workers = Vec::new();
    extend_file(
        &root,
        &file,
        &all_files_globs(),
        None,
        &mut producers,
        &mut workers,
    );
    assert_eq!(producers.len(), 1);
    assert_eq!(workers.len(), 1);
    assert_eq!(producers[0].queue.as_ref().unwrap().queue_name, "orders");
    assert_eq!(workers[0].queue.as_ref().unwrap().queue_name, "orders");
}

#[test]
fn extend_kafka_skips_unmatched_and_filtered_paths() {
    let root = PathBuf::from("/repo");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    let compiled = Glob::new("producer.ts").unwrap().compile_matcher();
    let globs = QueueGlobMatchers {
        enqueues: vec![(compiled.clone(), "producer.ts".into())],
        workers: vec![(compiled, "producer.ts".into())],
        clusters: HashMap::new(),
        default_cluster: None,
    };
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("other.ts").unwrap());
    let filter = builder.build().unwrap();
    let mut producers = Vec::new();
    let mut workers = Vec::new();
    extend_kafka(
        &root,
        &[root.join("consumer.ts"), root.join("producer.ts")],
        &sources,
        &globs,
        Some(&filter),
        &mut producers,
        &mut workers,
    );
    assert!(producers.is_empty());
    assert!(workers.is_empty());
}
