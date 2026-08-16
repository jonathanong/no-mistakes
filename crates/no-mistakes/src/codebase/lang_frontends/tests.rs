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
fn python_collects_relative_import_celery_and_django_routes() {
    let root = fixture("python-celery-django");
    let facts = collect_python_facts(&root, &all_files(&root), &["app".into()]);
    let views = facts
        .files
        .keys()
        .find(|path| path.ends_with("users/views.py"))
        .cloned()
        .expect("views");
    assert!(facts.files[&views]
        .imports
        .iter()
        .any(|import| import == "app.users.models" || import.ends_with(".models")));
    let urls = facts
        .files
        .values()
        .find(|file| file.path.ends_with("urls.py"))
        .expect("urls");
    assert!(urls
        .imports
        .iter()
        .any(|import| import == "app.users.views"));
    assert_eq!(
        facts.files[&views].module.as_deref(),
        Some("app.users.views")
    );
    let tasks = facts
        .files
        .values()
        .find(|file| file.path.ends_with("tasks.py"))
        .expect("tasks");
    assert!(tasks
        .queue_workers
        .iter()
        .any(|job| job.contains("send_welcome")));
    let enqueue = facts
        .files
        .values()
        .find(|file| file.path.ends_with("enqueue.py"))
        .expect("enqueue");
    assert!(enqueue.imports.iter().any(|import| import == "app.tasks"));
    assert!(enqueue
        .imports
        .iter()
        .any(|import| import == "app.users.models"));
    assert!(enqueue
        .queue_enqueues
        .iter()
        .any(|job| job == "send_welcome"));
    assert!(urls
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "api/users/" && handler.contains("user_list")));
    assert!(urls
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "users/" && handler.contains("UserView")));
}

#[test]
fn go_collects_asynq_task_and_handler() {
    let root = fixture("go-asynq");
    let facts = collect_go_facts(&root, &all_files(&root), &["worker".into()]);
    let enqueue = facts
        .files
        .values()
        .find(|file| file.path.ends_with("enqueue.go"))
        .expect("enqueue");
    assert!(enqueue.imports.iter().any(|import| import == "fmt"));
    assert!(enqueue
        .imports
        .iter()
        .any(|import| import == "github.com/hibiken/asynq"));
    assert_eq!(enqueue.queue_enqueues, vec!["mail:welcome".to_string()]);
    let tasks = facts
        .files
        .values()
        .find(|file| file.path.ends_with("tasks.go"))
        .expect("tasks");
    assert_eq!(tasks.queue_workers, vec!["mail:welcome".to_string()]);
    assert!(tasks
        .declarations
        .iter()
        .any(|name| name == "WelcomePayload" || name == "HandleWelcome"));
    let nested = collect_go_facts(&root, &all_files(&root), &[".".into(), "nested".into()]);
    let mail = nested
        .files
        .values()
        .find(|file| file.path.ends_with("nested/mail.go"))
        .expect("nested");
    assert_eq!(mail.module.as_deref(), Some("example.com/nested"));
}

#[test]
fn rust_collects_use_and_declaration() {
    let root = fixture("rust-mods");
    let facts = collect_rust_facts(&root, &all_files(&root), &[".".into()]);
    let lib = facts
        .files
        .values()
        .find(|file| file.path.ends_with("lib.rs"))
        .expect("lib");
    assert!(lib.imports.iter().any(|import| import == "mail"));
    assert!(lib.mods.iter().any(|name| name == "mail"));
    assert!(facts
        .files
        .values()
        .any(|file| file.module.as_deref() == Some("mail")));
    assert!(facts.declarations.contains_key("Welcome"));
}

#[test]
fn rails_collects_route_and_active_job() {
    let root = fixture("rails-jobs");
    let facts = collect_ruby_facts(&root, &all_files(&root), &[".".into()]);
    let routes = facts
        .files
        .values()
        .find(|file| file.path.ends_with("routes.rb"))
        .expect("routes");
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/api/users" && handler == "users#index"));
    let controller = facts
        .files
        .values()
        .find(|file| file.path.ends_with("users_controller.rb"))
        .expect("controller");
    assert_eq!(controller.queue_enqueues, vec!["WelcomeJob".to_string()]);
}

#[test]
fn php_collects_laravel_route_and_dispatch() {
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
        .any(|(route, _)| route == "/api/users"));
    let job = facts
        .files
        .values()
        .find(|file| file.path.ends_with("SomeJob.php"))
        .expect("job");
    assert!(!job.queue_workers.is_empty());
    assert!(job
        .declarations
        .iter()
        .any(|name| name == "App.Jobs.SomeJob" || name == "SomeJob"));
}
