use super::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    helpers: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
}

pub(crate) fn helper_call_site_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut selected = Vec::new();
    for rule in config.rule_applications("postgres-lock-inventory") {
        let options: Options = rule.try_rule_options()?;
        if options.helpers.is_empty() {
            continue;
        }
        let paths = crate::codebase::rules::RuleGlobFilter::new(
            "postgres-lock-inventory",
            &options.include,
            &options.exclude,
        )?;
        selected.extend(
            crate::codebase::rules::selected_rule_files(root, config, rule, files)?
                .into_iter()
                .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
                .filter(|path| {
                    paths.includes(&crate::codebase::ts_source::relative_slash_path(root, path))
                }),
        );
    }
    selected.sort();
    selected.dedup();
    Ok(selected)
}

#[doc(hidden)]
pub fn configure_lock_inventory_helper_call_sites(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    let files = visible_paths.paths_for(root);
    plan.require_call_site_paths(&helper_call_site_files(root, config, &files)?);
    Ok(())
}
