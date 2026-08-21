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
fn python_keeps_import_aliases_and_masks_docstring_imports() {
    let root = fixture("python-celery-django");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_python_facts(&root, &files, &["app".into()], &store);
    let enqueue = facts
        .files
        .values()
        .find(|file| file.path.ends_with("enqueue.py"))
        .expect("enqueue");
    assert!(enqueue
        .imports
        .iter()
        .any(|import| import == "celery_tasks=app.tasks"));
    let views = facts
        .files
        .values()
        .find(|file| file.path.ends_with("users/views.py"))
        .expect("views");
    assert!(!views
        .imports
        .iter()
        .any(|import| import.contains("fake_docstring")));
}

#[test]
fn rust_skips_inline_mods_and_treats_crate_root_self_as_root() {
    let root = fixture("rust-mods");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_rust_facts(&root, &files, &[".".into()], &store);
    let lib = facts
        .files
        .values()
        .find(|file| file.path.ends_with("lib.rs"))
        .expect("lib");
    assert!(lib.module.is_none());
    assert!(lib.imports.iter().any(|import| import == "mail"));
    assert!(!lib.imports.iter().any(|import| import.starts_with("lib.")));
    let mail = facts
        .files
        .values()
        .find(|file| file.path.ends_with("mail.rs"))
        .expect("mail");
    assert!(!mail.mods.iter().any(|name| name == "unused"));
}

#[test]
fn php_reads_readonly_classes_and_leading_route_separators() {
    let root = fixture("php-laravel");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_php_facts(&root, &files, &[".".into()], Some("laravel"), &store);
    assert!(facts
        .declarations
        .keys()
        .any(|name| name == "App.Dto.UserDto" || name == "UserDto"));
    let routes = facts
        .files
        .values()
        .find(|file| file.path.ends_with("web.php"))
        .expect("routes");
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| { route == "/fq-users" && handler.contains("UserController") }));
}

#[test]
fn kafka_matches_python_send_and_reordered_subscribe() {
    let (produces, consumes) = extract_kafka_topics(
        r#"
        producer.send("mail.welcome", value=payload)
        consumer.subscribe({ fromBeginning: true, topic: "mail.welcome" })
        "#,
    );
    assert_eq!(produces, vec!["mail.welcome".to_string()]);
    assert_eq!(consumes, vec!["mail.welcome".to_string()]);
}

#[test]
fn go_records_imports_of_configured_sibling_modules() {
    let root = fixture("go-asynq");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_go_facts(&root, &files, &["worker".into(), "nested".into()], &store);
    let enqueue = facts
        .files
        .values()
        .find(|file| file.path.ends_with("enqueue.go"))
        .expect("enqueue");
    assert!(enqueue
        .imports
        .iter()
        .any(|import| import == "example.com/nested"));
}

#[test]
fn rust_records_path_attr_mods_and_cargo_path_deps() {
    let root = fixture("rust-path-deps");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_rust_facts(&root, &files, &["app".into(), "helper".into()], &store);
    let lib = facts
        .files
        .values()
        .find(|file| file.path.ends_with("app/src/lib.rs"))
        .expect("app lib");
    assert!(lib.mods.iter().any(|name| name == "alt"));
    assert!(facts
        .package_path_deps
        .contains(&("app".to_string(), "helper".to_string())));
    assert!(facts
        .files
        .keys()
        .any(|path| path.ends_with("tests/integration.rs")));
    assert!(facts
        .files
        .keys()
        .any(|path| path.ends_with("src/tests.rs")));
}

#[test]
    assert!(dynamic.queue_enqueues.is_empty());
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
