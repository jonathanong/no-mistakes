use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn fetches_cmd() -> Command {
    let mut cmd = Command::cargo_bin("no-mistakes").unwrap();
    cmd.arg("fetches");
    cmd
}

#[test]
fn test_cli_no_fetches() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    fs::write(
        root.path().join("app/page.tsx"),
        "export default function Page() { return null; }",
    )
    .unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("(no fetches found)"));
}

#[test]
fn test_cli_matches_explicit_target_file() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    let page = root.path().join("app/page.tsx");
    fs::write(&page, "fetch('/api/explicit-target');").unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path()).arg(&page);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/explicit-target"));
}

#[test]
fn test_cli_target_file_match_uses_import_chain() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();

    let page = root.path().join("app/page.tsx");
    let middle = root.path().join("app/middle.ts");
    let target = root.path().join("app/target.ts");
    fs::write(&page, "import { helper } from './middle';\nhelper();").unwrap();
    fs::write(&middle, "import { helper } from './target';\nhelper();").unwrap();
    fs::write(&target, "fetch('/api/targeted');").unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path()).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/targeted"));
}

#[test]
fn test_cli_target_matching_uses_layout_wrapper_chain() {
    let root = tempdir().unwrap();
    let app = root.path().join("app");
    fs::create_dir_all(app.join("dashboard")).unwrap();

    let layout = app.join("layout.tsx");
    fs::write(&layout, "fetch('/api/layout');").unwrap();
    let page = app.join("dashboard/page.tsx");
    fs::write(&page, "fetch('/api/page');").unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path()).arg(&page);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_target_file_match_uses_layout_import_chain() {
    let root = tempdir().unwrap();
    let app = root.path().join("app");
    fs::create_dir_all(app.join("dashboard")).unwrap();

    let layout = app.join("dashboard/layout.tsx");
    fs::write(
        &layout,
        "
            import { helper } from './target';
            ",
    )
    .unwrap();
    let page = app.join("dashboard/page.tsx");
    fs::write(&page, "fetch('/api/page');").unwrap();
    let target = app.join("dashboard/target.ts");
    fs::write(&target, "export const helper = () => fetch('/api/target');").unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path()).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/page"));
}

#[test]
fn test_cli_includes_page_and_layout_routes_by_default() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    fs::write(root.path().join("app/layout.tsx"), "fetch('/api/layout');").unwrap();
    fs::write(root.path().join("app/page.tsx"), "fetch('/api/page');").unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root")
        .arg(root.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("/api/page"))
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_includes_client_side_cached_duplicates() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    fs::write(
        root.path().join("app/page.tsx"),
        "
            'use client';
            fetch('/api/cached', { cache: 'force-cache' });
            fetch('/api/cached', { cache: 'force-cache' });
            ",
    )
    .unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Cached API Calls: 2"))
        .stdout(predicates::str::contains("## Duplicates"));
}

#[test]
fn test_cli_follows_imports_when_analyzing_routes() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    fs::write(
        root.path().join("app/helper.ts"),
        "export const fetchUsers = () => fetch('/api/users');",
    )
    .unwrap();
    fs::write(
        root.path().join("app/page.tsx"),
        "import { fetchUsers } from './helper';\nfetchUsers();",
    )
    .unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/users"));
}

#[test]
fn test_cli_target_missing_reports_unmatched_error() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    fs::write(root.path().join("app/page.tsx"), "fetch('/api/page');").unwrap();

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(root.path()).arg("does-not-exist.ts");
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
