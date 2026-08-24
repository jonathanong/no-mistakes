use super::super::BaseDir;
use super::types::Extracted;
use regex::Regex;
use serde_yaml::Value;

pub(crate) fn workspace_filters(document: &Value) -> Vec<Extracted> {
    run_scripts(document)
        .into_iter()
        .flat_map(workspace_filters_in_script)
        .enumerate()
        .map(|(index, mut extracted)| {
            extracted.field = format!("pnpm filter {index}");
            extracted
        })
        .collect()
}

fn workspace_filters_in_script(source: &str) -> Vec<Extracted> {
    let filter = Regex::new(r#"--filter\s+(?:\\\s*)?(?:"([^"]+)"|'([^']+)'|([^\s'"]+))"#)
        .expect("pnpm filter pattern is valid");
    let mut extracted = Vec::new();
    for capture in filter.captures_iter(source) {
        let filter_offset = capture.get(0).unwrap().start();
        if !is_pnpm_invocation(source, filter_offset) {
            continue;
        }
        let raw = capture
            .get(1)
            .or_else(|| capture.get(2))
            .or_else(|| capture.get(3))
            .map(|match_| match_.as_str())
            .unwrap_or_default();
        let Some(path) = normalize_filter(raw) else {
            continue;
        };
        if is_guarded(source, filter_offset, &path) || path.starts_with('!') {
            continue;
        }
        extracted.push(Extracted {
            field: String::new(),
            allow_globs: path.contains('*') || path.contains('?') || path.contains('{'),
            base_dir: BaseDir::Root,
            value: path,
        });
    }
    extracted
}

fn is_pnpm_invocation(source: &str, filter_offset: usize) -> bool {
    let prefix = &source[..filter_offset];
    let command_start = prefix
        .char_indices()
        .rev()
        .find_map(|(offset, character)| match character {
            ';' | '|' | '&' => Some(offset + character.len_utf8()),
            '\n' if !prefix[..offset].trim_end().ends_with('\\') => {
                Some(offset + character.len_utf8())
            }
            _ => None,
        })
        .unwrap_or(0);
    let command = source[command_start..filter_offset].trim();
    let invocation =
        Regex::new(r"^(?:(?:[A-Za-z_][A-Za-z0-9_]*=[^\s]+)\s+)*(?:then\s+)?pnpm(?:\s|$)")
            .expect("pnpm invocation pattern is valid");
    invocation.is_match(command)
}

fn run_scripts(document: &Value) -> Vec<&str> {
    let mut scripts = Vec::new();
    if let Some(jobs) = document.get("jobs").and_then(Value::as_mapping) {
        for job in jobs.values() {
            scripts.extend(step_run_scripts(job));
        }
    }
    if let Some(runs) = document.get("runs") {
        scripts.extend(step_run_scripts(runs));
    }
    scripts
}

fn step_run_scripts(value: &Value) -> Vec<&str> {
    value
        .get("steps")
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(|step| step.get("run").and_then(Value::as_str))
        .collect()
}

fn normalize_filter(raw: &str) -> Option<String> {
    let mut path = raw.trim().trim_end_matches(';');
    if let Some(braced) = path.strip_prefix('{') {
        path = braced.strip_suffix("}...").unwrap_or(braced);
        path = path.strip_suffix('}').unwrap_or(path);
    } else {
        path = path.strip_suffix("...").unwrap_or(path);
    }
    if !path.starts_with("./") || path == "./" {
        return None;
    }
    Some(path.to_string())
}

fn is_guarded(source: &str, filter_offset: usize, path: &str) -> bool {
    if path.contains('*') || path.contains('?') || path.contains('{') {
        return false;
    }
    let normalized = path.trim_start_matches("./");
    let escaped = regex::escape(normalized);
    let directory = format!(r#"(?:\./)?{escaped}"#);
    let file = format!(r#"(?:\./)?{escaped}(?:/package\.json)?"#);
    let patterns = [
        format!(r#"\[\[?\s+-f\s+[\"']?{file}[\"']?\s*\]\]?"#),
        format!(r#"\[\[?\s+-d\s+[\"']?{directory}[\"']?\s*\]\]?"#),
        format!(r#"test\s+-f\s+[\"']?{file}[\"']?"#),
        format!(r#"test\s+-d\s+[\"']?{directory}[\"']?"#),
    ];
    let pnpm_start = source[..filter_offset]
        .rfind("pnpm")
        .unwrap_or(filter_offset);
    patterns.iter().any(|pattern| {
        let guard = Regex::new(pattern).expect("pnpm filter guard pattern is valid");
        let guarded = guard.find_iter(&source[..pnpm_start]).any(|matched| {
            let after_guard = &source[matched.end()..pnpm_start];
            // `test -d path; pnpm …` is not conditional. The guard controls
            // this pnpm invocation only through shell `&&` or an unclosed if.
            after_guard.trim().trim_end_matches('\\').trim() == "&&"
                || (source[..matched.start()].trim_end().ends_with("if")
                    && after_guard.contains("then")
                    && !after_guard.contains("fi"))
        });
        guarded
    })
}
