use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactPlan;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(crate) struct PreparedFactDemand {
    companion_import_files: Vec<PathBuf>,
    has_companion_plan: bool,
    call_site_files: Vec<PathBuf>,
    runner_config_files: Vec<PathBuf>,
    needs_other_facts: bool,
}

impl PreparedFactDemand {
    pub(crate) fn include_companion_plan(&mut self, plan: &CheckFactPlan) {
        self.has_companion_plan = !plan.companion_import_plans.is_empty();
        self.companion_import_files.clear();
        // Only a companion-only primary scope consumes this membership list.
        // Other consumers already retain the complete discovered scope, while
        // explicit historical import demand remains on the shared fact plan.
        if self.has_companion_plan && !self.needs_other_facts {
            self.companion_import_files
                .extend(plan.production_import_paths.iter().cloned());
        }
    }
    pub(crate) fn needs_shared_facts(&self) -> bool {
        self.needs_other_facts
            || self.has_companion_plan
            || !self.call_site_files.is_empty()
            || !self.runner_config_files.is_empty()
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
        let mut requested = self.call_site_files.clone();
        requested.extend(self.companion_import_files.iter().cloned());
        requested.extend(self.runner_config_files.iter().cloned());
        no_mistakes::codebase::check_facts::ordered_path_intersection(&requested, &discovered)
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
        no_mistakes::codebase::check_facts::ordered_path_exclusion(
            &self.call_site_files,
            primary_files,
            graph_files,
        )
    }
}

pub(crate) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    plan: &mut CheckFactPlan,
    graph_configured: bool,
    playwright_configured: bool,
) -> Result<PreparedFactDemand> {
    let needs_other_facts =
        graph_configured || playwright_configured || super::enabled::plan_requests_facts(plan);
    let files =
        no_mistakes::codebase::rules::finite_set_consistency::try_required_call_site_fact_files(
            root, config,
        )?;
    if !files.is_empty() {
        plan.graph.call_sites = true;
    }
    let mut demand = PreparedFactDemand {
        companion_import_files: Vec::new(),
        has_companion_plan: false,
        call_site_files: files,
        runner_config_files: plan
            .integration_runner_configs
            .as_ref()
            .map(|plan| plan.paths().cloned().collect())
            .unwrap_or_default(),
        needs_other_facts,
    };
    demand.include_companion_plan(plan);
    Ok(demand)
}

#[cfg(test)]
#[path = "tests/finite_set_plan.rs"]
mod tests;

pub(crate) fn no_analysis_requested(
    needs_shared_facts: bool,
    filesystem_rules_configured: bool,
    playwright_rules_configured: bool,
) -> bool {
    !needs_shared_facts && !filesystem_rules_configured && !playwright_rules_configured
}
