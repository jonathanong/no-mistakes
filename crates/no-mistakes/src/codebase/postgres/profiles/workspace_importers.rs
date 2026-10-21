use super::*;

/// Declare configured annotation importers using the request's existing inventory.
/// Explicit direct annotation requests retain unrestricted workspace compatibility.
#[doc(hidden)]
pub fn configure_prepared_postgres_plan_with_scope(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    let retain_masks =
        !plan.query_annotation || plan.query_annotation_workspace_importers.is_some();
    let masks = super::plan::configure_plan_and_masks(config, plan, retain_masks)?;
    assign_from_masks(root, config, all_files, plan, masks)
}

pub(super) fn assign_from_masks(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
    masks: super::plan::AnnotationMasks<'_>,
) -> Result<()> {
    let Some(existing) = &mut plan.query_annotation_workspace_importers else {
        return Ok(());
    };
    let mut selected = crate::fx::PathSet::default();
    for (rule, paths) in masks {
        for path in crate::codebase::rules::selected_rule_files(root, config, rule, all_files)? {
            if paths.includes(&crate::codebase::ts_source::relative_slash_path(
                root, &path,
            )) {
                selected.insert(crate::codebase::ts_resolver::normalize_path(&path));
            }
        }
    }
    if !selected.is_empty() {
        Arc::make_mut(existing).extend(selected);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
