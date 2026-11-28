use super::options::EmbeddedSqlOptions;
use crate::codebase::rules::{path_filter::RulePathFilter, RuleGlobFilter};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::{collections::BTreeMap, path::Path, sync::Arc};

#[derive(Clone)]
pub struct SqlIdentifierScan {
    pub index: usize,
    pub rule: String,
    pub application_index: Option<usize>,
    paths: Arc<RuleGlobFilter>,
    application: Arc<RulePathFilter>,
    typed_executor_types: Arc<Vec<crate::config::v2::schema::EffectTargetConfig>>,
    declared_factories: Arc<Vec<crate::config::v2::schema::EffectTargetConfig>>,
}
pub type SqlIdentifierScans = BTreeMap<EmbeddedSqlOptions, Vec<SqlIdentifierScan>>;
impl SqlIdentifierScan {
    pub fn new(
        root: &Path,
        config: &NoMistakesConfig,
        application: &RuleDef,
        index: usize,
        include: &[String],
        exclude: &[String],
    ) -> Result<Self> {
        Ok(Self {
            index,
            rule: application.rule.clone(),
            application_index: config
                .rule_applications(&application.rule)
                .iter()
                .position(|rule| std::ptr::eq(*rule, application)),
            paths: Arc::new(RuleGlobFilter::new(&application.rule, include, exclude)?),
            application: Arc::new(RulePathFilter::new(root, config, application)?),
            typed_executor_types: Arc::new(Vec::new()),
            declared_factories: Arc::new(Vec::new()),
        })
    }
    pub fn with_typed_executor_types(
        mut self,
        typed_executor_types: &[crate::config::v2::schema::EffectTargetConfig],
    ) -> Self {
        let mut types = typed_executor_types.to_vec();
        types.sort_by(|left, right| {
            (&left.module, &left.export).cmp(&(&right.module, &right.export))
        });
        types.dedup_by(|left, right| left.module == right.module && left.export == right.export);
        self.typed_executor_types = Arc::new(types);
        self
    }
    pub fn typed_executor_types(&self) -> &[crate::config::v2::schema::EffectTargetConfig] {
        &self.typed_executor_types
    }
    pub fn with_declared_factories(
        mut self,
        factories: &[crate::config::v2::schema::EffectTargetConfig],
    ) -> Self {
        let mut factories = factories.to_vec();
        factories.sort_by(|left, right| {
            (&left.module, &left.export).cmp(&(&right.module, &right.export))
        });
        factories
            .dedup_by(|left, right| left.module == right.module && left.export == right.export);
        self.declared_factories = Arc::new(factories);
        self
    }
    pub fn declared_factories(&self) -> &[crate::config::v2::schema::EffectTargetConfig] {
        &self.declared_factories
    }
    pub fn includes(&self, root: &Path, path: &Path) -> bool {
        self.paths
            .includes(&crate::codebase::ts_source::relative_slash_path(root, path))
            && self.application.is_match(path)
    }
}
pub fn merge(existing: &mut SqlIdentifierScans, additional: SqlIdentifierScans) {
    for (profile, scans) in additional {
        let slot = existing.entry(profile).or_default();
        for scan in scans {
            if !slot.iter().any(|current| current.index == scan.index) {
                slot.push(scan);
            }
        }
    }
}

pub fn selected(root: &Path, path: &Path, plan: &super::CheckFactPlan) -> bool {
    plan.sql_identifier_profiles
        .values()
        .any(|scans| scans.iter().any(|scan| scan.includes(root, path)))
}
