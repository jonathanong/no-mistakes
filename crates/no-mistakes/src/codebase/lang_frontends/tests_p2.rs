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

#[test]
fn go_skips_unconfigured_nested_modules() {
    let root = fixture("go-asynq");
    let outer_only = collect_go_facts(&root, &all_files(&root), &[".".into()]);
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
    let facts = collect_ruby_facts(&root, &all_files(&root), &[".".into()]);
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
    let facts = collect_php_facts(&root, &all_files(&root), &[".".into()], Some("laravel"));
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
