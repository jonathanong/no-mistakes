use super::*;
use crate::config::v2::NoMistakesConfig;
use globset::Glob;

#[test]
fn empty_v2_config_disables_language_frontends() {
    assert!(lang_config_is_empty(&lang_config_from_v2(
        &NoMistakesConfig::default()
    )));
}

#[test]
fn matching_cluster_requires_globs() {
    let root = std::path::Path::new("/repo");
    let globs = QueueGlobMatchers {
        enqueues: Vec::new(),
        workers: Vec::new(),
        clusters: HashMap::new(),
        default_cluster: None,
    };
    assert!(matching_cluster(root, &root.join("app.py"), &[], &globs).is_none());
}

#[test]
fn matching_cluster_uses_glob_and_default_cluster() {
    let root = std::path::Path::new("/repo");
    let glob = "**/*.py";
    let compiled = Glob::new(glob).unwrap().compile_matcher();
    let globs = QueueGlobMatchers {
        enqueues: vec![(compiled.clone(), glob.to_string())],
        workers: Vec::new(),
        clusters: HashMap::new(),
        default_cluster: Some("orders".into()),
    };
    assert_eq!(
        matching_cluster(root, &root.join("app.py"), &globs.enqueues, &globs),
        Some(Some("orders".into()))
    );
    assert!(matching_cluster(root, &root.join("app.go"), &globs.enqueues, &globs).is_none());
}

#[test]
fn queue_globs_from_v2_compile_prefixed_project_globs() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
projects:
  worker:
    type: server
    root: worker
    queues:
      cluster: orders
      enqueues: ["**/*"]
      workers: ["**/*"]
"#,
    )
    .unwrap();
    let globs = queue_globs_from_v2(&config);
    assert!(!globs.enqueues.is_empty());
    assert_eq!(globs.default_cluster.as_deref(), Some("orders"));
    let root = std::path::Path::new("/repo");
    assert_eq!(
        matching_cluster(
            root,
            &root.join("worker/enqueue.go"),
            &globs.enqueues,
            &globs
        ),
        Some(Some("orders".into()))
    );
}

#[test]
fn prefixed_globs_keep_or_add_project_root() {
    assert_eq!(
        prefixed_globs(Some("."), &["app/**".into()]),
        vec!["app/**"]
    );
    assert_eq!(
        prefixed_globs(Some("worker"), &["**/*".into()]),
        vec!["worker/**/*"]
    );
    assert_eq!(
        prefixed_globs(Some("worker"), &["worker/**/*".into()]),
        vec!["worker/**/*"]
    );
}
