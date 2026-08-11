use super::types::{
    ExportBucket, ExportOccurrence, ExportOrigin, PreparedUniqueExportFinding, UniqueExportFinding,
    UniqueExportsOptions,
};
use super::RULE_ID;
use anyhow::Result;
use std::collections::BTreeMap;

pub(super) fn unique_export_findings(
    occurrences: Vec<ExportOccurrence>,
    options: UniqueExportsOptions,
) -> Result<Vec<PreparedUniqueExportFinding>> {
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
                if suppressed(&unique_occurrences[index]) && !suppressed(&occurrence) {
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
        let active_occurrences = unique_occurrences
            .iter()
            .filter(|occurrence| !suppressed(occurrence))
            .collect::<Vec<_>>();
        // Baseline and audit reports must select their public duplicate from
        // the same active occurrences. Suppressed occurrences below are only
        // sidecars for directive accounting.
        let first_active = active_occurrences.first().copied();
        if let Some(first_active) = first_active {
            for duplicate in active_occurrences.into_iter().skip(1) {
                findings.push(finding(duplicate, first_active, &name, bucket, None));
            }
        }

        // Retain every suppressed duplicate as an accounting sidecar without
        // permitting it to become the public comparison anchor. When every
        // occurrence is suppressed, preserve the normal n - 1 cardinality.
        let sidecar_anchor = first_active.or_else(|| unique_occurrences.first());
        if let Some(sidecar_anchor) = sidecar_anchor {
            for suppressed in unique_occurrences
                .iter()
                .filter(|occurrence| suppressed(occurrence))
            {
                if !std::ptr::eq(suppressed, sidecar_anchor) {
                    findings.push(finding(
                        suppressed,
                        sidecar_anchor,
                        &name,
                        bucket,
                        suppressed.suppression_location.clone(),
                    ));
                }
            }
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn finding(
    duplicate: &ExportOccurrence,
    first: &ExportOccurrence,
    name: &str,
    bucket: ExportBucket,
    suppression_source_location: Option<(String, u32)>,
) -> PreparedUniqueExportFinding {
    PreparedUniqueExportFinding {
        finding: UniqueExportFinding {
            rule: RULE_ID.to_string(),
            file: duplicate.file.clone(),
            line: duplicate.line,
            export_name: name.to_string(),
            export_kind: bucket.as_str().to_string(),
            message: format!(
                "{} `{}` is already exported from {}:{}; rename or consolidate this exported API",
                bucket.message_label(),
                name,
                first.file,
                first.line
            ),
        },
        suppression_source_location,
    }
}

fn suppressed(occurrence: &ExportOccurrence) -> bool {
    // Re-export provenance can be populated independently from the legacy
    // boolean while deferred aggregate suppression is resolving a source
    // directive. Either representation means this occurrence is a sidecar.
    occurrence.suppressed || occurrence.suppression_location.is_some()
}
