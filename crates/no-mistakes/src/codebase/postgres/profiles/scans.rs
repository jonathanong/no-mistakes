use super::super::EmbeddedSqlOptions;
use crate::codebase::rules::path_filter::RulePathFilter;
use crate::config::v2::schema::{ProjectType, RuleDef};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
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
    annotation_application: Option<Arc<RulePathFilter>>,
}

impl EmbeddedSqlRuleScan {
    pub(crate) fn new(rule_id: &str, include: &[String], exclude: &[String]) -> Result<Self> {
        Ok(Self {
            rule_id: rule_id.to_string(),
            annotation_seed: false,
            suppression_audit: false,
            include: include.to_vec(),
            exclude: exclude.to_vec(),
            annotation_application: None,
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

    pub(crate) fn annotation_unrestricted(&self) -> bool {
        self.annotation_application.is_none() && self.unrestricted()
    }

    pub(crate) fn includes_annotation(&self, path: &Path, relative: &str) -> bool {
        self.annotation_seed
            && self.paths.includes(relative)
            && self
                .annotation_application
                .as_ref()
                .is_none_or(|filter| filter.is_match(path))
    }

    pub(crate) fn same_application(&self, other: &Self) -> bool {
        self.rule_id == other.rule_id
            && self.annotation_seed == other.annotation_seed
            && self.include == other.include
            && self.exclude == other.exclude
            && match (&self.annotation_application, &other.annotation_application) {
                (None, None) => true,
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                _ => false,
            }
    }
}

pub type EmbeddedSqlRuleScans = BTreeMap<EmbeddedSqlOptions, Vec<EmbeddedSqlRuleScan>>;

pub(super) fn record(
    grouped: &mut EmbeddedSqlRuleScans,
    profile: EmbeddedSqlOptions,
    rule_id: &str,
    options: &super::EmbeddedSqlRuleOptions,
    annotation_seed: bool,
    application: Option<(&Path, &super::NoMistakesConfig, &RuleDef)>,
) -> Result<()> {
    let include = include(rule_id, options);
    let mut scan = EmbeddedSqlRuleScan::new(rule_id, &include, &options.exclude)?;
    scan.annotation_seed = annotation_seed;
    if let Some((root, config, rule)) = application.filter(|_| annotation_seed) {
        let generic_unrestricted = rule.applies_to_repository()
            && rule.exclude.is_empty()
            && (rule.include.is_empty()
                || rule
                    .include
                    .iter()
                    .any(|value| matches!(value.as_str(), "**" | "**/*")));
        let requires_inference = rule
            .projects
            .iter()
            .filter_map(|name| config.projects.get(name))
            .any(|project| {
                project.root.is_none()
                    && matches!(
                        project.type_,
                        Some(ProjectType::Nextjs | ProjectType::Remix | ProjectType::Vitejs)
                    )
            });
        if !generic_unrestricted && !requires_inference {
            // Rule dispatch owns inferred roots and preserves filter error ordering.
            scan.annotation_application =
                RulePathFilter::new(root, config, rule).ok().map(Arc::new);
        }
    }
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
