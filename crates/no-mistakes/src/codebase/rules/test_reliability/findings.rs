use super::{
    Allowlist, Inputs, Location, RuleFinding, Shape, SourceDemand, PLAYWRIGHT_NO_SERIAL,
    TEST_NO_RETRIES,
};
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::ts_source::has_disable_file_comment;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn source_findings(
    path: &Path,
    relative: &str,
    inputs: &Inputs<'_>,
    id: &str,
    max_retries: u64,
    allowlist: &Allowlist,
    demand: SourceDemand,
) -> Result<Vec<RuleFinding>> {
    let Some(facts) = inputs.facts.and_then(|facts| facts.ts.get(path)) else {
        anyhow::bail!("{id}: missing prepared facts for {relative}");
    };
    if let Some(message) = &facts.parse_error {
        return unreadable(inputs, path, relative, id, message.clone());
    }
    Ok(findings_for(
        relative,
        id,
        max_retries,
        allowlist,
        facts.test_reliability.iter().filter(|location| {
            location.playwright_import.is_none() && demand.includes(location.origin)
        }),
    ))
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
    locations.filter_map(|location| {
        if location.project.as_ref().is_some_and(|project| allowlist.projects.contains(project)) {
            return None;
        }
        let (target, message) = match location.shape {
            Shape::Retry(target, count) if id == TEST_NO_RETRIES && count.is_none_or(|count| count > max_retries) => {
                let count = count.map_or_else(|| "an unverified retry count".to_string(), |count| format!("a retry count of {count}"));
                (target, format!("{target} configures {count}. Keep retries at or below {max_retries} and fix the underlying test failure."))
            }
            Shape::Serial(target) if id == PLAYWRIGHT_NO_SERIAL => {
                (target, format!("{target} limits test parallelism. Isolate test state and let Playwright run tests in parallel."))
            }
            _ => return None,
        };
        Some(RuleFinding {
            rule: id.into(),
            file: file.into(),
            line: location.line,
            message,
            import: None,
            target: Some(target.into()),
            causes: Vec::new(),
        })
    }).collect()
}
