use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn fixture(relative: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/test-runner-selector-grouping")
            .join(relative),
    )
}

fn plan(fixture_path: &Path, framework: &str) -> Value {
    let changed = if framework == "cargo" {
        "app/src/lib.rs"
    } else {
        "swift/Sources/App/Value.swift"
    };
    let mut args = vec![
        "test",
        "plan",
        framework,
        "--root",
        fixture_path.to_str().unwrap(),
    ];
    let config = (framework == "swift").then(|| fixture("swift/.no-mistakes.yml"));
    if let Some(config) = config.as_ref() {
        args.extend(["--config", config.to_str().unwrap()]);
    }
    args.extend(["--environment", "all", "--changed-file", changed, "--json"]);
    let output = run(&args);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(&output)).unwrap()
}

#[test]
fn cargo_execution_targets_keep_distinct_integration_selectors() {
    let plan = plan(&fixture("cargo"), "cargo");
    let targets = plan["execution_targets"].as_array().unwrap();
    assert_eq!(targets.len(), 2);
    assert!(targets.iter().any(|target| {
        target["runner"] == "cargo"
            && target["runner_args"] == serde_json::json!(["-p", "app", "--test", "a"])
            && target["test_files"] == serde_json::json!(["app/tests/a.rs"])
    }));
    assert!(targets.iter().any(|target| {
        target["runner"] == "cargo"
            && target["runner_args"] == serde_json::json!(["-p", "app", "--test", "b"])
            && target["test_files"] == serde_json::json!(["app/tests/b.rs"])
    }));
}

#[test]
fn swift_execution_targets_keep_distinct_test_filters() {
    let plan = plan(&fixture("."), "swift");
    let targets = plan["execution_targets"].as_array().unwrap();
    assert_eq!(targets.len(), 2);
    assert!(targets.iter().any(|target| {
        target["runner"] == "swift"
            && target["runner_args"]
                == serde_json::json!(["--package-path", "swift", "--filter", "AlphaTests"])
            && target["test_files"] == serde_json::json!(["swift/Tests/AlphaTests/Alpha.swift"])
    }));
    assert!(targets.iter().any(|target| {
        target["runner"] == "swift"
            && target["runner_args"]
                == serde_json::json!(["--package-path", "swift", "--filter", "BetaTests"])
            && target["test_files"] == serde_json::json!(["swift/Tests/BetaTests/Beta.swift"])
    }));
}
