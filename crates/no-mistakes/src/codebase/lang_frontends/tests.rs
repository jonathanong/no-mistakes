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
        .any(|import| import.contains("models")));
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
    assert!(enqueue
        .queue_enqueues
        .iter()
        .any(|job| job == "send_welcome"));
    let urls = facts
        .files
        .values()
        .find(|file| file.path.ends_with("urls.py"))
        .expect("urls");
    assert!(urls
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "api/users/" && handler.contains("user_list")));
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
    assert_eq!(enqueue.queue_enqueues, vec!["mail:welcome".to_string()]);
    let tasks = facts
        .files
        .values()
        .find(|file| file.path.ends_with("tasks.go"))
        .expect("tasks");
    assert_eq!(tasks.queue_workers, vec!["mail:welcome".to_string()]);
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

#[test]
fn kafka_extracts_static_topics_and_skips_dynamic() {
    let (produces, consumes) = extract_kafka_topics(
        r#"
        producer.send({ topic: "mail.welcome" });
        consumer.subscribe({ topic: "mail.welcome" });
        producer.send({ topic: prefix + name });
        "#,
    );
    assert_eq!(produces, vec!["mail.welcome".to_string()]);
    assert_eq!(consumes, vec!["mail.welcome".to_string()]);
    assert_eq!(
        topic_identity(Some("orders"), "mail.welcome"),
        "orders:mail.welcome"
    );
}

#[test]
fn empty_config_collects_nothing() {
    let root = fixture("python-celery-django");
    let files = all_files(&root);
    assert!(collect_python_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_go_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_rust_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_ruby_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_php_facts(&root, &files, &[], None).files.is_empty());
}
