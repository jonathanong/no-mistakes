use serde_yaml::{Mapping, Value};

#[derive(Clone, Copy)]
pub(super) enum RefRisk {
    Base,
    Head,
    Unknown,
}

pub(super) fn classify(step: &Mapping, default_branch: Option<&str>) -> RefRisk {
    let with = match step.get("with") {
        None | Some(Value::Null) => return RefRisk::Base,
        Some(Value::Mapping(with)) => with,
        _ => return RefRisk::Unknown,
    };
    match with.get("ref") {
        None => repository_bound(with, RefRisk::Base),
        Some(Value::String(text)) => {
            repository_bound(with, classify_text(text.trim(), default_branch))
        }
        _ => RefRisk::Unknown,
    }
}

fn classify_text(text: &str, default_branch: Option<&str>) -> RefRisk {
    // An empty checkout input falls back to the event's base-side ref.
    if text.is_empty() {
        return RefRisk::Base;
    }
    if text
        .strip_prefix("refs/pull/")
        .is_some_and(|tail| !tail.is_empty())
    {
        return RefRisk::Head;
    }
    if default_branch.is_some_and(|branch| {
        (text == branch && !is_commit_id(text)) || text.strip_prefix("refs/heads/") == Some(branch)
    }) {
        return RefRisk::Base;
    }
    let Some(expression) = text
        .strip_prefix("${{")
        .and_then(|text| text.strip_suffix("}}"))
    else {
        return RefRisk::Unknown;
    };
    match expression.trim() {
        "github.sha"
        | "github.ref"
        | "github.base_ref"
        | "github.event.repository.default_branch"
        | "github.event.pull_request.base.sha"
        | "github.event.pull_request.base.ref" => RefRisk::Base,
        "github.head_ref"
        | "github.event.pull_request.head.sha"
        | "github.event.pull_request.head.ref" => RefRisk::Head,
        _ => RefRisk::Unknown,
    }
}

fn repository_bound(with: &Mapping, risk: RefRisk) -> RefRisk {
    // A head ref stays unsafe; branch names need proof of the base repository.
    if !matches!(risk, RefRisk::Base) {
        return risk;
    }
    match with.get("repository") {
        None => risk,
        Some(Value::String(text)) if is_base_repository(text.trim()) => risk,
        _ => RefRisk::Unknown,
    }
}

fn is_base_repository(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    let Some(expression) = text
        .strip_prefix("${{")
        .and_then(|text| text.strip_suffix("}}"))
    else {
        return false;
    };
    matches!(
        expression.trim(),
        "github.repository"
            | "github.event.repository.full_name"
            | "github.event.pull_request.base.repo.full_name"
    )
}

fn is_commit_id(text: &str) -> bool {
    // Checkout resolves bare commit IDs before considering branch conventions.
    matches!(text.len(), 40 | 64) && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}
