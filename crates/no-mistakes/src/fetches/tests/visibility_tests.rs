use crate::fetches::cli::Cli;
use crate::fetches::pipeline::run::run_with_base_root;
use no_mistakes::cli::Format;
use std::path::PathBuf;

fn cli(targets: Vec<String>) -> Cli {
    Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets,
    }
}

#[test]
fn fetch_traversal_excludes_ignored_helpers_layouts_and_templates() {
    let fixture = crate::test_support::materialize_gitignore_fixture("transitive-visibility");

    let report = run_with_base_root(fixture.path(), &cli(Vec::new())).unwrap();
    let paths = report
        .routes
        .iter()
        .flat_map(|route| route.api_calls.iter().map(|call| call.path.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(paths, vec!["/api/visible-page"]);
}

#[test]
fn ignored_import_bridge_cannot_match_an_explicit_fetch_target() {
    let fixture = crate::test_support::materialize_gitignore_fixture("transitive-visibility");

    let error = run_with_base_root(fixture.path(), &cli(vec!["app/target.ts".to_string()]))
        .err()
        .expect("ignored bridge must not match the target");

    assert!(error.to_string().contains("targets not found"));
}
