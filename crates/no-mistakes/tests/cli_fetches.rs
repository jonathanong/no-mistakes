use assert_cmd::Command;
use std::path::PathBuf;

pub(super) fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/nextjs-fetches/cli/fixture")
            .join(name),
    )
}

pub(super) fn fetches_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    // Each integration child gets its own invocation lock, including stderr checks.
    cmd.env(
        "CARGO_BIN_EXE_no-mistakes",
        env!("CARGO_BIN_EXE_no-mistakes"),
    );
    cmd.arg("fetches");
    cmd
}

#[test]
fn test_cli_no_fetches() {
    let root = fixture("no-fetches");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("(no fetches found)"));
}

#[test]
fn test_cli_matches_explicit_target_file() {
    let root = fixture("matches-explicit-target-file");
    let page = root.join("app/page.tsx");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&page);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/explicit-target"));
}

// The saved fixture chain is intentionally a valid import-and-call path.
#[test]
fn test_cli_target_file_match_uses_import_chain() {
    let root = fixture("target-file-match-uses-import-chain");
    let target = root.join("app/target.ts");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/targeted"));
}

#[test]
fn test_cli_target_matching_uses_layout_wrapper_chain() {
    let root = fixture("target-matching-uses-layout-wrapper-chain");
    let page = root.join("app/dashboard/page.tsx");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&page);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_target_file_match_uses_layout_import_chain() {
    let root = fixture("target-file-match-uses-layout-import-chain");
    let target = root.join("app/dashboard/target.ts");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/page"));
}

#[test]
fn test_cli_includes_page_and_layout_routes_by_default() {
    let root = fixture("includes-page-and-layout-routes-by-default");

    let mut cmd = fetches_cmd();
    cmd.arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicates::str::contains("/api/page"))
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_includes_client_side_cached_duplicates() {
    let root = fixture("includes-client-side-cached-duplicates");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Cached API Calls: 2"))
        .stdout(predicates::str::contains("## Duplicates"));
}

#[test]
fn test_cli_follows_imports_when_analyzing_routes() {
    let root = fixture("follows-imports-when-analyzing-routes");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/users"));
}

#[test]
fn test_cli_target_missing_reports_unmatched_error() {
    let root = fixture("target-missing-reports-unmatched-error");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg("does-not-exist.ts");
    cmd.assert()
        .code(2)
        .stderr(predicates::str::contains("Error: targets not found"));
}

#[test]
fn v2_config_nextjs_project_root_used_for_route_scanning() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-fetches/v2-project-root/fixture");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root);
    cmd.assert().success();
}
