use super::super::{PlaywrightFactPlan, PlaywrightSettingsKey};
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::selectors::AppSelector;
use std::collections::HashMap;
use std::path::Path;

pub(super) struct SourceFacts {
    pub(super) selectors: HashMap<(PlaywrightSettingsKey, bool), Vec<AppSelector>>,
    pub(super) text_targets: HashMap<PlaywrightSettingsKey, Vec<AppTextTarget>>,
}

pub(super) fn collect(
    root: &Path,
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    playwright: Option<&PlaywrightFactPlan>,
) -> SourceFacts {
    let mut selectors = HashMap::new();
    let mut text_targets = HashMap::new();
    if let Some(plan) = playwright {
        for source_plan in plan.source_plans_for(path) {
            selectors
                .entry((source_plan.settings_key.clone(), source_plan.scan_html_ids))
                .or_insert_with(Vec::new)
                .extend(crate::playwright::selectors::extract_app_selectors_from_program_from_visible_deferred(
                    path,
                    source,
                    program,
                    &source_plan.selector_regexes,
                    &source_plan.visible_files,
                ));
            text_targets
                .entry(source_plan.settings_key.clone())
                .or_insert_with(Vec::new)
                .extend(
                    crate::playwright::analysis::app_text::extract_app_text_targets_from_program(
                        root,
                        path,
                        source,
                        &source_plan.settings,
                        program,
                    ),
                );
        }
    }
    for selectors in selectors.values_mut() {
        selectors.sort();
    }
    for targets in text_targets.values_mut() {
        targets.sort();
        targets.dedup();
    }
    SourceFacts {
        selectors,
        text_targets,
    }
}
