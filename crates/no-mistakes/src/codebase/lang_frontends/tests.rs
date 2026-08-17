use super::*;
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
fn python_collects_relative_import_celery_and_django_routes() {
    let root = fixture("python-celery-django");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_python_facts(&root, &files, &["app".into()], &store);
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
        .find(|file| file.path.ends_with("app/urls.py") && !file.path.ends_with("api/urls.py"))
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
fn python_collects_flask_and_fastapi_literal_routes() {
    let root = fixture("python-flask-fastapi");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_python_facts(&root, &files, &[".".into()], &store);
    let flask = facts
        .files
        .values()
        .find(|file| file.path.ends_with("flask_app.py"))
        .expect("flask");
    assert!(flask
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/users" && handler == "users"));
    assert!(flask
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/ping" && handler == "ping"));
    let fastapi = facts
        .files
        .values()
        .find(|file| file.path.ends_with("fastapi_app.py"))
        .expect("fastapi");
    assert!(fastapi
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/health" && handler == "health"));
    let computed = facts
        .files
        .values()
        .find(|file| file.path.ends_with("computed.py"))
        .expect("computed");
    assert!(computed.route_handlers.is_empty());
}

#[test]
fn go_collects_http_literal_routes() {
    let root = fixture("go-http");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_go_facts(&root, &files, &[".".into()], &store);
    let routes = facts
        .files
        .values()
        .find(|file| file.path.ends_with("routes.go"))
        .expect("routes");
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/health" && handler == "Health"));
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/users" && handler == "Users"));
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/items" && handler == "CreateItem"));
    assert!(routes.queue_workers.is_empty());
    let computed = facts
        .files
        .values()
        .find(|file| file.path.ends_with("computed.go"))
        .expect("computed");
    assert!(computed.route_handlers.is_empty());
}

#[test]
fn go_collects_asynq_task_and_handler() {
    let root = fixture("go-asynq");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_go_facts(&root, &files, &["worker".into()], &store);
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
    let nested = collect_go_facts(&root, &files, &[".".into(), "nested".into()], &store);
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
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_rust_facts(&root, &files, &[".".into()], &store);
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
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_ruby_facts(&root, &files, &[".".into()], &store);
    let routes = facts
        .files
        .values()
        .find(|file| file.path.ends_with("routes.rb"))
        .expect("routes");
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/api/users" && handler == "users#index"));
    assert!(routes
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/admin/users" && handler == "admin/users#index"));
    assert!(facts.declarations.contains_key("Admin::UsersController"));
    let controller = facts
        .files
        .values()
        .find(|file| file.path.ends_with("controllers/users_controller.rb"))
        .expect("controller");
    assert_eq!(controller.queue_enqueues, vec!["WelcomeJob".to_string()]);
}

#[test]
fn php_collects_laravel_route_and_dispatch() {
    let root = fixture("php-laravel");
    let files = all_files(&root);
    let store = store_for(&files);
    let facts = collect_php_facts(&root, &files, &[".".into()], Some("laravel"), &store);
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
    assert!(facts
        .declarations
        .keys()
        .any(|name| name.contains("Mailer")));
    assert!(routes
        .imports
        .iter()
        .any(|import| import == "App.Jobs.SomeJob"));
    assert!(routes.imports.iter().all(|import| !import.contains(" as ")));
}

#[test]
fn collect_all_lang_facts_matches_independent_language_collectors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends"),
    );
    let files = all_files(&root);
    let config = LangFrontendConfig {
        python_packages: vec!["python-celery-django/app".into()],
        go_modules: vec!["go-asynq".into(), "go-asynq/worker".into()],
        rust_packages: vec!["rust-mods".into(), "rust-mods/src".into()],
        rails_apps: vec!["rails-jobs".into()],
        php_apps: vec!["php-laravel".into()],
        php_framework: Some("laravel".into()),
    };
    let store = store_for(&files);
    let collected = collect_all_lang_facts(&root, &files, &config, &store);
    assert_eq!(
        collected.python,
        collect_python_facts(&root, &files, &config.python_packages, &store)
    );
    assert_eq!(
        collected.go,
        collect_go_facts(&root, &files, &config.go_modules, &store)
    );
    assert_eq!(
        collected.rust,
        collect_rust_facts(&root, &files, &config.rust_packages, &store)
    );
    assert_eq!(
        collected.ruby,
        collect_ruby_facts(&root, &files, &config.rails_apps, &store)
    );
    assert_eq!(
        collected.php,
        collect_php_facts(
            &root,
            &files,
            &config.php_apps,
            config.php_framework.as_deref(),
            &store,
        )
    );
    assert!(
        !collected.python.files.is_empty(),
        "composed fixture must produce python facts"
    );
    assert!(
        !collected.go.files.is_empty(),
        "composed fixture must produce go facts"
    );
    assert!(
        !collected.rust.files.is_empty(),
        "composed fixture must produce rust facts"
    );
    assert!(
        !collected.ruby.files.is_empty(),
        "composed fixture must produce ruby facts"
    );
    assert!(
        !collected.php.files.is_empty(),
        "composed fixture must produce php facts"
    );
}

#[test]
fn collect_all_lang_facts_with_partially_configured_languages() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends"),
    );
    let files = all_files(&root);
    let config = LangFrontendConfig {
        python_packages: vec!["python-celery-django/app".into()],
        go_modules: vec!["go-asynq".into(), "go-asynq/worker".into()],
        ..LangFrontendConfig::default()
    };
    let store = store_for(&files);
    let collected = collect_all_lang_facts(&root, &files, &config, &store);
    assert_eq!(
        collected.python,
        collect_python_facts(&root, &files, &config.python_packages, &store)
    );
    assert_eq!(
        collected.go,
        collect_go_facts(&root, &files, &config.go_modules, &store)
    );
    assert!(collected.rust.files.is_empty());
    assert!(collected.ruby.files.is_empty());
    assert!(collected.php.files.is_empty());
    assert!(!collected.python.files.is_empty());
    assert!(!collected.go.files.is_empty());
}

#[test]
fn collect_all_lang_facts_skips_unconfigured_languages() {
    let root = fixture("python-celery-django");
    let files = all_files(&root);
    let store = store_for(&files);
    let collected = collect_all_lang_facts(&root, &files, &LangFrontendConfig::default(), &store);
    assert!(collected.python.files.is_empty());
    assert!(collected.go.files.is_empty());
    assert!(collected.rust.files.is_empty());
    assert!(collected.ruby.files.is_empty());
    assert!(collected.php.files.is_empty());
}

#[test]
fn second_language_collect_reuses_source_store_reads() {
    let root = fixture("python-celery-django");
    let files = all_files(&root);
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        &files,
    ));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let store = crate::codebase::ts_source::SourceStore::new_observed(
        inventory,
        Some(std::sync::Arc::clone(&observer)),
    );
    let first = collect_python_facts(&root, &files, &["app".into()], &store);
    assert!(!first.files.is_empty());
    let reads_after_first = observer
        .snapshot()
        .work
        .get("source.reads")
        .copied()
        .unwrap_or(0);
    assert!(reads_after_first > 0);
    let second = collect_python_facts(&root, &files, &["app".into()], &store);
    assert_eq!(first, second);
    let reads_after_second = observer
        .snapshot()
        .work
        .get("source.reads")
        .copied()
        .unwrap_or(0);
    assert_eq!(reads_after_first, reads_after_second);
}
