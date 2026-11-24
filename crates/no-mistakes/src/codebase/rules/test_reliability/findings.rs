use super::{
    Allowlist, Inputs, Location, RuleFinding, Shape, SourceDemand, PLAYWRIGHT_NO_SERIAL,
    TEST_NO_RETRIES,
};
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::codebase::ts_source::has_disable_file_comment;
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
        let Some((target, message)) = finding_details(location, id, ctx.max_retries) else {
            continue;
        };
        let configured = location
            .resolved_playwright_exports
            .iter()
            .any(|(m, e)| ctx.fixtures.contains(&(m.clone(), e.to_string())));
        if location.playwright_import.is_none() || configured {
            findings.push(RuleFinding {
                rule: id.into(),
                file: relative.into(),
                line: location.line,
                message,
                import: None,
                target: Some(target.into()),
                causes: Vec::new(),
            });
            continue;
        }
        let unresolved = unresolved_binding_message(relative, id, location);
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

fn unresolved_binding_message(relative: &str, id: &str, location: &Location) -> String {
    let binding = location
        .playwright_binding
        .as_deref()
        .unwrap_or("test");
    if let Some((module, export)) = location.resolved_playwright_exports.first() {
        return format!(
            "{relative}:{}: cannot verify whether imported binding `{binding}` (from `{module}` export `{export}`) is Playwright's `test`; if it is a Playwright fixture, add `{{ module: \"{module}\", export: \"{export}\" }}` to `{id}` `options.fixtures`",
            location.line
        );
    }
    let (specifier, export) = location
        .playwright_import
        .as_ref()
        .map_or(("<unknown>", "test"), |import| {
            (import.specifier.as_ref(), import.name.as_ref())
        });
    format!(
        "{relative}:{}: cannot verify whether imported binding `{binding}` (from `{specifier}` export `{export}`) is Playwright's `test`; if it is a Playwright fixture, add its repository-relative module and export to `{id}` `options.fixtures`",
        location.line
    )
}

pub(super) fn unreadable(
    inputs: &Inputs<'_>,
    path: &Path,
    relative: &str,
    id: &str,
    message: String,
) -> Result<Vec<RuleFinding>> {
    let message = format!("{message}. Repair this file so {id} can verify its test settings.");
    if inputs
        .sources
        .read_path(path)
        .is_ok_and(|source| has_disable_file_comment(&source, id))
    {
        return Ok(vec![RuleFinding {
            rule: id.into(),
            file: relative.into(),
            line: 1,
            message,
            import: None,
            target: Some("unanalyzable".into()),
            causes: Vec::new(),
        }]);
    }
    Err(CheckInputFailure {
        file: Some(PathBuf::from(relative)),
        property: None,
        message,
    }
    .into())
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
