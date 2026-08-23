use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_enqueues(source: &str) -> Vec<String> {
    let masked = super::super::strip::mask_strings(source);
    extract_named(&masked, job_enqueue_re())
}

pub(super) fn extract_workers(source: &str) -> Vec<String> {
    let masked = super::super::strip::mask_strings(source);
    extract_declared_workers(&masked)
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

enum Scope {
    Named(String),
    Block,
}

fn extract_declared_workers(source: &str) -> Vec<String> {
    let mut scopes: Vec<Scope> = Vec::new();
    let mut names = Vec::new();
    for line in source.lines() {
        for stmt in line.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            if singleton_class_re().is_match(stmt) {
                scopes.push(Scope::Block);
            } else if let Some(name) = class_name_re()
                .captures(stmt)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
            {
                scopes.push(Scope::Named(name));
                if application_job_class_re().is_match(stmt) {
                    names.extend(qualified_name(&scopes));
                }
            } else if let Some(name) = module_name_re()
                .captures(stmt)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
            {
                scopes.push(Scope::Named(name));
            } else if block_open_re().is_match(stmt) {
                scopes.push(Scope::Block);
            }
            if sidekiq_include_re().is_match(stmt) {
                names.extend(qualified_name(&scopes));
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

fn qualified_name(scopes: &[Scope]) -> Option<String> {
    let mut parts = Vec::new();
    for scope in scopes {
        if let Scope::Named(name) = scope {
            if name.contains("::") {
                parts.clear();
            }
            parts.push(name.as_str());
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("::"))
    }
}

fn job_enqueue_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b([A-Z][\w:]*)(?:\.set\([^)]*\))?\.perform_(?:later|async)\b")
            .expect("enqueue")
    })
}

fn application_job_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<\s*ApplicationJob\b").expect("job class"))
}

fn class_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^class\s+([A-Z][\w:]*)").expect("class"))
}

fn singleton_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^class\s+<<").expect("singleton class"))
}

fn module_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^module\s+([A-Z][\w:]*)").expect("module"))
}

fn block_open_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?:def|begin|case|if|unless|while|until)\b|\bdo\b").expect("block")
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
