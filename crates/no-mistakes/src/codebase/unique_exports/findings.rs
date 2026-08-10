use super::types::{ExportBucket, ExportOccurrence, UniqueExportFinding, UniqueExportsOptions};
use super::RULE_ID;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn unique_export_findings(
    occurrences: Vec<ExportOccurrence>,
    options: UniqueExportsOptions,
) -> Result<Vec<UniqueExportFinding>> {
    let mut buckets: BTreeMap<(String, ExportBucket), Vec<ExportOccurrence>> = BTreeMap::new();
    for occurrence in occurrences {
        buckets
            .entry((
                occurrence.name.clone(),
                occurrence
                    .bucket
                    .key(options.unique_across_types_and_values),
            ))
            .or_default()
            .push(occurrence);
    }

    let mut findings = Vec::new();
    for ((name, bucket), mut occurrences) in buckets {
        occurrences.sort_by(|a, b| (&a.file, a.line, &a.kind).cmp(&(&b.file, b.line, &b.kind)));
        let mut origins = BTreeSet::new();
        let unique_occurrences = occurrences
            .into_iter()
            .filter(|occurrence| origins.insert(occurrence.origin.clone()))
            .collect::<Vec<_>>();
        if unique_occurrences.len() < 2 {
            continue;
        }
        // In aggregate mode preserve standalone semantics: a suppressed
        // occurrence must not turn an unsuppressed export into a duplicate.
        // Still emit the suppressed occurrence so finalization can account for
        // its directive.
        let first = unique_occurrences
            .iter()
            .find(|occurrence| !occurrence.suppressed)
            .unwrap_or(&unique_occurrences[0]);
        for duplicate in unique_occurrences
            .iter()
            .filter(|item| !std::ptr::eq(*item, first))
        {
            findings.push(UniqueExportFinding {
                rule: RULE_ID.to_string(),
                file: duplicate
                    .suppression_location
                    .as_ref()
                    .map(|(file, _)| file.clone())
                    .unwrap_or_else(|| duplicate.file.clone()),
                line: duplicate
                    .suppression_location
                    .as_ref()
                    .map(|(_, line)| *line)
                    .unwrap_or(duplicate.line),
                export_name: name.clone(),
                export_kind: bucket.as_str().to_string(),
                message: format!(
                    "{} `{}` is already exported from {}:{}; rename or consolidate this exported API",
                    bucket.message_label(),
                    name,
                    first.file,
                    first.line
                ),
            });
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}
