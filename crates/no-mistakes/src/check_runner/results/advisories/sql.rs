use no_mistakes::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::ts_source::{byte_offset_to_line, SourceStore};
use std::path::Path;

pub(crate) fn merge_sql(
    advisories: &mut Vec<RuleFinding>,
    root: &Path,
    sources: &SourceStore,
    verdicts: &[SqlObligationVerdict],
) {
    let mut added = false;
    for verdict in verdicts.iter().filter(|verdict| {
        verdict.advisory
            && verdict.status != SqlVerdictStatus::ProvenClear
            && !verdict.residual_reasons.is_empty()
    }) {
        added = true;
        let line = sources
            .read_path(&root.join(&verdict.file))
            .ok()
            .map(|source| byte_offset_to_line(&source, verdict.span.0 as usize) as usize)
            .unwrap_or(1);
        let reasons = verdict.residual_reasons.join(", ");
        let prefix = verdict
            .known_prefix
            .as_ref()
            .map_or_else(String::new, |text| {
                format!(
                    "; known SQL prefix {:?}",
                    text.chars().take(160).collect::<String>()
                )
            });
        let helpers = if verdict.helper_lines.is_empty() {
            String::new()
        } else {
            format!("; helper lines {:?}", verdict.helper_lines)
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
        let explanation = if verdict.status == SqlVerdictStatus::Violation {
            "proved a violation, but remaining SQL is unresolved for"
        } else {
            "cannot prove"
        };
        advisories.push(RuleFinding {
            rule: verdict.rule.clone(),
            file: verdict.file.clone(),
            line,
            message: format!(
                "{}:{line}: {}{application} {explanation} {}: {reasons}{prefix}{helpers}",
                verdict.file, verdict.rule, verdict.subcheck
            ),
            import: None,
            target: Some(verdict.subcheck.clone()),
        });
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
