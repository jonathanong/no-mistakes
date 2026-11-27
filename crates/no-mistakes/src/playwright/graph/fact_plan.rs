use crate::codebase::check_facts::PlaywrightFactPlan;
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::playwright::config::Settings;
use std::path::Path;

pub(crate) fn fact_plan(
    root: &Path,
    settings: &[Settings],
    visible_paths: &VisiblePathSnapshot,
) -> anyhow::Result<Option<PlaywrightFactPlan>> {
    let mut merged: Option<PlaywrightFactPlan> = None;
    for settings in settings {
        let plan = merged.get_or_insert_with(Default::default);
        crate::playwright::analysis::pipeline::extend_standalone_fact_plan(
            plan,
            root,
            settings,
            crate::playwright::analysis::types::UniqueSelectorPolicy::default(),
            visible_paths,
        )?;
    }
    Ok(merged)
}
