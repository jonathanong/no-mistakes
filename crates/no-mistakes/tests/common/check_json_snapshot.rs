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
mod source_lines;

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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
    let mut document = serde_json::Map::new();
    for (label, root, config) in cases {
        let jobs_one = run_normalized(root, config, 1, envs);
        let jobs_two = run_normalized(root, config, 2, envs);
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

fn run_normalized(root: &Path, config: &Path, jobs: usize, envs: &[(&str, &str)]) -> Value {
    let jobs_arg = jobs.to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .current_dir(root)
        .env_remove("RAYON_NUM_THREADS")
        .envs(envs.iter().copied())
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
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = manifest.ancestors().nth(2).unwrap_or(manifest);
    source_lines::add_source_lines(&mut value, root, repo);
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
    // Windows checkouts embed CR in SQL that the report quotes. Fold it before
    // any path compare so the committed snapshot stays LF.
    let mut text = text.replace("\r\n", "\n").replace('\r', "\n");
    text = text.replace('\\', "/");
    for root in roots {
        let root = root.trim_end_matches('/');
        if !root.is_empty() {
            text = text.replace(root, "<root>");
        }
    }
    // Config findings sit outside the fixture root. Replace the checkout first:
    // a worktree under /tmp would otherwise become <temp>.
    for root in repo_roots() {
        text = replace_path_prefix(&text, root, "<repo>");
    }
    // Windows `strip_prefix` can miss when the loaded path and the canonical
    // root use different prefixes, so the CLI emits an absolute path. Root
    // replacement then leaves `<root>/...` where other platforms already
    // emitted the repository-relative path.
    let text = text.replace("<root>/", "");
    let text = io_errors().replace_all(&text, "<io-error>");
    let text = redact_temp(&text);
    let text = pids().replace_all(&text, "$1 <pid>");
    timings().replace_all(&text, "<timing>").into_owned()
}

fn repo_roots() -> &'static [String] {
    static ROOTS: OnceLock<Vec<String>> = OnceLock::new();
    ROOTS.get_or_init(|| {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo = manifest.ancestors().nth(2).unwrap_or(manifest);
        root_variants(repo)
    })
}

fn replace_path_prefix(text: &str, root: &str, token: &str) -> String {
    let root = root.trim_end_matches('/');
    if root.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find(root) {
        let after = index + root.len();
        let boundary = rest[after..].chars().next();
        let at_boundary =
            boundary.is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-');
        out.push_str(&rest[..index]);
        out.push_str(if at_boundary { token } else { root });
        rest = &rest[after..];
    }
    out.push_str(rest);
    out
}

fn io_errors() -> &'static regex::Regex {
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(concat!(
            r"(?i)(?:no such file or directory",
            r"|the system cannot find the file specified\.",
            r"|the system cannot find the path specified\.)",
            r" \(os error \d+\)",
        ))
        .expect("io error pattern")
    })
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
