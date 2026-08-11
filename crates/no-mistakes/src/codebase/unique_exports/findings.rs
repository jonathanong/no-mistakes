use super::types::{
    ExportBucket, ExportOccurrence, ExportOrigin, UniqueExportFinding, UniqueExportsOptions,
};
use super::RULE_ID;
use anyhow::Result;
use std::collections::BTreeMap;

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
        let mut unique_occurrences: Vec<ExportOccurrence> = Vec::new();
        let mut origin_indices: BTreeMap<ExportOrigin, usize> = BTreeMap::new();
        for occurrence in occurrences {
            let origin = occurrence.origin.clone();
            if let Some(index) = origin_indices.get(&origin).copied() {
                if unique_occurrences[index].suppressed && !occurrence.suppressed {
                    unique_occurrences[index] = occurrence;
                }
            } else {
                let index = unique_occurrences.len();
                origin_indices.insert(origin, index);
                unique_occurrences.push(occurrence);
            }
        }
        if unique_occurrences.len() < 2 {
            continue;
        }
        // Preserve the visible duplicate representative in both ordinary and
        // audit reports. Suppressed canonical provenance is retained by the
        // occurrence metadata for accounting, not by changing this selection.
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
                file: duplicate.file.clone(),
                line: duplicate.line,
                export_name: name.clone(),
                export_kind: bucket.as_str().to_string(),
                message: format!(
                    "{} `{}` is already exported from {}:{}; rename or consolidate this exported API",
                    bucket.message_label(),
                    name,
                    first.file,
                    first.line
                ),
                suppression_source_file: duplicate
                    .suppression_location
                    .as_ref()
                    .map(|(file, _)| file.clone())
            });
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}
