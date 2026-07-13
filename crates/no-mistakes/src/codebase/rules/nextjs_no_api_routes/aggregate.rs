use super::{finding_for_file, is_nextjs_api_route, RuleFinding, RULE_ID};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(crate) fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(root, config, shared, None)
}

pub(crate) fn check_with_facts_and_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: &crate::codebase::config::InferredRoots,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(root, config, shared, Some(inferred_roots))
}

fn check_with_optional_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let target_roots = target_roots(&root, config, inferred_roots);
    let mut items = Vec::new();
    for path in shared.files() {
        if !is_nextjs_api_route(path, &target_roots) {
            continue;
        }
        let Some(facts) = shared.ts.get(path) else {
            continue;
        };
        if facts.parse_error.is_some() {
            continue;
        }
        let Some(source) = facts.source.as_ref() else {
            bail!("{} requires source facts for {}", RULE_ID, path.display());
        };
        items.push(SourceItem {
            path: path.as_path(),
            source: source.as_str(),
        });
    }
    check_items(
        &root,
        config,
        &items,
        |item| item.path,
        |item| item.source,
        inferred_roots,
    )
}

struct SourceItem<'a> {
    path: &'a Path,
    source: &'a str,
}

struct LoadedSourceItem {
    path: PathBuf,
    source: String,
}

pub(super) fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let target_roots = target_roots(root, config, None);
    let items: Vec<_> = files
        .par_iter()
        .filter(|path| {
            target_roots
                .iter()
                .any(|target_root| path.starts_with(target_root))
                && is_nextjs_api_route(path, &target_roots)
        })
        .map(|path| -> Result<LoadedSourceItem> {
            let source = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            Ok(LoadedSourceItem {
                path: path.clone(),
                source,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    check_items(
        root,
        config,
        &items,
        |item| item.path.as_path(),
        |item| item.source.as_str(),
        None,
    )
}

fn check_items<T>(
    root: &Path,
    config: &NoMistakesConfig,
    items: &[T],
    path_for: impl Fn(&T) -> &Path + Sync,
    source_for: impl Fn(&T) -> &str + Sync,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>>
where
    T: Sync,
{
    let mut findings = Vec::new();
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    for rule in config.rule_applications(RULE_ID) {
        let target_roots = crate::codebase::rules::target_roots_with_inferred(
            root,
            config,
            rule,
            &mut inferred_roots,
        );
        let filter = crate::codebase::rules::path_filter::RulePathFilter::new_with_inferred(
            root,
            config,
            rule,
            &mut inferred_roots,
        )?;
        findings.extend(
            items
                .par_iter()
                .filter_map(|item| {
                    let path = path_for(item);
                    let source = source_for(item);
                    filter
                        .is_match(path)
                        .then(|| finding_for_file(root, &target_roots, path, source))
                        .flatten()
                })
                .collect::<Vec<_>>(),
        );
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn target_roots(
    root: &Path,
    config: &NoMistakesConfig,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Vec<PathBuf> {
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    let mut roots: Vec<_> = config
        .rule_applications(RULE_ID)
        .into_iter()
        .flat_map(|rule| {
            crate::codebase::rules::target_roots_with_inferred(
                root,
                config,
                rule,
                &mut inferred_roots,
            )
        })
        .collect();
    roots.sort();
    roots.dedup();
    roots
}
