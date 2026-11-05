use super::super::{allow_audit::AllowAudit, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::path::Path;

pub(crate) struct ExceptionAudit {
    pub(crate) audit: AllowAudit,
    selectors: Vec<String>,
    universe: Option<std::sync::Arc<Vec<std::path::PathBuf>>>,
    checked: dashmap::DashSet<std::path::PathBuf>,
}

impl ExceptionAudit {
    pub(crate) fn new(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        sources: &SourceStore,
        option: &str,
        selectors: &[String],
    ) -> Self {
        Self {
            audit: AllowAudit::new(root, config, rule, Some(sources), option, selectors.len()),
            selectors: selectors.to_vec(),
            universe: if selectors.is_empty() {
                None
            } else {
                Some(sources.inventory().paths())
            },
            checked: dashmap::DashSet::new(),
        }
    }

    pub(crate) fn checked(&self, path: &Path) {
        if !self.selectors.is_empty() {
            self.checked
                .insert(crate::codebase::ts_resolver::normalize_path(path));
        }
    }

    pub(crate) fn finish(
        &self,
        _sources: &SourceStore,
        domain: impl Fn(&Path) -> bool,
    ) -> Vec<RuleFinding> {
        let Some(universe) = &self.universe else {
            return Vec::new();
        };
        let mut matched = false;
        let complete = universe.iter().filter(|path| domain(path)).all(|path| {
            matched = true;
            self.checked.contains(path)
        });
        self.audit.finish(
            |_| complete && matched,
            |index| self.selectors[index].clone(),
        )
    }
}
