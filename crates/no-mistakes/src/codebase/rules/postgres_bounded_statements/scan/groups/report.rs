use crate::codebase::rules::{LocatedRuleFinding, PostgresFindings};
use std::collections::BTreeMap;

type Site = (usize, Option<super::Execution>, (usize, usize));

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
    let mut counts = BTreeMap::new();
    for (index, _, _) in sites {
        *counts.entry(output.0[*index].finding.clone()).or_insert(0) += 1;
    }
    for (index, call, (line, column)) in sites {
        let finding = &mut output.0[*index].finding;
        if counts[&*finding] > 1 {
            let call = call
                .map(|execution| match execution {
                    super::Execution::Call(index) => format!("; executor call {index}"),
                    super::Execution::Projection(index) => format!("; executor projection {index}"),
                })
                .unwrap_or_default();
            finding.message.push_str(&format!(
                " (statement at SQL line {line}, column {column}{call})"
            ));
        }
    }
}
