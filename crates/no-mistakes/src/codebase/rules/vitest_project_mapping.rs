mod project_sources;
mod projects;

use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use project_sources::vitest_projects;
use projects::{build_project_globs, matching_projects};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "vitest-project-mapping";

const DEFAULT_TEST_EXTENSIONS: &[&str] = &[
    ".test.mts",
    ".test.mtsx",
    ".test.cts",
    ".test.ctsx",
    ".test.ts",
    ".test.tsx",
    ".test.mjs",
    ".test.mjsx",
    ".test.cjs",
    ".test.cjsx",
    ".test.js",
    ".test.jsx",
    ".spec.mts",
    ".spec.mtsx",
    ".spec.cts",
    ".spec.ctsx",
    ".spec.ts",
    ".spec.tsx",
    ".spec.mjs",
    ".spec.mjsx",
    ".spec.cjs",
    ".spec.cjsx",
    ".spec.js",
    ".spec.jsx",
];
const DEFAULT_TEST_DIR_EXTENSIONS: &[&str] = &[
    ".mts", ".mtsx", ".cts", ".ctsx", ".ts", ".tsx", ".mjs", ".mjsx", ".cjs", ".cjsx", ".js",
    ".jsx",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) scopes: Vec<String>,
    pub(crate) test_extensions: Vec<String>,
    pub(crate) explicit_projects_only: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, config, &opts, &files, &target_roots)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let projects = vitest_projects(root, config, opts)?;
    if projects.is_empty() {
        return Ok(Vec::new());
    }

    let project_globs = build_project_globs(&projects)?;
    let test_extensions = test_extensions(opts);

    let mut findings = Vec::new();
    for path in files {
        let rel = relative_slash_path(root, path);
        let match_rels = relative_paths_for_matching(root, path, target_roots);
        if !is_test_file(&rel, &test_extensions, opts.test_extensions.is_empty())
            || !match_rels.iter().any(|rel| in_scope(rel, &opts.scopes))
        {
            continue;
        }
        let matches = matching_projects(&rel, &project_globs);
        match matches.as_slice() {
            [_one] => {}
            [] => findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!("{rel}: Vitest test file does not map to any Vitest project"),
                import: None,
                target: None,
            }),
            many => findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!(
                    "{rel}: Vitest test file maps to multiple Vitest projects: {}",
                    many.join(", ")
                ),
                import: None,
                target: Some(many.join(",")),
            }),
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn test_extensions(opts: &Options) -> Vec<&str> {
    if opts.test_extensions.is_empty() {
        DEFAULT_TEST_EXTENSIONS.to_vec()
    } else {
        opts.test_extensions.iter().map(String::as_str).collect()
    }
}

fn is_test_file(rel: &str, test_extensions: &[&str], default_extensions: bool) -> bool {
    test_extensions
        .iter()
        .any(|extension| rel.ends_with(extension))
        || (default_extensions
            && (rel.starts_with("__tests__/") || rel.contains("/__tests__/"))
            && DEFAULT_TEST_DIR_EXTENSIONS
                .iter()
                .any(|extension| rel.ends_with(extension)))
}

fn in_scope(rel: &str, scopes: &[String]) -> bool {
    scopes.is_empty() || scopes.iter().any(|scope| rel_in_scope(rel, scope))
}

fn rel_in_scope(rel: &str, scope: &str) -> bool {
    let scope = normalize_scope(scope);
    scope.is_empty() || rel == scope || rel.starts_with(&format!("{scope}/"))
}

pub(super) fn normalize_scope(scope: &str) -> String {
    let mut parts = Vec::new();
    for part in scope.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn relative_paths_for_matching(root: &Path, file: &Path, target_roots: &[PathBuf]) -> Vec<String> {
    let mut paths = target_roots
        .iter()
        .filter(|target_root| file.starts_with(target_root))
        .map(|target_root| relative_slash_path(target_root, file))
        .collect::<Vec<_>>();
    let repo_rel = relative_slash_path(root, file);
    if !paths.contains(&repo_rel) {
        paths.push(repo_rel);
    }
    paths
}

#[cfg(test)]
mod tests;
