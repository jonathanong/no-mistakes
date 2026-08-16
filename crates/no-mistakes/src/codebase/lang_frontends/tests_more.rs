use super::*;
use crate::codebase::lang_frontends::kafka::extract_kafka_topics;
use crate::codebase::lang_frontends::strip::mask_strings;
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

#[test]
fn python_masks_docstring_symbols_and_keeps_include_routes() {
    let root = fixture("python-celery-django");
    let facts = collect_python_facts(&root, &all_files(&root), &["app".into()]);
    let views = facts
        .files
        .values()
        .find(|file| file.path.ends_with("users/views.py"))
        .expect("views");
    assert!(!views.declarations.iter().any(|name| name == "LegacyUser"));
    let urls = facts
        .files
        .values()
        .find(|file| file.path.ends_with("app/urls.py") && !file.path.ends_with("api/urls.py"))
        .expect("urls");
    assert!(urls
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "api/" && handler == "app.api.urls"));
    assert_eq!(
        mask_strings(r#"x = "class Hidden:" """class Doc:""" 'ok'"#).contains("Hidden"),
        false
    );
}

#[test]
fn php_collects_invokable_routes_and_fq_should_queue() {
    let root = fixture("php-laravel");
    let facts = collect_php_facts(&root, &all_files(&root), &[".".into()], Some("laravel"));
    let routes = facts
        .files
        .values()
        .find(|file| file.path.ends_with("web.php"))
        .expect("routes");
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/ping" && handler.contains("PingController")));
    let fq = facts
        .files
        .values()
        .find(|file| file.path.ends_with("FqJob.php"))
        .expect("fq job");
    assert!(!fq.queue_workers.is_empty());
}

#[test]
fn kafka_captures_every_subscription_array_topic() {
    let (_, consumes) = extract_kafka_topics(r#"consumer.subscribe(["orders", "payments"]);"#);
    assert!(consumes.iter().any(|topic| topic == "orders"));
    assert!(consumes.iter().any(|topic| topic == "payments"));
}

#[test]
fn go_skips_test_files_and_scopes_package_modules() {
    let root = fixture("go-asynq");
    let facts = collect_go_facts(&root, &all_files(&root), &["worker".into()]);
    let test = facts
        .files
        .values()
        .find(|file| file.path.ends_with("ping_test.go"))
        .expect("test");
    assert!(test.module.is_none());
    let dot = facts
        .files
        .values()
        .find(|file| file.path.ends_with("pkg/dot.go"))
        .expect("dot import");
    assert!(dot.imports.iter().any(|import| import == "mail"));
    let pkg_user = facts
        .files
        .values()
        .find(|file| file.path.ends_with("pkg/user.go"))
        .expect("pkg user");
    assert_ne!(
        pkg_user.module.as_deref(),
        facts
            .files
            .values()
            .find(|file| file.path.ends_with("mail/user.go"))
            .and_then(|file| file.module.as_deref())
    );
}

#[test]
fn rust_expands_grouped_use_trees() {
    let root = fixture("rust-mods");
    let facts = collect_rust_facts(&root, &all_files(&root), &[".".into()]);
    let lib = facts
        .files
        .values()
        .find(|file| file.path.ends_with("lib.rs"))
        .expect("lib");
    assert!(lib.imports.iter().any(|import| import == "aaa"));
    assert!(lib.imports.iter().any(|import| import == "mail"));
}

#[test]
fn ruby_captures_qualified_constants() {
    let root = fixture("rails-jobs");
    let facts = collect_ruby_facts(&root, &all_files(&root), &[".".into()]);
    let controller = facts
        .files
        .values()
        .find(|file| file.path.ends_with("controllers/users_controller.rb"))
        .expect("controller");
    assert!(controller
        .references
        .iter()
        .any(|name| name == "Admin::User"));
    assert!(facts.declarations.contains_key("Admin::User"));
}
