use super::super::EmbeddedSqlOptions;
use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Arc;

/// One embedded-SQL rule application and the paths it scans.
#[derive(Clone)]
pub struct EmbeddedSqlRuleScan {
    pub(crate) rule_id: String,
    pub(crate) annotation_seed: bool,
    pub(crate) suppression_audit: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    pub(crate) paths: Arc<crate::codebase::rules::RuleGlobFilter>,
}

impl EmbeddedSqlRuleScan {
    pub(crate) fn new(rule_id: &str, include: &[String], exclude: &[String]) -> Result<Self> {
        Ok(Self {
            rule_id: rule_id.to_string(),
            annotation_seed: false,
            suppression_audit: false,
            include: include.to_vec(),
            exclude: exclude.to_vec(),
            paths: Arc::new(crate::codebase::rules::RuleGlobFilter::new(
                rule_id, include, exclude,
            )?),
        })
    }

    pub(crate) fn unrestricted(&self) -> bool {
        self.exclude.is_empty()
            && (self.include.is_empty()
                || self
                    .include
                    .iter()
                    .any(|value| matches!(value.as_str(), "**" | "**/*")))
    }

    pub(crate) fn same_application(&self, other: &Self) -> bool {
        self.rule_id == other.rule_id
            && self.annotation_seed == other.annotation_seed
            && self.include == other.include
            && self.exclude == other.exclude
    }
}

pub type EmbeddedSqlRuleScans = BTreeMap<EmbeddedSqlOptions, Vec<EmbeddedSqlRuleScan>>;

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
