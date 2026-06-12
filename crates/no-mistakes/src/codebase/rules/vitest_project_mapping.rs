mod projects;

use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::{config as integration_config, project_config, types::Framework};
use anyhow::Result;
use projects::{build_project_globs, matching_projects};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "vitest-project-mapping";

const DEFAULT_TEST_EXTENSIONS: &[&str] = &[
    ".test.mts",
    ".test.cts",
    ".test.ctsx",
    ".test.ts",
    ".test.tsx",
    ".test.mjs",
    ".test.cjs",
    ".test.js",
    ".test.jsx",
    ".spec.mts",
    ".spec.cts",
    ".spec.ctsx",
    ".spec.ts",
    ".spec.tsx",
    ".spec.mjs",
    ".spec.cjs",
    ".spec.js",
    ".spec.jsx",
];
const DEFAULT_TEST_DIR_EXTENSIONS: &[&str] = &[
    ".mts", ".cts", ".ctsx", ".ts", ".tsx", ".mjs", ".cjs", ".js", ".jsx",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) scopes: Vec<String>,
    pub(crate) test_extensions: Vec<String>,
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
            scan(root, config, &opts, &files)
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
) -> Result<Vec<RuleFinding>> {
    let needs_config_projects = config.tests.vitest.configs.is_none()
        || config.tests.vitest.projects.is_empty()
        || config
            .tests
            .vitest
            .projects
            .values()
            .any(|policy| policy.include.is_empty());
    let mut projects = if needs_config_projects {
        project_config::load_projects(
            root,
            Framework::Vitest,
            config.tests.vitest.configs.as_ref(),
        )?
    } else {
        Vec::new()
    };
    for (project_name, policy) in &config.tests.vitest.projects {
        if let Some(project) = integration_config::configured_project(root, project_name, policy) {
            projects.push(project);
        }
    }
    if projects.is_empty() {
        return Ok(Vec::new());
    }

    let project_globs = build_project_globs(&projects)?;
    let test_extensions = test_extensions(opts);

    let mut findings = Vec::new();
    for path in files {
        let rel = relative_slash_path(root, path);
        if !is_test_file(&rel, &test_extensions, opts.test_extensions.is_empty())
            || !in_scope(&rel, &opts.scopes)
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
    let scope = scope.trim_matches('/');
    scope.is_empty() || rel == scope || rel.starts_with(&format!("{scope}/"))
}

#[cfg(test)]
mod tests;
