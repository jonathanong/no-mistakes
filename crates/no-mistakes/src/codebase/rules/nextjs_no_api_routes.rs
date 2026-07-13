use super::RuleFinding;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_source::{
    has_disable_file_comment, has_disable_line_comment, relative_slash_path,
};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

mod aggregate;
pub(crate) use aggregate::{check_with_facts, check_with_facts_and_inferred};

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

fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    aggregate::check_files(root, config, files)
}

fn finding_for_file(
    root: &Path,
    target_roots: &[PathBuf],
    path: &Path,
    source: &str,
) -> Option<RuleFinding> {
    if has_disable_file_comment(source, RULE_ID) || has_disable_line_comment(source, 1, RULE_ID) {
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
