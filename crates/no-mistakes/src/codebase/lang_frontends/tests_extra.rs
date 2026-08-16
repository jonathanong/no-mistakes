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
    assert!(collect_python_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_go_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_rust_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_ruby_facts(&root, &files, &[]).files.is_empty());
    assert!(collect_php_facts(&root, &files, &[], None).files.is_empty());
}

#[test]
fn php_without_framework_skips_laravel_extractors() {
    let root = fixture("php-laravel");
    let facts = collect_php_facts(&root, &all_files(&root), &[".".into()], None);
    let routes = facts.files.values().find(|f| f.path.ends_with("web.php"));
    assert!(routes.is_some_and(|file| file.route_handlers.is_empty()));
}
