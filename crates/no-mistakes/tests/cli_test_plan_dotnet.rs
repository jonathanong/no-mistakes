mod common;
#[path = "common/saved_fixture.rs"]
mod saved_fixture;
#[path = "cli_test_plan_dotnet/semantic.rs"]
mod semantic;

use common::{fixture, run, stdout};
use std::path::Path;
use std::process::Command;

fn semantic_fixture() -> tempfile::TempDir {
    saved_fixture::materialize("test-plan", "dotnet-semantic-plan/fixture")
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
            "base",
        ],
    );
}

fn semantic_plan(root: &Path, changed: &str, global_fallback: bool) -> serde_json::Value {
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--changed-file",
        changed,
        "--global-config-fallback",
        if global_fallback { "true" } else { "false" },
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(&output)).unwrap()
}

fn replace_from_change(root: &Path, change: &str, target: &str) {
    std::fs::copy(root.join("changes").join(change), root.join(target)).unwrap();
}

fn group<'a>(plan: &'a serde_json::Value, kind: &str) -> Vec<&'a str> {
    plan["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["type"] == kind)
        .map(|group| {
            group["selected"]
                .as_array()
                .unwrap()
                .iter()
                .map(|selected| selected.as_str().unwrap())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn test_plan_dotnet_uses_projects_and_dependency_graph() {
    let root = fixture("dotnet-test-plan");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/src/App/FeedService.cs",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec!["dotnet-clients/tests/App.Tests/FeedServiceTests.cs"]
    );
    let target = &plan["selected_tests"][0]["targets"][0];
    assert_eq!(target["runner"], "dotnet");
    assert_eq!(target["project"], "Company.App.Tests");
    assert_eq!(
        target["config"],
        "dotnet-clients/tests/App.Tests/App.Tests.csproj"
    );
    assert_eq!(
        target["base_command"],
        serde_json::json!(["dotnet", "test"])
    );
    assert_eq!(
        target["runner_args"],
        serde_json::json!([
            "dotnet-clients/tests/App.Tests/App.Tests.csproj",
            "--no-restore"
        ])
    );
    let via: Vec<&str> = plan["selected_tests"][0]["reasons"][0]["via"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(via, vec!["dotnet"]);
}

#[test]
fn test_plan_dotnet_commands_format_uses_project_level_command() {
    let root = fixture("dotnet-test-plan");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/src/App/FeedService.cs",
        "--format",
        "commands",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = stdout(&output);
    assert_eq!(
        stdout.trim(),
        "dotnet test dotnet-clients/tests/App.Tests/App.Tests.csproj --no-restore"
    );
    assert!(!stdout.contains("--filter"));
}

#[test]
fn test_plan_dotnet_direct_and_coverage_error() {
    let root = fixture("dotnet-test-plan");
    let direct = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/tests/App.Tests/FeedServiceTests.cs",
        "--json",
    ]);
    assert!(
        direct.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&direct.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&direct)).unwrap();
    assert_eq!(
        plan["groups"][0]["selected"],
        serde_json::json!(["dotnet-clients/tests/App.Tests/FeedServiceTests.cs"])
    );

    let coverage = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/src/App/FeedService.cs",
        "--environment",
        "coverage-only",
        "--json",
    ]);
    assert!(!coverage.status.success());
    assert!(String::from_utf8_lossy(&coverage.stderr)
        .contains("dotnet test plans do not support the coverage group"));
}

#[test]
fn test_plan_dotnet_falls_back_when_source_graph_is_unconfigured() {
    let root = fixture("dotnet-test-plan").join("dotnet-clients");
    // A source file without a configured graph can only fall back to the full
    // suite when the caller explicitly opts in to global fallback behavior.
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/App/FeedService.cs",
        "--global-config-fallback",
        "true",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("dotnet source impact"));

    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec![
            "tests/App.Tests/FeedServiceTests.cs",
            "tests/App.Tests/ParserEdgeCases.cs",
        ]
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["base_command"],
        serde_json::json!(["dotnet", "test"])
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["runner_args"],
        serde_json::json!(["--no-restore"])
    );
}

#[test]
fn test_plan_dotnet_project_file_without_baseline_warns_without_causal_selection() {
    let root = fixture("dotnet-scoped-fallback");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "clients/src/App/App.csproj",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert!(selected.is_empty(), "{plan:#}");
    assert!(plan["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["type"] == "dotnet-dependency-no-baseline"));
}

#[test]
fn test_plan_dotnet_deleted_solution_file_triggers_native_fallback() {
    let root = fixture("dotnet-test-plan");
    let diff_path = root.join("delete-solution.diff");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--diff",
        diff_path.to_str().unwrap(),
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec![
            "dotnet-clients/tests/App.Tests/FeedServiceTests.cs",
            "dotnet-clients/tests/App.Tests/ParserEdgeCases.cs",
        ]
    );
}

#[test]
fn test_plan_dotnet_multiple_project_files_without_baseline_warn_without_fallback() {
    let root = fixture("dotnet-scoped-fallback");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "clients/src/App/App.csproj",
        "--changed-file",
        "clients/src/Other/Other.csproj",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert!(selected.is_empty(), "{plan:#}");
    assert_eq!(
        plan["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|warning| warning["type"] == "dotnet-dependency-no-baseline")
            .count(),
        2
    );
}

#[test]
fn test_plan_dotnet_no_baseline_project_warning_preserves_existing_selections() {
    let root = fixture("dotnet-scoped-fallback");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "clients/tests/Other.Tests/OtherServiceTests.cs",
        "--changed-file",
        "clients/src/App/App.csproj",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec!["clients/tests/Other.Tests/OtherServiceTests.cs"]
    );
    assert!(plan["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["type"] == "dotnet-dependency-no-baseline"));
}

#[test]
fn test_plan_dotnet_solution_file_triggers_native_fallback() {
    let root = fixture("dotnet-test-plan");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/App.sln",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec![
            "dotnet-clients/tests/App.Tests/FeedServiceTests.cs",
            "dotnet-clients/tests/App.Tests/ParserEdgeCases.cs",
        ]
    );
}
