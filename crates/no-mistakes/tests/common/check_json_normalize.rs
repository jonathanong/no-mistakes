use serde_json::Value;
use std::path::Path;
use std::sync::OnceLock;

pub(super) fn root_variants(root: &Path) -> Vec<String> {
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

pub(super) fn normalize_value(value: &mut Value, roots: &[String]) {
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

pub(super) fn repo_roots() -> &'static [String] {
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

pub(super) fn sort_value(value: &mut Value) {
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
