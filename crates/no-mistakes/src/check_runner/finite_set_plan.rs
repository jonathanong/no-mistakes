use no_mistakes::codebase::check_facts::CheckFactPlan;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(crate) struct PreparedFactDemand {
    call_site_files: Vec<PathBuf>,
    needs_other_facts: bool,
}

impl PreparedFactDemand {
    pub(crate) fn needs_shared_facts(&self) -> bool {
        self.needs_other_facts || !self.call_site_files.is_empty()
    }

    /// Files needed by the request's primary shared-facts consumers.
    ///
    /// Configured finite-set call sources are deliberately excluded here. A
    /// call source can be under `filesystem.skipDirectories`; retain only
    /// call sources that discovery already admitted to the primary scope.
    pub(crate) fn primary_files(&self, discovered: Vec<PathBuf>) -> Vec<PathBuf> {
        if self.needs_other_facts {
            return discovered;
        }
        self.call_site_files
            .iter()
            .filter(|path| discovered.contains(path))
            .cloned()
            .collect()
    }

    /// Call-source files absent from the request's primary fact and graph scopes.
    ///
    /// A graph-only source has richer graph facts (for example, imports) than
    /// the call-site-only supplemental variant. Recollecting it here would
    /// sparsely overwrite that entry when the fact maps are combined.
    pub(crate) fn supplemental_call_site_files(
        &self,
        primary_files: &[PathBuf],
        graph_files: &[PathBuf],
    ) -> Vec<PathBuf> {
        self.call_site_files
            .iter()
            .filter(|path| !primary_files.contains(path) && !graph_files.contains(path))
            .cloned()
            .collect()
    }
}

pub(crate) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    plan: &mut CheckFactPlan,
    graph_configured: bool,
    playwright_configured: bool,
) -> PreparedFactDemand {
    let needs_other_facts =
        graph_configured || playwright_configured || super::enabled::plan_requests_facts(plan);
    let files = no_mistakes::codebase::rules::finite_set_consistency::required_call_site_fact_files(
        root, config,
    );
    if !files.is_empty() {
        plan.graph.call_sites = true;
    }
    PreparedFactDemand {
        call_site_files: files,
        needs_other_facts,
    }
}

pub(crate) fn no_analysis_requested(
    needs_shared_facts: bool,
    filesystem_rules_configured: bool,
    playwright_rules_configured: bool,
) -> bool {
    !needs_shared_facts && !filesystem_rules_configured && !playwright_rules_configured
}
