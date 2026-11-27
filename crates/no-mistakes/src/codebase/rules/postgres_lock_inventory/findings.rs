use super::{LockInventoryReport, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use std::collections::BTreeMap;

pub(super) fn report_findings(report: LockInventoryReport) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for site in report.added {
        findings.push(finding(
            site.file,
            site.line,
            format!(
                "new PostgreSQL lock site ({}, {}). Remove the lock or record the reviewed site in the baseline",
                site.category, site.key
            ),
            site.category,
        ));
    }
    if report.baseline_unanalyzed {
        let mut first_lines: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for gap in &report.unanalyzed {
            first_lines
                .entry((gap.file.as_str(), gap.reason.as_str()))
                .or_insert(gap.line);
        }
        for delta in &report.unanalyzed_added {
            let line = first_lines
                .get(&(delta.file.as_str(), delta.reason.as_str()))
                .copied()
                .unwrap_or(1);
            findings.push(finding(
                delta.file.clone(),
                line,
                format!(
                    "unanalyzed PostgreSQL lock inputs grew ({}: {} current, {} baselined). Make the configured SQL or helper call statically analyzable or record the reviewed count in the baseline",
                    delta.reason, delta.current_count, delta.baseline_count
                ),
                "unanalyzable".into(),
            ));
        }
    } else {
        for gaps in report.unanalyzed.chunk_by(PartialEq::eq) {
            for (index, gap) in gaps.iter().enumerate() {
                let reason = if gaps.len() == 1 {
                    gap.reason.clone()
                } else {
                    format!("{}, occurrence {} of {}", gap.reason, index + 1, gaps.len())
                };
                findings.push(finding(
                    gap.file.clone(),
                    gap.line,
                    format!(
                        "incomplete PostgreSQL lock inventory ({reason}). Make the configured SQL or helper call statically analyzable"
                    ),
                    "unanalyzable".into(),
                ));
            }
        }
    }
    let removed_sites = report.removed.len();
    let removed_unanalyzed: usize = report
        .unanalyzed_removed
        .iter()
        .map(|delta| delta.baseline_count - delta.current_count)
        .sum();
    if let Some(summary) = stale_baseline_summary(removed_sites, removed_unanalyzed) {
        findings.push(finding(
            report.baseline_file,
            1,
            format!(
                "stale PostgreSQL lock baseline contains {summary}. Run `no-mistakes postgres lock-inventory --write-baseline` after resolving any new sites"
            ),
            "stale-baseline".into(),
        ));
    }
    findings
}

fn stale_baseline_summary(removed_sites: usize, removed_unanalyzed: usize) -> Option<String> {
    match (removed_sites, removed_unanalyzed) {
        (0, 0) => None,
        (sites, 0) => Some(format!("{sites} removed site(s)")),
        (0, unanalyzed) => Some(format!("{unanalyzed} fewer unanalyzed input(s)")),
        (sites, unanalyzed) => Some(format!(
            "{sites} removed site(s) and {unanalyzed} fewer unanalyzed input(s)"
        )),
    }
}

pub(super) fn finding(file: String, line: usize, message: String, target: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file,
        line,
        message,
        target: Some(target),
        import: None,
        causes: Vec::new(),
    }
}
