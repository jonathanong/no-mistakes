use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_enqueues(source: &str) -> Vec<String> {
    let masked = super::super::strip::mask_strings(source);
    canonicalize_names(extract_named(&masked, job_enqueue_re()))
}

pub(super) fn extract_workers(source: &str) -> Vec<String> {
    let masked = super::super::strip::mask_strings(source);
    let mut names = extract_named(&masked, application_job_class_re());
    names.extend(extract_sidekiq_workers(&masked));
    canonicalize_names(names)
}

fn extract_named(source: &str, re: &Regex) -> Vec<String> {
    let mut values: Vec<String> = re
        .captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();
    values.sort();
    values.dedup();
    values
}

fn extract_sidekiq_workers(source: &str) -> Vec<String> {
    let mut scopes: Vec<Option<String>> = Vec::new();
    let mut names = Vec::new();
    for line in source.lines() {
        for stmt in line.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            if let Some(name) = class_name_re()
                .captures(stmt)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
            {
                scopes.push(Some(name));
            } else if block_open_re().is_match(stmt) {
                scopes.push(None);
            }
            if sidekiq_include_re().is_match(stmt) {
                if let Some(name) = scopes.iter().rev().find_map(|scope| scope.clone()) {
                    names.push(name);
                }
            }
            if end_re().is_match(stmt) {
                scopes.pop();
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn canonicalize_names(names: Vec<String>) -> Vec<String> {
    let mut values: Vec<String> = names
        .into_iter()
        .map(|raw| match raw.rsplit_once("::") {
            Some((_, short)) => short.to_string(),
            None => raw,
        })
        .collect();
    values.sort();
    values.dedup();
    values
}

fn job_enqueue_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][\w:]*)\.perform_(?:later|async)\b").expect("enqueue"))
}

fn application_job_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*class\s+([A-Z][\w:]*)\s*<\s*ApplicationJob").expect("job class")
    })
}

fn class_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^class\s+([A-Z][\w:]*)").expect("class"))
}

fn block_open_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?:module|def|begin|case|if|unless|while|until)\b|\bdo\b").expect("block")
    })
}

fn end_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^end\b").expect("end"))
}

fn sidekiq_include_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\binclude Sidekiq::(?:Worker|Job)\b").expect("sidekiq"))
}

#[cfg(test)]
#[path = "ruby_queue_tests.rs"]
mod tests;
