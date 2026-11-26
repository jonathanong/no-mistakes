mod messages;

pub(super) use messages::unreadable;
use messages::unresolved_binding_message;

use super::{
    Allowlist, Inputs, Location, RuleFinding, Shape, SourceDemand, PLAYWRIGHT_NO_SERIAL,
    TEST_NO_RETRIES,
};
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::fx::FxHashSet;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) struct RuleContext<'a> {
    pub(super) id: &'a str,
    pub(super) max_retries: u64,
    pub(super) allowlist: &'a Allowlist,
    pub(super) fixtures: &'a FxHashSet<(String, String)>,
}

pub(super) fn source_findings(
    path: &Path,
    relative: &str,
    inputs: &Inputs<'_>,
    ctx: &RuleContext<'_>,
    demand: SourceDemand,
) -> Result<Vec<RuleFinding>> {
    let id = ctx.id;
    let Some(facts) = inputs.facts.and_then(|facts| facts.ts.get(path)) else {
        anyhow::bail!("{id}: missing prepared facts for {relative}");
    };
    if let Some(message) = &facts.parse_error {
        return unreadable(inputs, path, relative, id, message.clone());
    }
    let mut findings = Vec::new();
    let mut first_unresolved = None;
    let mut suppressions = None;
    for location in facts
        .test_reliability
        .iter()
        .filter(|location| demand.includes(location.origin))
    {
        if location
            .project
            .as_ref()
            .is_some_and(|project| ctx.allowlist.projects.contains(project))
        {
            continue;
        }
        let detail = finding_details(location, id, ctx.max_retries);
        let uncertain_retry = location.unresolved_reason.is_some()
            && id == TEST_NO_RETRIES
            && matches!(location.shape, Shape::Retry(..));
        if detail.is_none() && !uncertain_retry {
            continue;
        }
        let configured = location
            .resolved_playwright_exports
            .iter()
            .any(|(m, e)| ctx.fixtures.contains(&(m.clone(), e.to_string())));
        if location.unresolved_reason.is_some() && location.playwright_candidates.is_empty() && !configured
            && location.playwright_import.is_none()
        {
            continue;
        }
        if (location.unresolved_reason.is_none() && location.playwright_import.is_none())
            || (configured && location.playwright_candidates.is_empty())
        {
            if let Some((target, message)) = detail {
                findings.push(RuleFinding {
                    rule: id.into(),
                    file: relative.into(),
                    line: location.line,
                    message,
                    import: None,
                    target: Some(target.into()),
                    causes: Vec::new(),
                });
            }
            continue;
        }
        let unresolved = unresolved_binding_message(relative, id, location);
        let index = suppressions.get_or_insert_with(|| inputs.sources.suppressions_for_path(path));
        if index
            .as_ref()
            .and_then(|idx| idx.matching(id, location.line.try_into().ok(), true))
            .is_some()
        {
            findings.push(RuleFinding {
                rule: id.into(),
                file: relative.into(),
                line: location.line,
                message: unresolved,
                import: None,
                target: Some("unresolved".into()),
                causes: Vec::new(),
            });
            continue;
        }
        if let Some(session) = inputs
            .session
            .filter(|session| session.partial_checks_enabled())
        {
            session.record_check_failure(CheckFailure {
                consumer: id.into(),
                file: Some(PathBuf::from(relative)),
                property: None,
                message: unresolved,
            });
        } else if first_unresolved.is_none() {
            first_unresolved = Some(unresolved);
        }
    }
    if let Some(message) = first_unresolved {
        return Err(CheckInputFailure::source(relative, message).into());
    }
    Ok(findings)
}

pub(super) fn findings_for<'a>(
    file: &str,
    id: &str,
    max_retries: u64,
    allowlist: &Allowlist,
    locations: impl Iterator<Item = &'a Location>,
) -> Vec<RuleFinding> {
    locations
        .filter_map(|location| {
            if location
                .project
                .as_ref()
                .is_some_and(|project| allowlist.projects.contains(project))
            {
                return None;
            }
            let (target, message) = finding_details(location, id, max_retries)?;
            Some(RuleFinding {
                rule: id.into(),
                file: file.into(),
                line: location.line,
                message,
                import: None,
                target: Some(target.into()),
                causes: Vec::new(),
            })
        })
        .collect()
}

fn finding_details(location: &Location, id: &str, max_retries: u64) -> Option<(&'static str, String)> {
    match location.shape {
        Shape::Retry(target, count)
            if id == TEST_NO_RETRIES && count.is_none_or(|count| count > max_retries) =>
        {
            let count = count.map_or_else(
                || "an unverified retry count".to_string(),
                |count| format!("a retry count of {count}"),
            );
            Some((
                target,
                format!(
                    "{target} configures {count}. Keep retries at or below {max_retries} and fix the underlying test failure."
                ),
            ))
        }
        Shape::Serial(target) if id == PLAYWRIGHT_NO_SERIAL => Some((
            target,
            format!(
                "{target} limits test parallelism. Isolate test state and let Playwright run tests in parallel."
            ),
        )),
        _ => None,
    }
}
