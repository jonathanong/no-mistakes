use super::mapping::{classify, Verdict};
use super::ports_text::{inline_specs, list_spec};
use super::runs::pin_message;
use super::{RuleFinding, RULE_ID};

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Jobs,
    Job,
    Services,
    Service,
    Container,
    Ports,
    Other,
}

struct Level {
    indent: usize,
    role: Role,
}

pub(super) fn service_findings(rel: &str, source: &str) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let mut stack: Vec<Level> = Vec::new();
    let mut job = String::new();
    let mut service = String::new();
    for (offset, raw) in source.lines().enumerate() {
        let line = offset + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw.len() - raw.trim_start().len();
        while stack.last().is_some_and(|level| level.indent >= indent) {
            match stack.pop().map(|level| level.role) {
                Some(Role::Job) => job.clear(),
                Some(Role::Service) => service.clear(),
                _ => {}
            }
        }
        let Some((key, rest)) = mapping_key(trimmed) else {
            if parent(&stack) == Some(Role::Ports) {
                if let Some(spec) = list_spec(trimmed) {
                    note(
                        &mut findings,
                        rel,
                        &job,
                        &service,
                        parent_place(&stack),
                        &spec,
                        line,
                    );
                }
            }
            continue;
        };
        let role = role_for(parent(&stack), key, rest);
        if role == Role::Job {
            job = key.to_string();
        }
        if role == Role::Service {
            service = key.to_string();
        }
        if role == Role::Ports {
            for spec in inline_specs(rest) {
                note(
                    &mut findings,
                    rel,
                    &job,
                    &service,
                    parent(&stack),
                    &spec,
                    line,
                );
            }
        }
        stack.push(Level { indent, role });
    }
    findings
}

fn note(
    findings: &mut Vec<RuleFinding>,
    rel: &str,
    job: &str,
    service: &str,
    parent: Option<Role>,
    spec: &str,
    line: usize,
) {
    let Verdict::Literal(host) = classify(spec) else {
        return;
    };
    let Some(place) = place(job, service, parent) else {
        return;
    };
    findings.push(RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: pin_message(rel, line, &place, &host, spec),
        import: None,
        target: Some(job.to_string()),
    });
}

fn place(job: &str, service: &str, parent: Option<Role>) -> Option<String> {
    if job.is_empty() {
        return None;
    }
    match parent {
        Some(Role::Service) if !service.is_empty() => {
            Some(format!("job `{job}` service `{service}`"))
        }
        Some(Role::Container) => Some(format!("job `{job}` container")),
        _ => None,
    }
}

fn parent(stack: &[Level]) -> Option<Role> {
    stack.last().map(|level| level.role)
}

fn parent_place(stack: &[Level]) -> Option<Role> {
    stack.iter().rev().find_map(|level| match level.role {
        Role::Service | Role::Container => Some(level.role),
        _ => None,
    })
}

fn role_for(parent: Option<Role>, key: &str, rest: &str) -> Role {
    match parent {
        None | Some(Role::Other) if key == "jobs" => Role::Jobs,
        Some(Role::Jobs) => Role::Job,
        Some(Role::Job) if key == "services" && opens_mapping(rest) => Role::Services,
        Some(Role::Job) if key == "container" && opens_mapping(rest) => Role::Container,
        Some(Role::Services) => Role::Service,
        Some(Role::Service | Role::Container) if key == "ports" => Role::Ports,
        _ => Role::Other,
    }
}

fn opens_mapping(rest: &str) -> bool {
    let rest = rest.trim();
    rest.is_empty()
        || matches!(rest, "|" | ">" | "|-" | ">-" | "|+" | ">+")
        || rest.starts_with('{')
}

fn mapping_key(trimmed: &str) -> Option<(&str, &str)> {
    if trimmed.starts_with('-') {
        return None;
    }
    let (key, rest) = trimmed.split_once(':')?;
    let key = key.trim().trim_matches(['"', '\'']);
    if key.is_empty() || key.contains([' ', '{']) {
        return None;
    }
    Some((key, rest.trim()))
}
