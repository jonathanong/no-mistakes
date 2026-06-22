pub mod agents_md_max_size;
pub mod banned_paths;
pub mod banned_renamed_files;
pub mod config_path_references;
pub mod doc_consistency;
pub mod file_extension_policy;
mod file_matching;
pub mod finite_set_consistency;
pub mod forbidden_dependencies;
pub mod github_actions_pinned_hash;
mod ids;
pub mod lockfile_allowlist;
pub mod nextjs_no_api_routes;
pub mod nextjs_no_caching;
pub mod no_empty_or_comments_only_files;
pub mod no_git_identity_mutation;
pub mod package_json_registry_only;
pub mod package_json_workspace_coverage;
pub mod require_files_in_subdirs;
pub mod require_storybook_stories;
pub mod require_test_per_subdir;
pub mod required_companion_imports;
pub mod required_local_docs;
pub mod rust_max_lines_per_file;
pub mod rust_no_inline_allows;
pub mod rust_no_inline_tests;
pub mod rust_rules_combined;
pub mod server_route_client_boundary;
pub mod shellcheck_runner;
pub mod strict_package_layout;
pub mod structured_config_policy;
pub mod test_no_unmocked_dynamic_imports;
pub mod tsconfig_alias_folder_mapping;
pub mod vitest_project_mapping;
pub mod vitest_test_correspondence;
pub mod workspace_package_cycles;

pub mod filesystem_dispatch;
pub(crate) mod path_filter;
mod run;

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub use filesystem_dispatch::{run_filesystem_rules, run_filesystem_rules_with_files};
pub use ids::*;
pub use run::{run_check, run_check_with_facts};

pub(crate) use file_matching::matching_files;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleFinding {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

pub(crate) fn rule_enabled(config: &crate::config::v2::NoMistakesConfig, rule_id: &str) -> bool {
    config.rule_configured(rule_id)
}

pub(crate) fn target_roots(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if rule.applies_to_repository() {
        roots.push(root.to_path_buf());
    }
    let mut inferred_roots = crate::codebase::config::InferredRoots::default();
    for project_name in &rule.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        if let Some(project_root) = target_project_root(root, project, &mut inferred_roots) {
            roots.push(project_root);
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

pub(crate) fn file_allowed_by_roots_and_skip(
    root: &Path,
    skip: &HashSet<&str>,
    path: &Path,
    roots: &[PathBuf],
) -> bool {
    let mut matching_roots = roots.iter().filter(|rule_root| path.starts_with(rule_root));
    let Some(first_root) = matching_roots.next() else {
        return false;
    };

    if !crate::codebase::ts_source::is_under_skipped_dir(root, path, skip) {
        return true;
    }

    if !crate::codebase::ts_source::is_under_skipped_dir(first_root, path, skip) {
        return true;
    }

    matching_roots
        .any(|rule_root| !crate::codebase::ts_source::is_under_skipped_dir(rule_root, path, skip))
}

pub(crate) fn skip_dir_set(config: &crate::config::v2::NoMistakesConfig) -> HashSet<&str> {
    config
        .filesystem
        .skip_directories
        .iter()
        .map(String::as_str)
        .collect()
}

fn target_project_root(
    root: &Path,
    project: &crate::config::v2::schema::Project,
    inferred_roots: &mut crate::codebase::config::InferredRoots,
) -> Option<PathBuf> {
    if let Some(project_root) = project.root.as_deref() {
        return Some(root.join(project_root));
    }
    if project.type_ == Some(crate::config::v2::schema::ProjectType::Nextjs) {
        return inferred_roots
            .nextjs
            .get_or_insert_with(|| crate::codebase::config::infer_nextjs_root(root))
            .clone();
    }
    if project.type_ == Some(crate::config::v2::schema::ProjectType::Remix) {
        return inferred_roots
            .remix
            .get_or_insert_with(|| crate::codebase::config::infer_remix_root(root))
            .clone();
    }
    if project.type_ == Some(crate::config::v2::schema::ProjectType::Vitejs) {
        return inferred_roots
            .vitejs
            .get_or_insert_with(|| crate::codebase::config::infer_vitejs_root(root))
            .clone();
    }
    Some(root.to_path_buf())
}

pub(crate) fn sort_findings(findings: &mut Vec<RuleFinding>) {
    findings.sort();
    findings.dedup();
}

pub(crate) fn suppress_rule_findings(root: &Path, findings: &mut Vec<RuleFinding>) {
    let Some(root) = std::fs::canonicalize(root).ok() else {
        return;
    };
    let mut sources: HashMap<String, Option<String>> = HashMap::new();
    findings.retain(|finding| {
        let source = sources.entry(finding.file.clone()).or_insert_with(|| {
            source_path_for_finding(&root, &finding.file)
                .and_then(|path| std::fs::read_to_string(path).ok())
        });
        !source
            .as_deref()
            .is_some_and(|source| finding_is_suppressed(source, finding))
    });
}

fn source_path_for_finding(root: &Path, file: &str) -> Option<PathBuf> {
    let path = Path::new(file);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
                    | std::path::Component::ParentDir
            )
        })
    {
        return None;
    }
    let candidate = std::fs::canonicalize(root.join(path)).ok()?;
    let metadata = std::fs::metadata(&candidate).ok()?;
    (candidate.starts_with(root) && metadata.is_file()).then_some(candidate)
}

fn finding_is_suppressed(source: &str, finding: &RuleFinding) -> bool {
    let line = finding.line.try_into().ok();
    crate::codebase::ts_source::has_disable_file_comment(source, &finding.rule)
        || line.is_some_and(|line| {
            crate::codebase::ts_source::has_disable_comment(source, line, &finding.rule)
                || crate::codebase::ts_source::has_disable_line_comment(source, line, &finding.rule)
        })
}

#[cfg(test)]
mod suppression_tests;
#[cfg(test)]
mod target_roots_tests;
#[cfg(test)]
mod tests;
