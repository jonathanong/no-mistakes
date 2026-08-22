use super::*;
use crate::codebase::lang_frontends::kafka::extract_kafka_topics;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends")
            .join(name),
    )
}

fn store_for(files: &[PathBuf]) -> crate::codebase::ts_source::SourceStore {
    crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(files),
    ))
}

fn all_files(root: &std::path::Path) -> Vec<PathBuf> {
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

#[test]
fn kafka_extracts_static_topics_and_skips_dynamic() {
    let (produces, consumes) = extract_kafka_topics(
        r#"
        producer.send({ topic: "mail.welcome" });
        consumer.subscribe({ topic: "mail.welcome" });
        producer.send({ topic: prefix + name });
        // producer.send({ topic: "mail.commented" });
        "#,
    );
    assert_eq!(produces, vec!["mail.welcome".to_string()]);
    assert_eq!(consumes, vec!["mail.welcome".to_string()]);
    let (commented, _) = extract_kafka_topics("// producer.send({ topic: \"mail.commented\" });");
    assert!(commented.is_empty());
    assert_eq!(
        topic_identity(Some("orders"), "mail.welcome"),
        "orders:mail.welcome"
    );
}

#[test]
fn empty_config_collects_nothing() {
    let root = fixture("python-celery-django");
    let files = all_files(&root);
    let store = store_for(&files);
    assert!(collect_python_facts(&root, &files, &[], &store)
        .files
        .is_empty());
    assert!(collect_go_facts(&root, &files, &[], &store)
        .files
        .is_empty());
    assert!(collect_rust_facts(&root, &files, &[], &store)
        .files
        .is_empty());
    assert!(collect_ruby_facts(&root, &files, &[], &store)
        .files
        .is_empty());
    assert!(collect_php_facts(&root, &files, &[], None, &store)
        .files
        .is_empty());
    assert!(collect_java_facts(&root, &files, &[], &store)
        .files
        .is_empty());
}

#[test]
fn php_without_framework_skips_laravel_extractors() {
    let root = fixture("php-laravel");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_php_facts(&root, &files, &[".".into()], None, &store);
    let routes = facts.files.values().find(|f| f.path.ends_with("web.php"));
    assert!(routes.is_some_and(|file| file.route_handlers.is_empty()));
}

#[test]
fn strip_and_kafka_identity_cover_comment_and_empty_cluster_paths() {
    let stripped = super::strip::strip_comments_keep_strings(
        "# hash\n// line\n/* block\nstill */ \"keep // here\" 'ok' \"esc\\\"ape\" done",
    );
    assert!(stripped.contains("keep // here"));
    assert!(stripped.contains("done"));
    assert!(
        super::strip::strip_comments_keep_strings("#[Route('/health')]\n# real\n")
            .contains("#[Route('/health')]")
    );
    assert!(!super::strip::mask_strings("const doc = `LegacyUser`").contains("LegacyUser"));
    let escaped = super::strip::mask_strings("const s = \"a\\\\nb\\nz\"");
    assert!(escaped.contains("\""));
    let unclosed = super::strip::mask_strings("const s = \"open");
    assert!(unclosed.starts_with("const s = \""));
    let triple = super::strip::mask_triple_quoted_strings("'''keep\ncode''' after");
    assert!(triple.contains("'''"));
    assert_eq!(topic_identity(None, "mail.welcome"), "mail.welcome");
    assert_eq!(topic_identity(Some(""), "mail.welcome"), "mail.welcome");
}

#[test]
fn go_and_rust_collectors_cover_missing_manifest_roots() {
    let go = fixture("go-asynq");
    let go_files = all_files(&go);
    let go_store = store_for(&go_files);
    let go_facts = collect_go_facts(&go, &go_files, &["worker".into()], &go_store);
    assert!(!go_facts.files.is_empty());
    assert!(go_facts
        .files
        .values()
        .any(|file| file.path.ends_with("pkg/ping.go") && file.module.as_deref() == Some("pkg")));
    let rust = fixture("rust-mods");
    let rust_files = all_files(&rust);
    let rust_store = store_for(&rust_files);
    let rust_facts = collect_rust_facts(&rust, &rust_files, &["src".into()], &rust_store);
    assert!(rust_facts
        .files
        .values()
        .any(|file| file.mods.is_empty() || file.module.is_some()));
}

#[test]
fn rails_require_relative_and_python_init_module_keys() {
    let rails = fixture("rails-jobs");
    let files = all_files(&rails);
    let store = store_for(&files);
    let facts = collect_ruby_facts(&rails, &files, &[".".into()], &store);
    let controller = facts
        .files
        .values()
        .find(|file| file.path.ends_with("controllers/users_controller.rb"))
        .expect("controller");
    assert!(controller
        .imports
        .iter()
        .any(|import| import == "welcome_job" || import.ends_with("welcome_job")));
    let python = fixture("python-celery-django");
    let files = all_files(&python);
    let store = store_for(&files);
    let facts = collect_python_facts(&python, &files, &["app".into()], &store);
    assert!(facts
        .files
        .values()
        .any(|file| file.module.as_deref() == Some("app")));
    let pkg = std::path::Path::new("/pkg");
    assert_eq!(
        super::facts::module_from_path(pkg, &pkg.join("foo")).as_deref(),
        Some("foo")
    );
}

#[test]
fn rust_collects_http_literal_routes() {
    let root = fixture("rust-http");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_rust_facts(&root, &files, &[".".into()], &store);
    let routes = facts
        .files
        .values()
        .find(|file| file.path.ends_with("routes.rs"))
        .expect("routes");
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/users" && handler == "list_users"));
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/ready" && handler == "ready"));
    let handlers = facts
        .files
        .values()
        .find(|file| file.path.ends_with("handlers.rs"))
        .expect("handlers");
    assert!(handlers
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/health" && handler == "health"));
    assert!(handlers
        .declarations
        .iter()
        .any(|name| name == "list_users"));
    let computed = facts
        .files
        .values()
        .find(|file| file.path.ends_with("computed.rs"))
        .expect("computed");
    assert!(computed.route_handlers.is_empty());
}
