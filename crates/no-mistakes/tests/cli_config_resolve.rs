use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn repository_fixture(category: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(category)
            .join(name),
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
fn config_resolve_prints_vitest_and_per_framework_triggers() {
    let root = repository_fixture("config", "named-full-suite-triggers");
    let output = run(&["config", "resolve", "--root", root.to_str().unwrap()]);

    assert!(
        output.status.success(),
        "config resolve should succeed: {}",
        stdout(&output)
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let vitest_names: Vec<&str> = report["vitestFullSuiteTriggers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|trigger| trigger["name"].as_str())
        .collect();
    assert!(vitest_names.contains(&"postgres-resources"));
    assert_eq!(
        report["fullSuiteTriggers"][0]["framework"].as_str(),
        Some("vitest")
    );
    assert_eq!(
        report["fullSuiteTriggers"][0]["triggers"][0]["name"].as_str(),
        Some("postgres-resources")
    );
}
