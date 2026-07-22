#[path = "cli_test_plan_targeted_triggers/group_order.rs"]
mod group_order;
#[path = "common/saved_fixture.rs"]
mod saved_fixture;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture() -> tempfile::TempDir {
    saved_fixture::materialize("test-plan", "target-scoped-triggers")
}

fn plan(root: &Path, framework: &str, args: &[&str]) -> Output {
    Command::new(bin())
        .args(["tests", "plan", framework, "--root"])
        .arg(root)
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn json(output: &Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(output)).unwrap()
}

fn copy_config(root: &Path, name: &str) {
    std::fs::copy(
        root.join("configs").join(name),
        root.join(".no-mistakes.yml"),
    )
    .unwrap();
}

fn git(root: &Path, args: &[&str]) {
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
}

fn git_init(root: &Path) {
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["add", "-A"]);
    git(
        root,
        &[
            "-c",
            "user.name=no-mistakes tests",
            "-c",
            "user.email=no-mistakes-tests@example.invalid",
            "commit",
            "-q",
            "-m",
            "initial",
        ],
    );
}

fn git_commit(root: &Path, message: &str) {
    git(root, &["add", "-A"]);
    git(
        root,
        &[
            "-c",
            "user.name=no-mistakes tests",
            "-c",
            "user.email=no-mistakes-tests@example.invalid",
            "commit",
            "-q",
            "-m",
            message,
        ],
    );
}

fn selected_files(plan: &serde_json::Value) -> Vec<&str> {
    plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect()
}

#[test]
fn structured_trigger_selects_only_the_target_project_in_every_cli_format() {
    let fixture = fixture();
    let root = fixture.path();

    let output = plan(
        root,
        "vitest",
        &["--changed-file", "migrations/001.sql", "--json"],
    );
    let report = json(&output);
    assert_eq!(selected_files(&report), vec!["src/db/db.test.ts"]);
    assert_eq!(report["fallback_triggered"], false);
    assert!(report["fallback_reason"].is_null());
    assert_eq!(
        report["selected_tests"][0]["reasons"][0]["via"],
        serde_json::json!(["configured-trigger"])
    );
    assert_eq!(
        report["selected_tests"][0]["targets"][0]["project"],
        "database"
    );

    let paths = plan(
        root,
        "vitest",
        &["--changed-file", "migrations/001.sql", "--format", "paths"],
    );
    assert!(paths.status.success());
    assert_eq!(stdout(&paths), "src/db/db.test.ts\n");

    let commands = plan(
        root,
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--format",
            "commands",
        ],
    );
    assert!(commands.status.success());
    assert_eq!(
        stdout(&commands),
        "vitest --config vitest.config.mts --project database src/db/db.test.ts\n"
    );

    // A configured trigger contributes to the dependencies group; it must not
    // replace independently selected direct or graph candidates.
    let mixed = json(&plan(
        root,
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--changed-file",
            "src/web/web.test.ts",
            "--json",
        ],
    ));
    assert_eq!(
        selected_files(&mixed),
        vec!["src/db/db.test.ts", "src/web/web.test.ts"]
    );
}

#[test]
fn structured_triggers_union_targets_and_filter_shared_test_commands() {
    let fixture = fixture();
    let root = fixture.path();

    let union = json(&plan(
        root,
        "vitest",
        &["--changed-file", "migrations/union.sql", "--json"],
    ));
    assert_eq!(
        selected_files(&union),
        vec!["src/db/db.test.ts", "src/web/web.test.ts"]
    );
    assert_eq!(union["fallback_triggered"], false);

    let shared = json(&plan(
        root,
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--environment",
            "shared",
            "--json",
        ],
    ));
    assert_eq!(
        selected_files(&shared),
        vec!["src/db/db.test.ts", "src/shared.test.ts"]
    );
    let shared_targets = shared["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|test| test["test_file"] == "src/shared.test.ts")
        .unwrap()["targets"]
        .as_array()
        .unwrap();
    assert_eq!(shared_targets.len(), 1);
    assert_eq!(shared_targets[0]["project"], "database");

    let limited = json(&plan(
        root,
        "vitest",
        &[
            "--changed-file",
            "migrations/union.sql",
            "--environment",
            "limited",
            "--json",
        ],
    ));
    assert_eq!(limited["selected_tests"].as_array().unwrap().len(), 1);

    let exhausted = json(&plan(
        root,
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--changed-file",
            "src/web/web.test.ts",
            "--environment",
            "limited",
            "--json",
        ],
    ));
    assert_eq!(
        exhausted["groups"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|group| group["type"] == "dependencies")
            .count(),
        1
    );
}

#[test]
fn structured_trigger_negation_is_ordered_and_can_reinclude() {
    let fixture = fixture();
    let root = fixture.path();

    let excluded = json(&plan(
        root,
        "vitest",
        &["--changed-file", "migrations/ignored/skip.sql", "--json"],
    ));
    assert!(excluded["selected_tests"].as_array().unwrap().is_empty());
    assert_eq!(excluded["fallback_triggered"], false);

    let reinclude = json(&plan(
        root,
        "vitest",
        &[
            "--changed-file",
            "migrations/ignored/reinclude.sql",
            "--json",
        ],
    ));
    assert_eq!(selected_files(&reinclude), vec!["src/db/db.test.ts"]);
}

#[test]
fn target_trigger_does_not_enable_an_omitted_graph_dependency_group() {
    let fixture = fixture();
    let root = fixture.path();
    copy_config(root, "direct-only.yml");

    let report = json(&plan(
        root,
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--changed-file",
            "src/web/helper.ts",
            "--environment",
            "direct-only",
            "--json",
        ],
    ));
    assert_eq!(
        selected_files(&report),
        vec!["src/db/db.test.ts", "src/shared.test.ts"]
    );
    assert!(!selected_files(&report).contains(&"src/web/web.test.ts"));
    assert_eq!(
        report["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|group| group["type"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["direct", "dependencies"]
    );
}

#[test]
fn legacy_trigger_still_falls_back_to_the_framework_suite() {
    let fixture = fixture();
    let root = fixture.path();
    copy_config(root, "legacy.yml");

    let report = json(&plan(
        root,
        "vitest",
        &["--changed-file", "migrations/001.sql", "--json"],
    ));
    assert_eq!(report["fallback_triggered"], true);
    assert_eq!(
        selected_files(&report),
        vec!["src/db/db.test.ts", "src/web/web.test.ts"]
    );
}

#[test]
fn structured_trigger_rejects_unknown_and_ambiguous_runner_targets() {
    let fixture = fixture();
    let root = fixture.path();
    copy_config(root, "unknown-target.yml");
    let unknown = plan(
        root,
        "vitest",
        &["--changed-file", "migrations/001.sql", "--json"],
    );
    assert!(!unknown.status.success());
    let stderr = String::from_utf8_lossy(&unknown.stderr);
    assert!(stderr.contains(".no-mistakes.yml"), "{stderr}");
    assert!(stderr.contains("missing"), "{stderr}");

    let invalid_before_fallback = plan(
        root,
        "vitest",
        &[
            "--changed-file",
            ".no-mistakes.yml",
            "--global-config-fallback",
            "true",
            "--json",
        ],
    );
    assert!(!invalid_before_fallback.status.success());
    assert!(String::from_utf8_lossy(&invalid_before_fallback.stderr).contains("missing"));

    copy_config(root, "ambiguous-target.yml");
    let ambiguous = plan(
        root,
        "vitest",
        &["--changed-file", "migrations/001.sql", "--json"],
    );
    assert!(!ambiguous.status.success());
    let stderr = String::from_utf8_lossy(&ambiguous.stderr);
    assert!(stderr.contains("database"), "{stderr}");
    assert!(stderr.contains("ambiguous"), "{stderr}");
    assert!(stderr.contains("vitest.database-a.mts"), "{stderr}");
    assert!(stderr.contains("vitest.database-b.mts"), "{stderr}");
}

#[test]
fn revision_config_changes_invalidate_only_the_affected_framework() {
    for (variant, vitest_fallback, playwright_fallback) in [
        ("format-only.yml", false, false),
        ("vitest-change.yml", true, false),
        ("playwright-change.yml", false, true),
        ("project-change.yml", true, false),
    ] {
        let fixture = fixture();
        let root = fixture.path();
        git_init(root);
        copy_config(root, variant);
        git_commit(root, variant);

        let vitest = json(&plan(
            root,
            "vitest",
            &["--base", "HEAD~1", "--head", "HEAD", "--json"],
        ));
        let playwright = json(&plan(
            root,
            "playwright",
            &["--base", "HEAD~1", "--head", "HEAD", "--json"],
        ));
        assert_eq!(
            vitest["fallback_triggered"], vitest_fallback,
            "{variant}: {vitest}"
        );
        assert_eq!(
            playwright["fallback_triggered"], playwright_fallback,
            "{variant}: {playwright}"
        );
    }
}

#[test]
fn inline_diff_compares_complete_framework_configuration() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    copy_config(root, "vitest-change.yml");

    let command = "git diff -- .no-mistakes.yml";
    let vitest = json(&plan(
        root,
        "vitest",
        &["--diff-command", command, "--json"],
    ));
    let playwright = json(&plan(
        root,
        "playwright",
        &["--diff-command", command, "--json"],
    ));
    assert_eq!(vitest["fallback_triggered"], true);
    assert_eq!(playwright["fallback_triggered"], false);
}

#[test]
fn inline_diff_handles_added_deleted_and_modified_renamed_configs() {
    let added_fixture = fixture();
    let added_root = added_fixture.path();
    std::fs::remove_file(added_root.join(".no-mistakes.yml")).unwrap();
    git_init(added_root);
    copy_config(added_root, "format-only.yml");
    git(added_root, &["add", ".no-mistakes.yml"]);
    let added = json(&plan(
        added_root,
        "vitest",
        &[
            "--diff-command",
            "git diff --cached -- .no-mistakes.yml",
            "--json",
        ],
    ));
    assert_eq!(added["fallback_triggered"], true);

    let deleted_fixture = fixture();
    let deleted_root = deleted_fixture.path();
    git_init(deleted_root);
    std::fs::remove_file(deleted_root.join(".no-mistakes.yml")).unwrap();
    let deleted = json(&plan(
        deleted_root,
        "vitest",
        &[
            "--diff-command",
            "git diff -- .no-mistakes.yml",
            "--global-config-fallback",
            "true",
            "--json",
        ],
    ));
    assert_eq!(deleted["fallback_triggered"], true);

    let renamed_fixture = fixture();
    let renamed_root = renamed_fixture.path();
    git_init(renamed_root);
    std::fs::remove_file(renamed_root.join(".no-mistakes.yml")).unwrap();
    std::fs::copy(
        renamed_root.join("configs/vitest-change.yml"),
        renamed_root.join(".no-mistakes.yaml"),
    )
    .unwrap();
    git(renamed_root, &["add", "-A"]);
    let command = "git diff --cached --find-renames -- .no-mistakes.yml .no-mistakes.yaml";
    let vitest = json(&plan(
        renamed_root,
        "vitest",
        &["--diff-command", command, "--json"],
    ));
    let playwright = json(&plan(
        renamed_root,
        "playwright",
        &["--diff-command", command, "--json"],
    ));
    assert_eq!(vitest["fallback_triggered"], true);
    assert_eq!(playwright["fallback_triggered"], false);
}

#[test]
fn changed_file_only_and_unparseable_history_keep_conservative_fallback() {
    let fixture = fixture();
    let root = fixture.path();

    let changed_only = json(&plan(
        root,
        "vitest",
        &["--changed-file", ".no-mistakes.yml", "--json"],
    ));
    assert_eq!(changed_only["fallback_triggered"], true);

    copy_config(root, "malformed.yml");
    git_init(root);
    copy_config(root, "format-only.yml");
    git_commit(root, "repair config");
    let unreadable = json(&plan(
        root,
        "vitest",
        &["--base", "HEAD~1", "--head", "HEAD", "--json"],
    ));
    assert_eq!(unreadable["fallback_triggered"], true);
}

#[test]
fn renamed_config_with_equal_values_does_not_invalidate_frameworks() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    std::fs::rename(
        root.join(".no-mistakes.yml"),
        root.join(".no-mistakes.yaml"),
    )
    .unwrap();
    git_commit(root, "rename config extension");

    for framework in ["vitest", "playwright"] {
        let report = json(&plan(
            root,
            framework,
            &["--base", "HEAD~1", "--head", "HEAD", "--json"],
        ));
        assert_eq!(report["fallback_triggered"], false, "{framework}: {report}");
    }
}
