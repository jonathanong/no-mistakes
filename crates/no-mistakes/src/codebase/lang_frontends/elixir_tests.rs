use super::{collect_elixir_facts, extract_elixir_imports, primary_module};
use std::path::PathBuf;

#[test]
fn exact_alias_import_and_use_extract() {
    let source = r#"
alias MyApp.User
alias MyApp.{Role, Team}
import MyApp.Accounts
use Phoenix.Controller
"#;
    assert_eq!(
        extract_elixir_imports(source),
        vec!["MyApp.Accounts", "MyApp.User", "Phoenix.Controller"]
    );
}

#[test]
fn primary_module_prefers_filename_over_nested_builder() {
    assert_eq!(
        primary_module(&["MyApp.Builder".into(), "MyApp.User".into()], Some("user")).as_deref(),
        Some("MyApp.User")
    );
}

#[test]
fn primary_module_matches_underscored_file_stem() {
    assert_eq!(
        primary_module(&["MyAppWeb.UserController".into()], Some("user_controller")).as_deref(),
        Some("MyAppWeb.UserController")
    );
}

#[test]
fn elixir_collects_aliases_and_phoenix_routes() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends/phoenix-routes"),
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
    let facts = collect_elixir_facts(&root, &files, &[".".into()], &store);
    let app = facts
        .files
        .values()
        .find(|file| file.path.ends_with("app.ex"))
        .expect("app");
    assert!(app.imports.iter().any(|import| import == "MyApp.User"));
    assert_eq!(app.module.as_deref(), Some("MyApp.App"));
    let router = facts
        .files
        .values()
        .find(|file| file.path.ends_with("router.ex"))
        .expect("router");
    assert!(router
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/users" && handler.ends_with(".index")));
    assert!(router
        .route_handlers
        .iter()
        .any(|(route, handler)| route == "/users" && handler.ends_with(".create")));
    let computed = facts
        .files
        .values()
        .find(|file| file.path.ends_with("computed.ex"))
        .expect("computed");
    assert!(computed.route_handlers.is_empty());
}
