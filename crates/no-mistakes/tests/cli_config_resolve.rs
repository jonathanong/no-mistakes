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
    let text = stdout(&output);
    assert!(
        !text.trim_end().contains('\n'),
        "config resolve JSON must stay compact"
    );
    let report: serde_json::Value = serde_json::from_str(&text).unwrap();
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
    assert_eq!(
        report["fullSuiteTriggers"][0]["triggers"][0]["includeChangedTests"],
        false
    );
}

fn resolve_in(dir: &PathBuf, extra: &[&str]) -> Output {
    Command::new(bin())
        .current_dir(dir)
        .args(["config", "resolve", "--root", "."])
        .args(extra)
        .output()
        .expect("no-mistakes should run")
}

#[test]
fn config_resolve_accepts_platform_exports_with_relative_dot_root() {
    let dir = repository_fixture("config", "relative-root-platform-exports");
    let output = resolve_in(&dir, &[]);

    assert!(
        output.status.success(),
        "relative root should validate: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn config_resolve_rejects_platform_exports_escaping_relative_dot_root() {
    let dir = repository_fixture("config", "relative-root-platform-exports");
    let output = resolve_in(&dir, &["--config", "escape.yml"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("file must stay within the root"), "{stderr}");
}
