use super::{collect_kotlin_facts, extract_kotlin_imports, extract_package, primary_type};
use std::path::PathBuf;

#[test]
fn package_and_exact_imports_extract() {
    let source = r#"
package com.example
import com.example.User
import com.example.util.*
import com.example.Alias as Named
"#;
    assert_eq!(extract_package(source).as_deref(), Some("com.example"));
    assert_eq!(
        extract_kotlin_imports(source),
        vec!["com.example.Alias", "com.example.User"]
    );
}

#[test]
fn primary_type_prefers_filename_over_nested_builder() {
    assert_eq!(
        primary_type(&["Builder".into(), "Widget".into()], Some("Widget")).as_deref(),
        Some("Widget")
    );
}

#[test]
fn string_literal_imports_are_skipped() {
    let source = r#"
class App {
  val s = """
import com.example.User
"""
}
"#;
    let symbols = crate::codebase::lang_frontends::strip::mask_strings(source);
    assert!(extract_kotlin_imports(&symbols).is_empty());
}

#[test]
fn kotlin_collects_package_imports_and_spring_routes() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends/kotlin-spring"),
    );
    let repo = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    );
    let files = crate::codebase::ts_source::discover_visible_paths(&repo)
        .into_iter()
        .map(|path| {
            let absolute = if path.is_absolute() {
                path
            } else {
                repo.join(path)
            };
            crate::codebase::ts_resolver::normalize_path(&absolute)
        })
        .filter(|path| path.starts_with(&root))
        .collect::<Vec<_>>();
    let store = crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    let facts = collect_kotlin_facts(&root, &files, &[".".into()], &store);
    let app = facts
        .files
        .values()
        .find(|file| file.path.ends_with("App.kt"))
        .expect("app");
    assert!(app
        .imports
        .iter()
        .any(|import| import == "com.example.User"));
    assert_eq!(app.module.as_deref(), Some("com.example.App"));
    let users = facts
        .files
        .values()
        .find(|file| file.path.ends_with("Users.kt"))
        .expect("users");
    assert!(users
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/api/users" && handler == "listUsers"));
    assert!(users
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/api/users" && handler == "createUser"));
    let computed = facts
        .files
        .values()
        .find(|file| file.path.ends_with("Computed.kt"))
        .expect("computed");
    assert!(computed.route_handlers.is_empty());
}
