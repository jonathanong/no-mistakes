#[path = "cli_test_plan_targeted_triggers/config_invalidation.rs"]
mod config_invalidation;
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
fn structured_trigger_config_rejects_empty_and_invalid_target_fields() {
    for (fixture_name, expected) in [
        ("invalid-target-empty-paths.yml", "paths must not be empty"),
        (
            "invalid-target-empty-targets.yml",
            "targets must not be empty",
        ),
        (
            "invalid-target-blank-target.yml",
            "targets[0] must not be blank",
        ),
        ("invalid-target-glob.yml", "contains invalid glob"),
    ] {
        let fixture = fixture();
        let root = fixture.path();
        copy_config(root, fixture_name);
        let output = plan(
            root,
            "vitest",
            &["--changed-file", "migrations/001.sql", "--json"],
        );
        assert!(!output.status.success(), "{fixture_name}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(".no-mistakes.yml.testPlan.vitest"),
            "{stderr}"
        );
        assert!(stderr.contains(expected), "{stderr}");
    }
}
