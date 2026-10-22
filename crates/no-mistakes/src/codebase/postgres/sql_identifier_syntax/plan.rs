use super::EmbeddedSqlOptions;
use crate::codebase::rules::{path_filter::RulePathFilter, RuleGlobFilter};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::{collections::BTreeMap, path::Path, sync::Arc};

#[derive(Clone)]
pub struct SqlIdentifierScan {
    pub(crate) index: usize,
    paths: Arc<RuleGlobFilter>,
    application: Arc<RulePathFilter>,
}
pub type SqlIdentifierScans = BTreeMap<EmbeddedSqlOptions, Vec<SqlIdentifierScan>>;
impl SqlIdentifierScan {
    pub(crate) fn new(
        root: &Path,
        config: &NoMistakesConfig,
        application: &RuleDef,
        index: usize,
        include: &[String],
        exclude: &[String],
    ) -> Result<Self> {
        Ok(Self {
            index,
            paths: Arc::new(RuleGlobFilter::new(&application.rule, include, exclude)?),
            application: Arc::new(RulePathFilter::new(root, config, application)?),
        })
    }
    pub(crate) fn includes(&self, root: &Path, path: &Path) -> bool {
        self.paths
            .includes(&crate::codebase::ts_source::relative_slash_path(root, path))
            && self.application.is_match(path)
    }
}
pub(crate) fn merge(existing: &mut SqlIdentifierScans, additional: SqlIdentifierScans) {
    for (profile, scans) in additional {
        let slot = existing.entry(profile).or_default();
        for scan in scans {
            if !slot.iter().any(|current| current.index == scan.index) {
                slot.push(scan);
            }
        }
    }
}
