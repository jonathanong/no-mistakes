use super::PlaywrightFactPlan;
use std::collections::BTreeMap;
use std::sync::Arc;

impl PlaywrightFactPlan {
    pub(crate) fn set_matcher_warnings(&mut self, warnings: Vec<String>) {
        self.add_matcher_warnings(None, warnings);
    }

    pub(crate) fn add_matcher_warnings(&mut self, project: Option<String>, warnings: Vec<String>) {
        let stored = Arc::make_mut(&mut self.matcher_warnings)
            .entry(project)
            .or_default();
        stored.extend(warnings);
        stored.sort();
        stored.dedup();
    }

    pub(crate) fn matcher_warnings(&self) -> Arc<BTreeMap<Option<String>, Vec<String>>> {
        Arc::clone(&self.matcher_warnings)
    }
}
