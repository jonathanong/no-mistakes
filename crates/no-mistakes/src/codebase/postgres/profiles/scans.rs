use super::super::EmbeddedSqlOptions;
pub use crate::codebase::postgres_analysis::scans::{EmbeddedSqlRuleScan, EmbeddedSqlRuleScans};
use anyhow::Result;

pub(super) fn record(
    grouped: &mut EmbeddedSqlRuleScans,
    profile: EmbeddedSqlOptions,
    rule_id: &str,
    options: &super::EmbeddedSqlRuleOptions,
    annotation_seed: bool,
) -> Result<()> {
    let include = include(rule_id, options);
    let mut scan = EmbeddedSqlRuleScan::new(rule_id, &include, &options.exclude)?;
    scan.annotation_seed = annotation_seed;
    let scans = grouped.entry(profile).or_default();
    if !scans
        .iter()
        .any(|existing| existing.same_application(&scan))
    {
        scans.push(scan);
    }
    Ok(())
}

pub(super) fn include(rule_id: &str, options: &super::EmbeddedSqlRuleOptions) -> Vec<String> {
    if rule_id == "postgres-no-test-state-mutation" {
        options
            .test_globs
            .iter()
            .chain(&options.helper_roots)
            .cloned()
            .collect()
    } else {
        options.include.clone()
    }
}

pub(super) fn configure_suppression_audit(scans: &mut EmbeddedSqlRuleScans, active: bool) {
    if active {
        for scan in scans.values_mut().flatten() {
            scan.suppression_audit = true;
        }
    }
}
