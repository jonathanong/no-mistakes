use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::ts_source::{byte_offset_to_line, SourceStore};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn merge_sql(
    advisories: &mut Vec<RuleFinding>,
    root: &Path,
    sources: &SourceStore,
    facts: &CheckFactMap,
    verdicts: &[SqlObligationVerdict],
) {
    let mut groups = BTreeMap::new();
    for verdict in verdicts.iter().filter(|verdict| {
        verdict.advisory
            && verdict.status != SqlVerdictStatus::ProvenClear
            && !verdict.residual_reasons.is_empty()
    }) {
        // Keep the original call span: branch origins and equal display lines do
        // not identify an executor invocation. Applications/profiles stay distinct.
        groups
            .entry((
                &verdict.profile,
                &verdict.rule,
                &verdict.application,
                &verdict.file,
                verdict.span,
            ))
            .or_insert_with(Vec::new)
            .push(verdict);
    }
    let added = !groups.is_empty();
    for rows in groups.into_values() {
        advisories.push(finding(root, sources, facts, &rows));
    }
    if added {
        advisories.sort_by(|left, right| {
            (&left.file, left.line, &left.rule, &left.message).cmp(&(
                &right.file,
                right.line,
                &right.rule,
                &right.message,
            ))
        });
    }
}

fn finding(
    root: &Path,
    sources: &SourceStore,
    facts: &CheckFactMap,
    rows: &[&SqlObligationVerdict],
) -> RuleFinding {
    let verdict = rows[0];
    let line = facts
        .prepared_low_sql_display_line(root, verdict)
        .unwrap_or_else(|| {
            sources
                .read_path(&root.join(&verdict.file))
                .ok()
                .map(|source| byte_offset_to_line(&source, verdict.span.0 as usize) as usize)
                .unwrap_or(1)
        });
    let reasons = rows
        .iter()
        .flat_map(|row| row.residual_reasons.iter().map(String::as_str))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    let checks = rows
        .iter()
        .map(|row| row.subcheck.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    let prefix = rows
        .iter()
        .filter_map(|row| row.known_prefix.as_ref())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|text| {
            format!(
                "; known SQL prefix {:?}",
                text.chars().take(160).collect::<String>()
            )
        })
        .collect::<String>();
    let helper_lines = rows
        .iter()
        .flat_map(|row| row.helper_lines.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let helpers = if helper_lines.is_empty() {
        String::new()
    } else {
        format!("; helper lines {helper_lines:?}")
    };
    let application = verdict
        .application
        .as_ref()
        .map_or_else(String::new, |app| {
            app.name.as_ref().map_or_else(
                || format!(" application #{}", app.index),
                |name| format!(" application {name:?} (#{})", app.index),
            )
        });
    let outcome = if rows
        .iter()
        .any(|row| row.status == SqlVerdictStatus::Violation)
    {
        "proved a violation, but remaining SQL is unresolved for"
    } else {
        "has unresolved SQL for"
    };
    RuleFinding {
        rule: verdict.rule.clone(),
        file: verdict.file.clone(),
        line,
        message: format!(
            "{}:{line}: {}{application} {outcome} {checks}: {reasons}{prefix}{helpers}",
            verdict.file, verdict.rule
        ),
        import: None,
        target: Some(checks),
    }
}

#[cfg(test)]
mod tests;
