use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn case(path: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(path),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn ci_impact_lists_triggered_workflows() {
    let root = case("ci-graph/triggers");
    let output = run(&[
        "ci",
        "impact",
        "src/app.ts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("\"workflows\""));
    assert!(out.contains(".github/workflows/paths.yml"));
}

#[test]
fn ci_env_lists_locations() {
    let root = case("ci-graph/env");
    let output = run(&[
        "ci",
        "env",
        "CIGRAPH_WORKFLOW_VAR",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "paths",
    ]);
    assert!(output.status.success());
    assert!(stdout(&output).contains(".github/workflows/env.yml"));
}

#[test]
fn impacted_checks_lists_commands() {
    let root = case("impacted-checks/basic");
    let output = run(&[
        "impacted-checks",
        "src/foo.ts",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "paths",
    ]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("vitest --project unit"));
}
