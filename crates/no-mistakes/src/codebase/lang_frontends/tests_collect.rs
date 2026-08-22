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
        java_packages: vec!["java-spring".into()],
        kotlin_packages: vec!["kotlin-spring".into()],
        elixir_apps: vec!["phoenix-routes".into()],
        dart_packages: vec![],
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
    assert_eq!(
        collected.java,
        collect_java_facts(&root, &files, &config.java_packages, &store)
    );
    assert_eq!(
        collected.kotlin,
        collect_kotlin_facts(&root, &files, &config.kotlin_packages, &store)
    );
    assert_eq!(
        collected.elixir,
        collect_elixir_facts(&root, &files, &config.elixir_apps, &store)
    );
    assert_eq!(
        collected.dart,
        collect_dart_facts(&root, &files, &config.dart_packages, &store)
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
    assert!(
        !collected.java.files.is_empty(),
        "composed fixture must produce java facts"
    );
    assert!(
        !collected.kotlin.files.is_empty(),
        "composed fixture must produce kotlin facts"
    );
    assert!(
        !collected.elixir.files.is_empty(),
        "composed fixture must produce elixir facts"
    );
    assert!(
        collected.dart.files.is_empty(),
        "composed fixture must not parse Dart packages"
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
    assert!(collected.java.files.is_empty());
    assert!(collected.kotlin.files.is_empty());
    assert!(collected.elixir.files.is_empty());
    assert!(collected.dart.files.is_empty());
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
    assert!(collected.java.files.is_empty());
    assert!(collected.kotlin.files.is_empty());
    assert!(collected.elixir.files.is_empty());
    assert!(collected.dart.files.is_empty());
}
