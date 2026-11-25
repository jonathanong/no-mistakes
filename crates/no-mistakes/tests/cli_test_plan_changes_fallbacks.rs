use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn plan_vitest(root: &Path, extra: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["tests", "plan", "vitest", "--root"])
        .arg(root)
        .args(["--format", "json", "--jobs", "1"])
        .args(extra)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn selected_files(report: &Value) -> Vec<&str> {
    report["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect()
}

fn warning_types(report: &Value) -> Vec<&str> {
    report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|warning| warning["type"].as_str())
        .collect()
}

fn git_cmd(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[test]
fn scoped_category_fallback_does_not_suppress_changed_files_full_suite_fallback() {
    let fixture_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes");
    let fixture = crate::saved_fixture::materialize_path(&fixture_root);
    for extra in [
        &["--changes", "reports/scoped-and-untraceable.json"][..],
        &[
            "--changes",
            "reports/scoped-and-untraceable.json",
            "--limit-files",
            "1",
        ][..],
    ] {
        let report = plan_vitest(fixture.path(), extra);
        assert_eq!(report["fallback_triggered"], true, "{extra:?}");
        assert_eq!(
            selected_files(&report),
            [
                "tests/browser.test.ts",
                "tests/example.test.ts",
                "tests/reader.test.ts",
            ],
            "{extra:?}"
        );
    }
}

#[test]
fn malformed_lockfile_manifest_and_untraceable_dep_fall_back_with_base() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes-union");
    let patches = source.join("patches");
    let reports = source.join("reports");
    let cases = [
        (
            "malformed-pnpm-lock.yaml",
            "pnpm-lock.yaml",
            "lockfile-only.json",
            "lockfile-pnpm-malformed",
        ),
        (
            "malformed-package.json",
            "package.json",
            "package-only.json",
            "package-manifest-malformed",
        ),
        (
            "untraceable-pnpm-lock.yaml",
            "pnpm-lock.yaml",
            "lockfile-only.json",
            "package-dependency-untraceable",
        ),
    ];
    for (patch_name, target_rel, report_name, expected_warning) in cases {
        let repo = crate::saved_fixture::materialize_path(&source.join("base"));
        git_cmd(repo.path(), &["init", "-b", "main"]);
        git_cmd(
            repo.path(),
            &["config", "user.email", "test@example.invalid"],
        );
        git_cmd(repo.path(), &["config", "user.name", "Test"]);
        git_cmd(repo.path(), &["add", "."]);
        git_cmd(repo.path(), &["commit", "-m", "base"]);
        let base_sha = git_cmd(repo.path(), &["rev-parse", "HEAD"]);
        std::fs::copy(patches.join(patch_name), repo.path().join(target_rel)).unwrap();
        git_cmd(repo.path(), &["add", "."]);
        git_cmd(repo.path(), &["commit", "-m", "head"]);

        let report_path = reports.join(report_name);
        let result = plan_vitest(
            repo.path(),
            &[
                "--base",
                &base_sha,
                "--changes",
                report_path.to_str().unwrap(),
            ],
        );
        assert_eq!(result["fallback_triggered"], true, "{patch_name}");
        assert_eq!(
            selected_files(&result),
            [
                "tests/avatar.test.ts",
                "tests/cart.test.ts",
                "tests/unrelated.test.ts",
                "tests/widget.test.ts",
            ],
            "{patch_name}"
        );
        assert!(
            warning_types(&result).contains(&expected_warning),
            "{patch_name}: expected `{expected_warning}` in {:?}",
            warning_types(&result)
        );
    }
}

#[test]
fn changes_report_changed_files_fallback_for_deleted_config_and_invalid_paths() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes-union");
    let head = source.join("head");
    let reports = source.join("reports");
    for (report_name, expected_fragment) in [
        ("deleted-file.json", "Deleted file"),
        ("config-changed.json", ".no-mistakes.yml"),
        ("outside-root.json", "outside the project root"),
    ] {
        let report_path = reports.join(report_name);
        let result = plan_vitest(&head, &["--changes", report_path.to_str().unwrap()]);
        assert_eq!(result["fallback_triggered"], true, "{report_name}");
        assert_eq!(selected_files(&result).len(), 4, "{report_name}");
        let reason = result["fallback_reason"].as_str().unwrap_or_default();
        assert!(
            reason.contains(expected_fragment),
            "{report_name}: expected `{expected_fragment}` in `{reason}`"
        );
    }
    let generic = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["tests", "plan", "--root"])
        .arg(&head)
        .args([
            "--format",
            "json",
            "--changes",
            reports.join("config-changed.json").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        generic.status.success(),
        "{}",
        String::from_utf8_lossy(&generic.stderr)
    );
    let generic_report: Value = serde_json::from_slice(&generic.stdout).unwrap();
    assert_eq!(generic_report["fallback_triggered"], true);
    let reason = generic_report["fallback_reason"]
        .as_str()
        .unwrap_or_default();
    assert!(
        reason.contains(".no-mistakes.yml"),
        "expected `.no-mistakes.yml` in `{reason}`"
    );
}
