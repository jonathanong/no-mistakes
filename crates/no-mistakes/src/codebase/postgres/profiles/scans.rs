use super::super::EmbeddedSqlOptions;
use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Arc;

/// One embedded-SQL rule application and the paths it scans.
#[derive(Clone)]
pub struct EmbeddedSqlRuleScan {
    pub(crate) rule_id: String,
    include: Vec<String>,
    exclude: Vec<String>,
    pub(crate) paths: Arc<crate::codebase::rules::RuleGlobFilter>,
}

impl EmbeddedSqlRuleScan {
    pub(crate) fn new(rule_id: &str, include: &[String], exclude: &[String]) -> Result<Self> {
        Ok(Self {
            rule_id: rule_id.to_string(),
            include: include.to_vec(),
            exclude: exclude.to_vec(),
            paths: Arc::new(crate::codebase::rules::RuleGlobFilter::new(
                rule_id, include, exclude,
            )?),
        })
    }

    pub(crate) fn same_application(&self, other: &Self) -> bool {
        self.rule_id == other.rule_id
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
) -> Result<()> {
    let scan = EmbeddedSqlRuleScan::new(rule_id, &options.include, &options.exclude)?;
    let scans = grouped.entry(profile).or_default();
    if !scans
        .iter()
        .any(|existing| existing.same_application(&scan))
    {
        scans.push(scan);
    }
    Ok(())
}
