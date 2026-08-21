use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_enqueues(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for raw in extract_named(source, job_enqueue_re()) {
        names.push(raw.clone());
        if let Some((_, short)) = raw.rsplit_once("::") {
            names.push(short.to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

pub(super) fn extract_workers(source: &str) -> Vec<String> {
    let mut names = extract_named(source, application_job_class_re());
    names.extend(extract_sidekiq_workers(source));
    names.sort();
    names.dedup();
    names
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
    let mut current: Option<String> = None;
    let mut names = Vec::new();
    for line in source.lines() {
        if let Some(name) = class_name_re()
            .captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
        {
            current = Some(name);
        } else if sidekiq_include_re().is_match(line) {
            if let Some(name) = current.take() {
                names.push(name);
            }
        }
    }
    names.sort();
    names.dedup();
    names
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
    RE.get_or_init(|| Regex::new(r"(?m)^\s*class\s+([A-Z][\w:]*)").expect("class"))
}

fn sidekiq_include_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\binclude Sidekiq::(?:Worker|Job)\b").expect("sidekiq"))
}

#[cfg(test)]
#[path = "ruby_queue_tests.rs"]
mod tests;
