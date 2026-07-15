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

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf8")
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

#[test]
fn impacted_checks_multi_file_json_covers_every_configured_framework() {
    let root = case("impacted-checks/multi-framework");
    let output = run(&[
        "impacted-checks",
        "src/value.ts",
        "dotnet/src/App/Value.cs",
        "swift/App/Sources/App/Value.swift",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report["checks"].as_array().unwrap().len(), 4);
    assert_eq!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|check| check["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["dotnet", "playwright", "swift", "vitest"]
    );
    assert_eq!(report["fallback_triggered"], false);
}

#[test]
fn impacted_checks_timings_preserve_json_and_report_ordered_phases() {
    let root = case("impacted-checks/multi-framework");
    let base_args = [
        "impacted-checks",
        "src/value.ts",
        "dotnet/src/App/Value.cs",
        "swift/App/Sources/App/Value.swift",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
    ];
    let without_timings = run(&base_args);
    let mut timed_args = base_args.to_vec();
    timed_args.push("--timings");
    let with_timings = run(&timed_args);

    assert!(without_timings.status.success());
    assert!(with_timings.status.success(), "{}", stderr(&with_timings));
    assert_eq!(with_timings.stdout, without_timings.stdout);
    let plain: serde_json::Value = serde_json::from_str(&stdout(&without_timings)).unwrap();
    let timed: serde_json::Value = serde_json::from_str(&stdout(&with_timings)).unwrap();
    assert_eq!(timed, plain);
    assert!(timed.get("timings").is_none());

    let err = stderr(&with_timings);
    let completed_phases = err
        .lines()
        .filter_map(|line| {
            let line = line.strip_prefix("[timing] ")?;
            let (phase, duration) = line.split_once(": ")?;
            duration
                .strip_suffix("ms")?
                .parse::<f64>()
                .ok()
                .map(|_| phase)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        completed_phases,
        vec![
            "prepare",
            "discover.dotnet",
            "discover.vitest",
            "discover.playwright",
            "discover.swift",
            "graph",
            "select.dotnet",
            "select.vitest",
            "select.playwright",
            "select.swift",
            "generic-checks",
            "total",
        ],
        "completed phases should stay deterministic: {err}"
    );
    assert!(
        err.lines().any(|line| {
            line.strip_prefix("[timing] graph: ")
                .and_then(|duration| duration.strip_suffix("ms"))
                .is_some_and(|duration| duration.parse::<f64>().is_ok())
        }),
        "graph completion should include a duration: {err}"
    );
}

#[test]
fn impacted_checks_timings_omit_lazy_graph_when_every_framework_runs_all() {
    let root = case("impacted-checks/multi-framework");
    let config = root.join("all.no-mistakes.yml");
    let output = run(&[
        "impacted-checks",
        "src/value.ts",
        "--root",
        root.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--format",
        "json",
        "--timings",
    ]);

    assert!(output.status.success(), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(
        !err.lines().any(|line| line.starts_with("[timing] graph: ")),
        "{err}"
    );
    assert!(
        err.lines()
            .any(|line| line.starts_with("[timing] select.dotnet: ")),
        "{err}"
    );
}

#[test]
fn impacted_checks_timings_report_phase_and_total_before_actionable_error() {
    let root = case("impacted-checks/multi-framework");
    let config = root.join("invalid.no-mistakes.yml");
    let output = run(&[
        "impacted-checks",
        "src/value.ts",
        "--root",
        root.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--format",
        "json",
        "--timings",
    ]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let err = stderr(&output);
    assert!(err.contains("[timing] prepare: "), "{err}");
    assert!(err.contains("[timing] total: "), "{err}");
    assert!(err.contains("invalid.no-mistakes.yml"), "{err}");
}
