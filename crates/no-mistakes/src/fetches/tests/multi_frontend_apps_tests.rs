//! `no-mistakes fetches` regression coverage for
//! <https://github.com/jonathanong/no-mistakes/issues/624>, using the shared
//! fixture at `fixtures/playwright/multi-frontend-apps/`.

use crate::fetches::cli::Cli;
use crate::fetches::pipeline::run::run_with_base_root;
use no_mistakes::cli::Format;
use std::path::PathBuf;

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/playwright/multi-frontend-apps")
}

fn cli(config: &str) -> Cli {
    Cli {
        root: PathBuf::from("."),
        config: Some(PathBuf::from(config)),
        format: Format::Human,
        json: false,
        targets: vec![],
    }
}

/// Both apps' routes are reported, not just whichever `type: nextjs` project
/// sorts first — the same class of bug as #624, for `no-mistakes fetches`.
#[test]
fn reports_routes_for_both_apps() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture_source());
    let root = fixture.path().canonicalize().unwrap();

    let report = run_with_base_root(&root, &cli("bound.no-mistakes.yml")).unwrap();

    assert!(report
        .routes
        .iter()
        .any(|r| r.route == "/hosts" && r.file.contains("control-web")));
    assert!(report
        .routes
        .iter()
        .any(|r| r.route == "/tasks" && r.file.contains("agent-web")));
    assert_eq!(
        report.routes.iter().filter(|r| r.route == "/").count(),
        2,
        "expected one root route per app: {:?}",
        report.routes.iter().map(|r| &r.file).collect::<Vec<_>>()
    );
}

/// A rewrite configured on `control-web` only must not synthesize a virtual
/// route for `agent-web`.
#[test]
fn app_rewrites_do_not_leak_into_other_apps_routes() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture_source());
    let root = fixture.path().canonicalize().unwrap();

    let report = run_with_base_root(&root, &cli("rewrites.no-mistakes.yml")).unwrap();

    let alias_routes: Vec<_> = report
        .routes
        .iter()
        .filter(|r| r.route == "/control-alias")
        .map(|r| r.file.as_str())
        .collect();
    assert_eq!(alias_routes.len(), 1, "{alias_routes:?}");
    assert!(alias_routes[0].contains("control-web"));
    assert!(!report
        .routes
        .iter()
        .any(|r| r.route == "/control-alias" && r.file.contains("agent-web")));
}
