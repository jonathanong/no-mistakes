//! Normalized `check --format json` snapshots.
//!
//! Absolute paths, temp directories, process ids, and timings are rewritten
//! before the snapshot is stored. Arrays are sorted so `--jobs` cannot change
//! the committed file. The redaction tokens are part of the contract: dropping
//! them would make Linux, macOS, and Windows snapshots diverge.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub fn assert_saved_fixture(name: &str, cases: &[(&str, PathBuf, PathBuf)]) {
    let mut document = serde_json::Map::new();
    for (label, root, config) in cases {
        let jobs_one = run_normalized(root, config, 1);
        let jobs_two = run_normalized(root, config, 2);
        assert_eq!(
            jobs_one, jobs_two,
            "{label} changed between --jobs 1 and --jobs 2"
        );
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

fn run_normalized(root: &Path, config: &Path, jobs: usize) -> Value {
    let jobs_arg = jobs.to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .current_dir(root)
        .env_remove("RAYON_NUM_THREADS")
        .args(["--jobs", &jobs_arg, "check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
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
    sort_value(&mut value);
    value
}

fn root_variants(root: &Path) -> Vec<String> {
    let mut variants = vec![slash(root)];
    if let Ok(canonical) = root.canonicalize() {
        variants.push(slash(&canonical));
    }
    variants.sort_by_key(|item| std::cmp::Reverse(item.len()));
    variants.dedup();
    variants
}

pub fn normalize_report(raw: &str, roots: &[String]) -> String {
    let mut value = serde_json::from_str::<Value>(raw).expect("json");
    normalize_value(&mut value, roots);
    sort_value(&mut value);
    format!("{}\n", serde_json::to_string_pretty(&value).unwrap())
}

fn normalize_value(value: &mut Value, roots: &[String]) {
    match value {
        Value::Array(items) => {
            for item in items {
                normalize_value(item, roots);
            }
        }
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                let Some(child) = map.get_mut(&key) else {
                    continue;
                };
                let normalized = key.to_ascii_lowercase().replace('-', "_");
                if is_scalar(child) && is_pid_key(&normalized) {
                    *child = Value::String("<pid>".to_string());
                    continue;
                }
                if is_scalar(child) && is_timing_key(&normalized) {
                    *child = Value::String("<timing>".to_string());
                    continue;
                }
                normalize_value(child, roots);
            }
        }
        Value::String(text) => *text = normalize_text(text, roots),
        _ => {}
    }
}

fn is_scalar(value: &Value) -> bool {
    !value.is_array() && !value.is_object()
}

fn is_pid_key(key: &str) -> bool {
    matches!(key, "pid" | "ppid" | "processid" | "process_id") || key.ends_with("_pid")
}

fn is_timing_key(key: &str) -> bool {
    matches!(
        key,
        "elapsed"
            | "elapsed_ms"
            | "elapsedms"
            | "duration"
            | "duration_ms"
            | "durationms"
            | "duration_ns"
            | "durationns"
            | "started_ms"
            | "startedms"
            | "time_ms"
            | "timems"
            | "millis"
            | "milliseconds"
            | "micros"
            | "microseconds"
            | "nanos"
            | "timing"
            | "timings"
    )
}

fn normalize_text(text: &str, roots: &[String]) -> String {
    let mut text = text.replace('\\', "/");
    for root in roots {
        let root = root.trim_end_matches('/');
        if !root.is_empty() {
            text = text.replace(root, "<root>");
        }
    }
    let text = redact_temp(&text);
    let text = pids().replace_all(&text, "$1 <pid>");
    timings().replace_all(&text, "<timing>").into_owned()
}

fn redact_temp(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for found in temp_paths().find_iter(text) {
        let start = found.start();
        let inside_token = start > 0
            && text[..start]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '>');
        if inside_token {
            continue;
        }
        out.push_str(&text[last..start]);
        out.push_str("<temp>");
        last = found.end();
    }
    out.push_str(&text[last..]);
    out
}

fn temp_paths() -> &'static regex::Regex {
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(concat!(
            r"(?i)(?:[A-Za-z]:)?(?:/private/var/folders|/var/folders)/[^/\s]+/[^/\s]+/T(?:/\S*)?",
            r"|(?:[A-Za-z]:)?/Users/[^/\s]+/AppData/Local/Temp(?:/\S*)?",
            r"|(?:/private/tmp|/var/tmp|/tmp)(?:/\S*)?",
        ))
        .expect("temp path pattern")
    })
}

fn pids() -> &'static regex::Regex {
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| regex::Regex::new(r"(?i)\b(pid|ppid)\s*[:=]\s*\d+\b").expect("pid pattern"))
}

fn timings() -> &'static regex::Regex {
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(r"(?i)\b\d+(?:\.\d+)?\s*(?:ms|us|µs|ns)\b").expect("timing pattern")
    })
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sort_value(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items.iter_mut() {
                sort_value(item);
            }
            items.sort_by_key(|left| left.to_string());
        }
        Value::Object(map) => {
            for child in map.values_mut() {
                sort_value(child);
            }
            let pairs = std::mem::take(map);
            let mut pairs: Vec<_> = pairs.into_iter().collect();
            pairs.sort_by(|left, right| left.0.cmp(&right.0));
            for (key, child) in pairs {
                map.insert(key, child);
            }
        }
        _ => {}
    }
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
    assert_eq!(value["rules"][0]["file"], "<root>/CLAUDE.md");
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
    assert!(second.contains("<root>/CLAUDE.md"));
    let kept = normalize_report(
        r#"{"file":"/repo/tmp/keep.yml","other":"/tmp/real/file.yml"}"#,
        &["/repo".to_string()],
    );
    assert!(kept.contains("<root>/tmp/keep.yml"), "{kept}");
    assert!(kept.contains("<temp>"), "{kept}");
    assert!(!kept.contains("/tmp/real"), "{kept}");
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
