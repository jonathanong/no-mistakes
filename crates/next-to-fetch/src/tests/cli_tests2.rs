use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_target_duplicates_are_deduplicated() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    let page = root.path().join("app/page.tsx");
    fs::write(&page, "fetch('/api/page');").unwrap();

    let mut cmd = Command::cargo_bin("next-to-fetch").unwrap();
    cmd.arg("--root").arg(root.path()).arg(&page).arg(&page);
    cmd.assert().success();
}

#[test]
fn test_cli_target_file_match_uses_wrapper_chain() {
    let root = tempdir().unwrap();
    let app = root.path().join("app");
    fs::create_dir_all(app.join("dashboard")).unwrap();

    let layout = app.join("layout.tsx");
    fs::write(&layout, "fetch('/api/layout');").unwrap();
    let page = app.join("dashboard/page.tsx");
    fs::write(&page, "fetch('/api/page');").unwrap();

    let mut cmd = Command::cargo_bin("next-to-fetch").unwrap();
    cmd.arg("--root").arg(root.path()).arg(&layout);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_target_file_match_uses_wrapper_chain_via_import() {
    let root = tempdir().unwrap();
    let app = root.path().join("app");
    fs::create_dir_all(app.join("dashboard")).unwrap();

    let layout = app.join("dashboard/layout.tsx");
    fs::write(
        &layout,
        "
            import { target } from './target.ts';
            fetch('/api/layout');
            ",
    )
    .unwrap();
    let page = app.join("dashboard/page.tsx");
    fs::write(&page, "fetch('/api/page');").unwrap();
    let target = app.join("dashboard/target.ts");
    fs::write(&target, "export const target = 123;").unwrap();

    let mut cmd = Command::cargo_bin("next-to-fetch").unwrap();
    cmd.arg("--root").arg(root.path()).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_target_file_match_skips_non_matching_wrapper_then_matches_outer() {
    let root = tempdir().unwrap();
    let app = root.path().join("app");
    fs::create_dir_all(app.join("dashboard")).unwrap();

    // inner layout does NOT import the target
    let inner_layout = app.join("dashboard/layout.tsx");
    fs::write(
        &inner_layout,
        "export default function Layout({ children }) { return children; }",
    )
    .unwrap();
    let page = app.join("dashboard/page.tsx");
    fs::write(&page, "fetch('/api/dashboard');").unwrap();
    // outer layout DOES import the target
    let outer_layout = app.join("layout.tsx");
    fs::write(
        &outer_layout,
        "import { x } from './target.ts'; fetch('/api/root');",
    )
    .unwrap();
    let target = app.join("target.ts");
    fs::write(&target, "export const x = 1;").unwrap();

    let mut cmd = Command::cargo_bin("next-to-fetch").unwrap();
    cmd.arg("--root").arg(root.path()).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/root"));
}

#[test]
fn test_cli_includes_client_side_fetches() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    fs::write(
        root.path().join("app/page.tsx"),
        "'use client';\nfetch('/api/client');",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("next-to-fetch").unwrap();
    cmd.arg("--root").arg(root.path());
    cmd.assert().success().stdout(predicates::str::contains(
        "| GET | `/api/client` | client |",
    ));
}

#[test]
fn test_cli_sorts_multiple_unsupported_fetches() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("app/about")).unwrap();
    fs::write(root.path().join("app/page.tsx"), "fetch(url);").unwrap();
    fs::write(root.path().join("app/about/page.tsx"), "fetch(dynamic);").unwrap();

    let mut cmd = Command::cargo_bin("next-to-fetch").unwrap();
    cmd.arg("--root").arg(root.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("## Unsupported (Dynamic)"))
        .stdout(predicates::str::contains("### / (app/page.tsx)"))
        .stdout(predicates::str::contains("### /about (app/about/page.tsx)"));
}

#[test]
fn test_cli_includes_duplicates_and_unsupported_sections() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("app")).unwrap();
    fs::write(
        root.path().join("app/page.tsx"),
        "
            fetch(`/api/${dynamic}`);
            fetch('/api/duplicate');
            fetch('/api/duplicate');
            ",
    )
    .unwrap();
    fs::create_dir_all(root.path().join("app/about")).unwrap();
    fs::write(
        root.path().join("app/about/page.tsx"),
        "
            fetch('/api/second-duplicate');
            fetch('/api/second-duplicate');
            ",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("next-to-fetch").unwrap();
    cmd.arg("--root").arg(root.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("## Duplicates"))
        .stdout(predicates::str::contains("## Unsupported (Dynamic)"));
}
