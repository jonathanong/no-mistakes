pub mod agents_md_max_size;
pub mod banned_paths;
pub mod banned_renamed_files;
pub mod config_path_references;
pub mod doc_consistency;
pub mod file_extension_policy;
mod file_matching;
pub mod finite_set_consistency;
pub mod forbidden_dependencies;
pub mod forbidden_workspace_closure;
pub mod github_actions_pinned_hash;
mod ids;
pub mod integration_test_no_mocks;
pub mod lockfile_allowlist;
pub mod markdown_link_display_text;
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
pub mod test_email_domain_policy;
pub mod test_no_unmocked_dynamic_imports;
pub mod tsconfig_alias_folder_mapping;
pub mod vitest_ci_path_coverage;
mod vitest_project_catalog;
pub mod vitest_project_mapping;
pub mod vitest_test_correspondence;
pub mod workspace_package_cycles;

pub mod filesystem_dispatch;
pub(crate) mod path_filter;
mod run;
mod suppression;

use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub use filesystem_dispatch::{
    run_filesystem_rules, run_filesystem_rules_with_config,
    run_filesystem_rules_with_config_and_snapshot,
    run_filesystem_rules_with_config_snapshot_and_vitest_catalog,
    run_filesystem_rules_with_config_snapshot_catalog_and_sources, run_filesystem_rules_with_files,
};
pub use ids::*;
pub(crate) use run::canonical_graph_plan;
#[doc(hidden)]
pub use run::run_check_with_config_facts_playwright_and_graph;
pub use run::{
    run_check, run_check_with_config_and_facts_and_playwright, run_check_with_facts,
    run_check_with_facts_and_playwright, PreparedRulesCheck,
};
#[doc(hidden)]
pub use vitest_project_catalog::{prepare_vitest_project_catalog, PreparedVitestProjectCatalog};

pub(crate) use file_matching::matching_files;
pub(crate) use suppression::{
    suppress_rule_findings, suppress_rule_findings_with_source,
    suppress_rule_findings_with_sources_except,
};

pub(crate) fn source_store_for_files(
    files: &[PathBuf],
) -> std::sync::Arc<crate::codebase::ts_source::SourceStore> {
    std::sync::Arc::new(crate::codebase::ts_source::SourceStore::new(
        std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(files)),
    ))
}

pub(crate) fn read_source(
    sources: &crate::codebase::ts_source::SourceStore,
    path: &Path,
) -> Option<std::sync::Arc<str>> {
    sources.read_path(path)?.ok()
}

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
    let mut inferred_roots = crate::codebase::config::InferredRoots::default();
    target_roots_with_inferred(root, config, rule, &mut inferred_roots)
}

pub(crate) fn target_roots_with_inferred(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    inferred_roots: &mut crate::codebase::config::InferredRoots,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if rule.applies_to_repository() {
        roots.push(root.to_path_buf());
    }
    for project_name in &rule.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        if let Some(project_root) = target_project_root(root, project, inferred_roots) {
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
        return inferred_roots.nextjs_root(root);
    }
    if project.type_ == Some(crate::config::v2::schema::ProjectType::Remix) {
        return inferred_roots.remix_root(root);
    }
    if project.type_ == Some(crate::config::v2::schema::ProjectType::Vitejs) {
        return inferred_roots.vitejs_root(root);
    }
    Some(root.to_path_buf())
}

pub(crate) fn sort_findings(findings: &mut Vec<RuleFinding>) {
    findings.sort();
    findings.dedup();
}

#[cfg(test)]
mod suppression_tests;
#[cfg(test)]
mod target_roots_tests;
#[cfg(test)]
mod tests;
