use super::{extract, Options, RULE_ID, TS_CALL_FIRST_STRING_ARGUMENT};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Request boundaries use this before collection so the finite-set rule can
/// borrow the shared fact map instead of parsing its configured files itself.
#[doc(hidden)]
pub fn required_call_site_fact_files(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    try_required_call_site_fact_files(root, config).unwrap_or_default()
}

/// Fallible counterpart used by request boundaries that must surface malformed
/// rule options as configuration errors.
pub fn try_required_call_site_fact_files(
    root: &Path,
    config: &NoMistakesConfig,
) -> Result<Vec<PathBuf>> {
    collect(root, config, extract::resolve_spec_files)
}

pub(crate) fn call_site_fact_files_from_inventory(
    root: &Path,
    config: &NoMistakesConfig,
    inventory: &crate::codebase::ts_source::FileInventory,
) -> Result<Vec<PathBuf>> {
    collect(root, config, |root, file, target_roots| {
        let present = |path: &Path| {
            inventory
                .classification_for_path(path)
                .is_some_and(|kind| kind.target_is_file())
        };
        let repo_path = root.join(file);
        if present(&repo_path) {
            return vec![repo_path];
        }
        let paths = target_roots
            .iter()
            .filter(|target| target.as_path() != root)
            .map(|target| target.join(file))
            .filter(|path| present(path))
            .collect::<Vec<_>>();
        if paths.is_empty() {
            vec![repo_path]
        } else {
            paths
        }
    })
}

fn collect(
    root: &Path,
    config: &NoMistakesConfig,
    resolve: impl Fn(&Path, &str, &[PathBuf]) -> Vec<PathBuf>,
) -> Result<Vec<PathBuf>> {
    let resolve = &resolve;
    let paths: Result<Vec<Vec<PathBuf>>> = config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| {
            let opts: Options = rule.try_rule_options()?;
            opts.validate()?;
            if config.effective_rule_analysis(rule)?.effort == crate::config::v2::Effort::Low {
                return Ok(Vec::new());
            }
            let target_roots = super::super::target_roots(root, config, rule);
            Ok(opts
                .sets
                .into_iter()
                .filter(|spec| spec.kind == TS_CALL_FIRST_STRING_ARGUMENT)
                .flat_map(move |spec| resolve(root, &spec.file, &target_roots))
                .collect())
        })
        .collect();
    let mut paths = paths?.into_iter().flatten().collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}
