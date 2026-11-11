//! Normalized `check --format json` snapshots.
//!
//! Absolute paths, temp directories, process ids, timings, OS not-found text,
//! and CR line endings are rewritten before the snapshot is stored. A path the
//! CLI already made relative stays relative; an absolute path under the same
//! root loses its `<root>/` prefix so Windows matches that form. Paths outside
//! the fixture root and inside the checkout become `<repo>`. Arrays are sorted
//! so `--jobs` cannot change the committed file. Each finding with a file and
//! line also gets `sourceLine`, the trimmed text of that fixture line, so a
//! finding on the wrong line shows up in a snapshot diff. The redaction tokens are part
//! of the contract: dropping them would make Linux, macOS, and Windows
//! snapshots diverge.

#[path = "check_json_source_lines.rs"]
pub mod source_lines;

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
#[path = "check_json_normalize.rs"]
mod check_json_normalize;
pub use check_json_normalize::normalize_report;
#[cfg(test)]
use check_json_normalize::repo_roots;
use check_json_normalize::{normalize_value, root_variants, sort_value};

/// An empty configuration path selects automatic discovery, including fragments.
pub fn assert_saved_fixture(name: &str, cases: &[(&str, PathBuf, PathBuf)]) {
    assert_saved_fixture_with_env(name, cases, &[]);
}

/// Like `assert_saved_fixture`, with extra environment for the child only
/// (for example an empty `PATH` so an external tool is deterministically absent).
pub fn assert_saved_fixture_with_env(
    name: &str,
    cases: &[(&str, PathBuf, PathBuf)],
    envs: &[(&str, &str)],
) {
    assert_saved_fixture_with_env_and_args(name, cases, envs, &[]);
}

pub fn assert_saved_fixture_with_env_and_args(
    name: &str,
    cases: &[(&str, PathBuf, PathBuf)],
    envs: &[(&str, &str)],
    args: &[&str],
) {
    assert_saved_fixture_checked(name, cases, envs, false, args);
}

pub(super) fn assert_saved_fixture_checked(
    name: &str,
    cases: &[(&str, PathBuf, PathBuf)],
    envs: &[(&str, &str)],
    clean: bool,
    args: &[&str],
) {
    let mut document = serde_json::Map::new();
    for (label, root, config) in cases {
        let jobs_one = run_normalized(root, config, 1, envs, args);
        let jobs_two = run_normalized(root, config, 2, envs, args);
        assert_eq!(
            jobs_one, jobs_two,
            "{label} changed between --jobs 1 and --jobs 2"
        );
        if clean {
            assert_eq!(jobs_one["exitCode"], 0, "{label} must be clean: {jobs_one}");
            for category in [
                "rules",
                "codebase",
                "integration",
                "queues",
                "react",
                "advisories",
                "warnings",
            ] {
                assert!(
                    jobs_one["report"][category]
                        .as_array()
                        .is_some_and(Vec::is_empty),
                    "{label} must have no {category}: {jobs_one}"
                );
            }
        }
        document.insert((*label).to_string(), jobs_one);
    }
    let mut value = Value::Object(document);
    sort_value(&mut value);
    let pretty = format!("{}\n", serde_json::to_string_pretty(&value).unwrap());
    insta::with_settings!({
        snapshot_path => "snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, pretty);
    });
}

fn run_normalized(
    root: &Path,
    config: &Path,
    jobs: usize,
    envs: &[(&str, &str)],
    args: &[&str],
) -> Value {
    let jobs_arg = jobs.to_string();
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .current_dir(root)
        .env_remove("RAYON_NUM_THREADS")
        .envs(envs.iter().copied())
        .args(["--jobs", &jobs_arg, "check", "--root"])
        .arg(root)
        .args(args);
    if !config.as_os_str().is_empty() {
        command.arg("--config").arg(config);
    }
    let output = command
        .args(["--format", "json"])
        .output()
        .unwrap_or_else(|error| panic!("run check: {error}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let exit_code = output.status.code().unwrap_or(-1);
    let report = serde_json::from_str::<Value>(&stdout).unwrap_or_else(|error| {
        panic!(
            "check JSON ({error}), exit {exit_code}\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let mut value = serde_json::json!({
        "exitCode": exit_code,
        "report": report,
    });
    normalize_value(&mut value, &root_variants(root));
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = manifest.ancestors().nth(2).unwrap_or(manifest);
    source_lines::add_source_lines(&mut value, root, repo);
    sort_value(&mut value);
    value
}

pub fn rules_path(rule: &str, kind: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(rule)
            .join(kind)
            .join(name),
    )
}

#[test]
fn normalization_keeps_repo_paths_and_cr_line_endings_stable() {
    let repo = repo_roots()
        .iter()
        .max_by_key(|item| item.len())
        .expect("repo root");
    let config = format!("{repo}/test-cases/rules/example.yml");
    let raw = serde_json::json!({
        "file": config,
        "message": "line\r\nnext\r",
    })
    .to_string();
    let normalized = normalize_report(&raw, &[]);
    assert!(
        normalized.contains("<repo>/test-cases/rules/example.yml"),
        "{normalized}"
    );
    assert!(!normalized.contains("<temp>"), "{normalized}");
    assert!(normalized.contains("line\\nnext\\n"), "{normalized}");
}

#[test]
fn normalization_collapses_machine_specific_json() {
    let shuffled = r#"{
      "warnings": ["/tmp/xyz/out.json", "pid: 4242 took 12.5ms", "/private/tmp/abc/file.yml"],
      "rules": [{"message": "3 lines", "file": "/repo/fixture/fail/CLAUDE.md", "rule": "agents-md-max-size", "line": 1}],
      "timings": [{"pid": 99, "name": "rules", "elapsedMs": 3.25}]
    }"#;
    let windows = r#"{
      "rules": [{"rule": "agents-md-max-size", "file": "/repo/fixture/fail/CLAUDE.md", "line": 1, "message": "3 lines"}],
      "timings": [{"elapsedMs": 3.25, "pid": 99, "name": "rules"}],
      "warnings": ["C:\\Users\\runner\\AppData\\Local\\Temp\\nmabc\\CLAUDE.md", "/var/folders/ab/cd/T/xyz/out.json", "pid: 4242 took 12.5ms"]
    }"#;
    let first = normalize_report(shuffled, &["/repo/fixture/fail".to_string()]);
    let second = normalize_report(windows, &["/repo/fixture/fail".to_string()]);
    assert_eq!(
        first, second,
        "posix and windows temp paths must normalize together"
    );
    let value: Value = serde_json::from_str(&first).unwrap();
    assert_eq!(value["rules"][0]["file"], "CLAUDE.md");
    assert_eq!(value["timings"][0]["elapsedMs"], "<timing>");
    assert_eq!(value["timings"][0]["pid"], "<pid>");
    assert!(first.contains("pid <pid> took <timing>"));
    assert!(!first.contains("/tmp"));
    assert!(!first.contains("AppData"));
    assert!(!first.contains("var/folders"));
    assert!(!first.contains("4242"));
    assert!(!second.contains("AppData"));
    assert!(!second.contains("var/folders"));
    assert!(second.contains("<temp>"));
    assert!(second.contains("CLAUDE.md"));
    assert!(!second.contains("<root>/"));
    let kept = normalize_report(
        r#"{"file":"/repo/tmp/keep.yml","other":"/tmp/real/file.yml"}"#,
        &["/repo".to_string()],
    );
    assert!(kept.contains("tmp/keep.yml"), "{kept}");
    assert!(kept.contains("<temp>"), "{kept}");
    assert!(!kept.contains("/tmp/real"), "{kept}");
    assert!(!kept.contains("<root>/"), "{kept}");
    let windows_config = normalize_report(
        r#"{"file":"D:\\repo\\fixture\\snapshot-config.yml"}"#,
        &["D:/repo/fixture".to_string()],
    );
    let linux_config = normalize_report(
        r#"{"file":"snapshot-config.yml"}"#,
        &["/repo/fixture".to_string()],
    );
    assert_eq!(windows_config, linux_config, "{windows_config}");
    let missing = normalize_report(
        r#"{"message":"No such file or directory (os error 2)"}"#,
        &[],
    );
    let missing_file = normalize_report(
        r#"{"message":"The system cannot find the file specified. (os error 2)"}"#,
        &[],
    );
    let missing_path = normalize_report(
        r#"{"message":"The system cannot find the path specified. (os error 3)"}"#,
        &[],
    );
    assert_eq!(missing, missing_file);
    assert_eq!(missing, missing_path);
    assert!(missing.contains("<io-error>"));
    assert!(!missing.contains("os error"));
    let quoted = normalize_report(r#"{"message":"[tools\r\n"}"#, &[]);
    assert!(quoted.contains("[tools\\n"), "{quoted}");
    assert!(!quoted.contains("\\r"), "{quoted}");
}

#[test]
fn agents_md_max_size_json_matches_snapshot() {
    assert_saved_fixture(
        "agents_md_max_size",
        &[
            (
                "pass",
                rules_path("agents-md-max-size", "fixture", "pass"),
                rules_path("agents-md-max-size", "configs", "pass.yml"),
            ),
            (
                "fail",
                rules_path("agents-md-max-size", "fixture", "fail"),
                rules_path("agents-md-max-size", "configs", "fail.yml"),
            ),
            (
                "disabled",
                rules_path("agents-md-max-size", "fixture", "fail"),
                rules_path("agents-md-max-size", "configs", "disabled.yml"),
            ),
        ],
    );
}
