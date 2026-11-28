//! Merging coverage reports across duplicate `SF:` records and shards.

use crate::model::{CoverageReport, FileCoverage};

/// Merges `incoming` file coverage into `target` using saturating hit sums.
///
/// Function start/end lines preserve the first-seen definition while filling
/// missing metadata when an earlier record only saw `FNDA:` hits.
pub fn merge_file_coverage(target: &mut FileCoverage, incoming: &FileCoverage) {
    for (name, incoming_fn) in &incoming.functions {
        match target.functions.get_mut(name) {
            Some(existing) => {
                if existing.line.is_none() {
                    existing.line = incoming_fn.line;
                }
                if existing.end_line.is_none() {
                    existing.end_line = incoming_fn.end_line;
                }
                existing.hits = existing.hits.saturating_add(incoming_fn.hits);
            }
            None => {
                target.functions.insert(name.clone(), incoming_fn.clone());
            }
        }
    }

    for (key, &hits) in &incoming.branches {
        let entry = target.branches.entry(key.clone()).or_insert(0);
        *entry = entry.saturating_add(hits);
    }

    for (&line, &hits) in &incoming.lines {
        let entry = target.lines.entry(line).or_insert(0);
        *entry = entry.saturating_add(hits);
    }
}

/// Merges `incoming` into `target` in place, ignoring test names (`TN:`).
pub fn merge_into(target: &mut CoverageReport, incoming: &CoverageReport) {
    for (file, cov) in &incoming.files {
        let dest = target.files.entry(file.clone()).or_default();
        merge_file_coverage(dest, cov);
    }
}

/// Merges multiple coverage reports into a new deterministic report.
pub fn merge_reports<'a, I>(reports: I) -> CoverageReport
where
    I: IntoIterator<Item = &'a CoverageReport>,
{
    let mut merged = CoverageReport::new();
    for report in reports {
        merge_into(&mut merged, report);
    }
    merged
}
