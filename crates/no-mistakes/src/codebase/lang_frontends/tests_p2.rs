use super::*;
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
fn go_skips_unconfigured_nested_modules() {
    let root = fixture("go-asynq");
    let outer_only = collect_go_facts(&root, &all_files(&root), &[".".into()], &src(&root));
    assert!(outer_only
        .files
        .values()
        .all(|file| !file.path.ends_with("nested/mail.go")));
    assert!(outer_only
        .files
        .values()
        .any(|file| file.path.ends_with("enqueue.go")));
}

#[test]
fn ruby_require_relative_uses_normalized_module_key() {
    let root = fixture("rails-jobs");
    let facts = collect_ruby_facts(&root, &all_files(&root), &[".".into()], &src(&root));
    let controller = facts
        .files
        .values()
        .find(|file| file.path.ends_with("controllers/users_controller.rb"))
        .expect("controller");
    let job = facts
        .files
        .values()
        .find(|file| file.path.ends_with("jobs/welcome_job.rb"))
        .expect("job");
    let key = job.module.as_deref().expect("job module");
    assert!(
        controller.imports.iter().any(|import| import == key),
        "require_relative should match normalized module {key}"
    );
    assert!(!controller
        .imports
        .iter()
        .any(|import| import.contains("/../")));
}

#[test]
fn php_queue_identities_are_namespace_qualified() {
    let root = fixture("php-laravel");
    let facts = collect_php_facts(
        &root,
        &all_files(&root),
        &[".".into()],
        Some("laravel"),
        &src(&root),
    );
    let job = facts
        .files
        .values()
        .find(|file| file.path.ends_with("SomeJob.php"))
        .expect("job");
    assert!(job
        .queue_workers
        .iter()
        .any(|name| name == "App.Jobs.SomeJob"));
    assert!(job.queue_workers.iter().all(|name| name.contains('.')));
    let controller = facts
        .files
        .values()
        .find(|file| file.path.ends_with("UserController.php"))
        .expect("controller");
    assert!(controller
        .queue_enqueues
        .iter()
        .any(|name| name == "App.Jobs.SomeJob"));
}

#[test]
fn rust_keeps_intermediate_use_prefixes() {
    let root = fixture("rust-mods");
    let facts = collect_rust_facts(&root, &all_files(&root), &[".".into()], &src(&root));
    let lib = facts
        .files
        .values()
        .find(|file| file.path.ends_with("lib.rs"))
        .expect("lib");
    assert!(lib.imports.iter().any(|import| import == "aaa.helper"));
    assert!(lib.imports.iter().any(|import| import == "aaa"));
}

#[test]
fn go_masks_raw_strings_and_strips_mod_comments() {
    let root = fixture("go-asynq");
    let facts = collect_go_facts(&root, &all_files(&root), &["worker".into()], &src(&root));
    let ping = facts
        .files
        .values()
        .find(|file| file.path.ends_with("pkg/ping.go"))
        .expect("ping");
    assert!(!ping.references.iter().any(|name| name == "LegacyUser"));
    let nested = collect_go_facts(
        &root,
        &all_files(&root),
        &[".".into(), "nested".into()],
        &src(&root),
    );
    let mail = nested
        .files
        .values()
        .find(|file| file.path.ends_with("nested/mail.go"))
        .expect("nested");
    assert_eq!(mail.module.as_deref(), Some("example.com/nested"));
}

#[test]
fn php_collects_static_require_stems() {
    let root = fixture("php-laravel");
    let facts = collect_php_facts(
        &root,
        &all_files(&root),
        &[".".into()],
        Some("laravel"),
        &src(&root),
    );
    let routes = facts
        .files
        .values()
        .find(|file| file.path.ends_with("web.php"))
        .expect("routes");
    assert!(routes.imports.iter().any(|import| import == "helpers"));
}

#[test]
fn ruby_tracks_lexical_module_namespaces() {
    let root = fixture("rails-jobs");
    let facts = collect_ruby_facts(&root, &all_files(&root), &[".".into()], &src(&root));
    assert!(facts.declarations.contains_key("Admin::Ledger"));
}
