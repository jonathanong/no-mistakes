use crate::codebase::rules::{LocatedRuleFinding, PostgresFindings};
use std::collections::BTreeMap;

type Site = (usize, (usize, usize));

pub(super) fn combine(roots: BTreeMap<String, LocatedRuleFinding>) -> Option<LocatedRuleFinding> {
    if roots.len() < 2 {
        return roots.into_values().next();
    }
    let targets = roots.keys().cloned().collect::<Vec<_>>().join(", ");
    let mut findings = roots.into_values().collect::<Vec<_>>();
    let first = findings
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            (&left.finding.file, left.finding.line).cmp(&(&right.finding.file, right.finding.line))
        })?
        .0;
    let message = findings
        .iter()
        .map(|located| {
            let finding = &located.finding;
            finding
                .message
                .strip_prefix(&format!("{}:{}: ", finding.file, finding.line))
                .unwrap_or(&finding.message)
        })
        .collect::<Vec<_>>()
        .join("; ");
    let mut grouped = findings.swap_remove(first);
    grouped.finding.target = Some(targets);
    grouped.finding.message = format!(
        "{}:{}: {message}",
        grouped.finding.file, grouped.finding.line
    );
    Some(grouped)
}

pub(super) fn disambiguate(output: &mut PostgresFindings, sites: &[Site]) {
    if sites.len() < 2 {
        return;
    }
    let findings = &output.0;
    let suffixes = crate::codebase::rules::postgres_findings::site_suffixes(
        sites.iter().map(|(index, position)| {
            let finding = &findings[*index].finding;
            (finding, finding.file.as_str(), *position)
        }),
        "statement",
    );
    for ((index, _), suffix) in sites.iter().zip(suffixes) {
        if let Some(suffix) = suffix {
            output.0[*index].finding.message.push_str(&suffix);
        }
    }
}
