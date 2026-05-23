use super::RuleFinding;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "nextjs-no-api-routes";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let files =
        crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
    let files: Vec<_> = files
        .into_iter()
        .filter(|path| is_indexable(path))
        .collect();
    check_files(&root, config, &files)
}

pub(crate) fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut items = Vec::new();
    for path in shared.files() {
        let Some(facts) = shared.ts.get(path) else {
            continue;
        };
        let Some(source) = facts.source.as_ref() else {
            bail!("{} requires source facts for {}", RULE_ID, path.display());
        };
        items.push(SourceItem {
            path: path.as_path(),
            source: source.as_str(),
        });
    }
    check_items(&root, config, &items, |item| item.path, |item| item.source)
}

struct SourceItem<'a> {
    path: &'a Path,
    source: &'a str,
}

struct LoadedSourceItem {
    path: PathBuf,
    source: String,
}

fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let items: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            std::fs::read_to_string(path)
                .ok()
                .map(|source| LoadedSourceItem {
                    path: path.clone(),
                    source,
                })
        })
        .collect();
    check_items(
        root,
        config,
        &items,
        |item| item.path.as_path(),
        |item| item.source.as_str(),
    )
}

fn check_items<T>(
    root: &Path,
    config: &NoMistakesConfig,
    items: &[T],
    path_for: impl Fn(&T) -> &Path + Sync,
    source_for: impl Fn(&T) -> &str + Sync,
) -> Result<Vec<RuleFinding>>
where
    T: Sync,
{
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let target_roots = super::target_roots(root, config, rule);
        findings.extend(
            items
                .par_iter()
                .filter_map(|item| {
                    let path = path_for(item);
                    let source = source_for(item);
                    finding_for_file(root, &target_roots, path, source)
                })
                .collect::<Vec<_>>(),
        );
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn finding_for_file(
    root: &Path,
    target_roots: &[PathBuf],
    path: &Path,
    source: &str,
) -> Option<RuleFinding> {
    if has_disable_file_comment(source, RULE_ID) {
        return None;
    }
    if !target_roots
        .iter()
        .any(|target_root| path.starts_with(target_root))
    {
        return None;
    }
    if !is_nextjs_api_route(path, target_roots) {
        return None;
    }
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message:
            "Next.js API/server routes are disabled; move server endpoints out of the Next.js app"
                .to_string(),
        import: None,
        target: None,
    })
}

fn is_nextjs_api_route(path: &Path, target_roots: &[PathBuf]) -> bool {
    target_roots.iter().any(|target_root| {
        let Ok(rel) = path.strip_prefix(target_root) else {
            return false;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        is_app_route_handler(&rel) || is_pages_api_route(&rel)
    })
}

fn is_app_route_handler(rel: &str) -> bool {
    rel.starts_with("app/") && rel.rsplit('/').next().is_some_and(route_handler_filename)
        || rel.starts_with("src/app/") && rel.rsplit('/').next().is_some_and(route_handler_filename)
}

fn route_handler_filename(name: &str) -> bool {
    matches!(
        name,
        "route.js"
            | "route.jsx"
            | "route.ts"
            | "route.tsx"
            | "route.mjs"
            | "route.mts"
            | "route.cjs"
            | "route.cts"
    )
}

fn is_pages_api_route(rel: &str) -> bool {
    rel.starts_with("pages/api/") || rel.starts_with("src/pages/api/")
}

#[cfg(test)]
mod tests;
