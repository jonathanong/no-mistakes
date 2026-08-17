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

fn src(root: &std::path::Path) -> std::sync::Arc<crate::codebase::ts_source::SourceStore> {
    crate::codebase::rules::source_store_for_files(&all_files(root))
}

#[test]
fn python_keeps_import_aliases_and_masks_docstring_imports() {
    let root = fixture("python-celery-django");
    let facts = collect_python_facts(&root, &all_files(&root), &["app".into()], &src(&root));
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
    let facts = collect_rust_facts(&root, &all_files(&root), &[".".into()], &src(&root));
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
    let facts = collect_php_facts(
        &root,
        &all_files(&root),
        &[".".into()],
        Some("laravel"),
        &src(&root),
    );
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
    let facts = collect_go_facts(
        &root,
        &all_files(&root),
        &["worker".into(), "nested".into()],
        &src(&root),
    );
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
