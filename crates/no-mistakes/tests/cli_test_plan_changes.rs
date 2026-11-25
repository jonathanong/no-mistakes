use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> tempfile::TempDir {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes");
    crate::saved_fixture::materialize_path(&root)
}

fn plan(root: &Path, jobs: &str, extra: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["tests", "plan", "vitest", "--root"])
        .arg(root)
        .args(["--format", "json", "--jobs", jobs])
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

fn summary(report: &Value) -> Value {
    json!({
        "selected": selected_files(report),
        "fallback": report["fallback_triggered"],
        "reason": report["fallback_reason"],
        "warnings": report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|warning| warning["type"].as_str())
            .collect::<Vec<_>>(),
    })
}

const ALL_TESTS: [&str; 3] = [
    "tests/browser.test.ts",
    "tests/example.test.ts",
    "tests/reader.test.ts",
];

#[test]
fn changes_matches_literal_ids_and_supersedes_only_the_configured_path_trigger() {
    let fixture = fixture();
    let cases: &[(&str, &[&str])] = &[
        ("changes_only", &["--changes", "reports/matched.json"]),
        (
            "both_inputs",
            &[
                "--changes",
                "reports/matched.json",
                "--changed-file",
                "data/example.json",
                "--changed-file",
                "src/example.ts",
            ],
        ),
        ("path_only", &["--changed-file", "data/example.json"]),
        ("without_input", &[]),
        ("targeted", &["--changes", "reports/targeted.json"]),
        ("missing_trigger", &["--changes", "reports/unmatched.json"]),
        ("empty_trigger", &["--changes", "reports/empty-target.json"]),
        (
            "empty_changes",
            &[
                "--changes",
                "reports/empty.json",
                "--changed-file",
                "data/example.json",
            ],
        ),
        (
            "path_and_changes",
            &[
                "--changed-file",
                "data/example.json",
                "--changes",
                "reports/matched.json",
            ],
        ),
    ];
    let mut summaries = serde_json::Map::new();
    for &(name, args) in cases {
        let report = plan(fixture.path(), "1", args);
        assert_eq!(report, plan(fixture.path(), "2", args), "{name}");
        summaries.insert(name.into(), summary(&report));
    }
    let matching = plan(fixture.path(), "1", cases[0].1);
    let reasons = matching["selected_tests"][0]["reasons"].as_array().unwrap();
    assert_eq!(
        reasons
            .iter()
            .map(|reason| reason["via"][0].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "changes:fixtures:example.a+b",
            "changes:fixtures:new.id",
            "changes:fixtures:old.id",
            "changes:text:site.title",
        ]
    );
    let targeted = plan(fixture.path(), "1", cases[4].1);
    assert_eq!(
        targeted["selected_tests"][0]["targets"][0]["project"],
        "browser"
    );
    let empty_target = plan(fixture.path(), "1", cases[6].1);
    assert_eq!(selected_files(&empty_target), ALL_TESTS);
    assert!(empty_target["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("has no discovered tests for vitest project `empty`"));
    assert_eq!(summaries["path_only"]["selected"], json!(ALL_TESTS));
    assert_eq!(
        summaries["path_and_changes"]["selected"],
        json!(["tests/reader.test.ts"])
    );
    insta::assert_snapshot!(
        "changes_routing",
        serde_json::to_string_pretty(&summaries).unwrap()
    );
}

#[test]
fn invalid_changes_input_runs_the_full_discovered_suite_despite_plan_limits() {
    let fixture = fixture();
    let mut summaries = serde_json::Map::new();
    for input in [
        "bad-category.json",
        "bad-id.json",
        "incomplete.json",
        "invalid.txt",
        "missing.json",
        "no-version.json",
        "no-complete.json",
        "no-unclassified.json",
        "unclassified.json",
        "unknown.json",
        "unsupported.json",
    ] {
        let path = format!("reports/{input}");
        let args = ["--changes", &path, "--limit-files", "1"];
        let report = plan(fixture.path(), "1", &args);
        assert_eq!(report, plan(fixture.path(), "2", &args), "{input}");
        assert_eq!(selected_files(&report), ALL_TESTS, "{input}");
        assert_eq!(report["fallback_triggered"], true, "{input}");
        assert_eq!(report["warnings"][0]["type"], "changes-fallback", "{input}");
        summaries.insert(input.into(), summary(&report));
    }
    insta::assert_snapshot!(
        "changes_safety",
        serde_json::to_string_pretty(&summaries).unwrap()
    );
}

#[test]
fn changes_rejects_direct_test_owner_mode() {
    let fixture = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["tests", "plan", "vitest", "--root"])
        .arg(fixture.path())
        .args(["--direct-test-owner", "--changes", "reports/matched.json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("--direct-test-owner") && error.contains("--changes"),
        "{error}"
    );
}

fn union_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes-union")
}

fn diff_json(base: &Path, head: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["changes", "diff", "--base-root"])
        .arg(base)
        .arg("--head-root")
        .arg(head)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
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
fn changes_unions_fixture_source_and_lockfile_without_unclassified_fallback() {
    let source = union_fixture_root();
    let patches = source.join("patches");
    let reports = source.join("reports");
    let mut summaries = serde_json::Map::new();

    // 1. Fixture only (--changes alone)
    let fix_head = crate::saved_fixture::materialize_path(&source.join("base"));
    std::fs::copy(
        patches.join("avatar.json"),
        fix_head.path().join("fixtures/avatar.json"),
    )
    .unwrap();
    let fix_diff = diff_json(&source.join("base"), fix_head.path());
    let expected_fix: Value = serde_json::from_str(
        &std::fs::read_to_string(reports.join("fixture-changed.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(fix_diff, expected_fix);
    let fix_plan = plan(
        fix_head.path(),
        "1",
        &[
            "--changes",
            reports.join("fixture-changed.json").to_str().unwrap(),
        ],
    );
    assert_eq!(selected_files(&fix_plan), ["tests/avatar.test.ts"]);
    assert_eq!(fix_plan["fallback_triggered"], false);
    summaries.insert("fixture_only".into(), summary(&fix_plan));

    // 2. Source only (--changes alone, routed via report changedFiles)
    let src_head = crate::saved_fixture::materialize_path(&source.join("base"));
    std::fs::copy(patches.join("cart.ts"), src_head.path().join("src/cart.ts")).unwrap();
    let src_diff = diff_json(&source.join("base"), src_head.path());
    let expected_src: Value =
        serde_json::from_str(&std::fs::read_to_string(reports.join("source-only.json")).unwrap())
            .unwrap();
    assert_eq!(src_diff, expected_src);
    let src_plan = plan(
        src_head.path(),
        "1",
        &[
            "--changes",
            reports.join("source-only.json").to_str().unwrap(),
        ],
    );
    assert_eq!(selected_files(&src_plan), ["tests/cart.test.ts"]);
    assert_eq!(src_plan["fallback_triggered"], false);
    summaries.insert("source_only".into(), summary(&src_plan));

    // 3. Both fixture and source (--changes alone)
    let both_diff = diff_json(&source.join("base"), &source.join("head"));
    let expected_both: Value =
        serde_json::from_str(&std::fs::read_to_string(reports.join("both.json")).unwrap()).unwrap();
    assert_eq!(both_diff, expected_both);
    let both_plan = plan(
        &source.join("head"),
        "1",
        &["--changes", "../reports/both.json"],
    );
    assert_eq!(
        selected_files(&both_plan),
        ["tests/avatar.test.ts", "tests/cart.test.ts"]
    );
    assert_eq!(both_plan["fallback_triggered"], false);
    summaries.insert("both".into(), summary(&both_plan));

    // 4. Both fixture, source, and lockfile bump (without --base -> fallback; with --base -> union)
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
    std::fs::copy(
        patches.join("avatar.json"),
        repo.path().join("fixtures/avatar.json"),
    )
    .unwrap();
    std::fs::copy(patches.join("cart.ts"), repo.path().join("src/cart.ts")).unwrap();
    std::fs::copy(
        patches.join("package.json"),
        repo.path().join("package.json"),
    )
    .unwrap();
    std::fs::copy(
        patches.join("pnpm-lock.yaml"),
        repo.path().join("pnpm-lock.yaml"),
    )
    .unwrap();
    git_cmd(repo.path(), &["add", "."]);
    git_cmd(repo.path(), &["commit", "-m", "head"]);
    let lock_diff = diff_json(&source.join("base"), repo.path());
    let expected_lock: Value = serde_json::from_str(
        &std::fs::read_to_string(reports.join("both-with-lockfile.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(lock_diff, expected_lock);
    let lock_no_base = plan(
        repo.path(),
        "1",
        &[
            "--changes",
            reports.join("both-with-lockfile.json").to_str().unwrap(),
        ],
    );
    assert_eq!(lock_no_base["fallback_triggered"], true);
    assert_eq!(
        selected_files(&lock_no_base),
        [
            "tests/avatar.test.ts",
            "tests/cart.test.ts",
            "tests/unrelated.test.ts",
            "tests/widget.test.ts",
        ]
    );
    summaries.insert("lockfile_without_base".into(), summary(&lock_no_base));

    let lock_plan = plan(
        repo.path(),
        "1",
        &[
            "--base",
            &base_sha,
            "--changes",
            reports.join("both-with-lockfile.json").to_str().unwrap(),
        ],
    );
    assert_eq!(
        selected_files(&lock_plan),
        [
            "tests/avatar.test.ts",
            "tests/cart.test.ts",
            "tests/widget.test.ts",
        ]
    );
    assert_eq!(lock_plan["fallback_triggered"], false);
    summaries.insert("both_with_lockfile".into(), summary(&lock_plan));

    // 5. Excluded path under a category's paths stays unclassified and falls back
    let excl_head = crate::saved_fixture::materialize_path(&source.join("base"));
    std::fs::copy(
        patches.join("ignored.json"),
        excl_head.path().join("fixtures/ignored.json"),
    )
    .unwrap();
    let excl_diff = diff_json(&source.join("base"), excl_head.path());
    let expected_excl: Value = serde_json::from_str(
        &std::fs::read_to_string(reports.join("excluded-unclassified.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(excl_diff, expected_excl);
    let excl_plan = plan(
        excl_head.path(),
        "1",
        &[
            "--changes",
            reports.join("excluded-unclassified.json").to_str().unwrap(),
        ],
    );
    assert_eq!(excl_plan["fallback_triggered"], true);
    assert_eq!(
        selected_files(&excl_plan),
        [
            "tests/avatar.test.ts",
            "tests/cart.test.ts",
            "tests/unrelated.test.ts",
            "tests/widget.test.ts",
        ]
    );
    summaries.insert("excluded_unclassified".into(), summary(&excl_plan));

    insta::assert_snapshot!(
        "changes_union_routing",
        serde_json::to_string_pretty(&summaries).unwrap()
    );
}

#[test]
fn changes_relative_path_resolves_against_root_like_config() {
    let source = union_fixture_root();
    let head = source.join("head");
    let other_cwd = tempfile::tempdir().unwrap();
    for changes_arg in [
        "../reports/fixture-changed.json",
        source
            .join("reports/fixture-changed.json")
            .to_str()
            .unwrap(),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .current_dir(other_cwd.path())
            .args(["tests", "plan", "vitest", "--root"])
            .arg(&head)
            .args([
                "--config",
                ".no-mistakes.yml",
                "--changes",
                changes_arg,
                "--changed-file",
                "fixtures/avatar.json",
                "--changed-file",
                "src/cart.ts",
                "--format",
                "json",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{changes_arg}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["fallback_triggered"], false,
            "{changes_arg}: {report:#}"
        );
        assert_eq!(
            selected_files(&report),
            ["tests/avatar.test.ts", "tests/cart.test.ts"],
            "{changes_arg}"
        );
    }
}
