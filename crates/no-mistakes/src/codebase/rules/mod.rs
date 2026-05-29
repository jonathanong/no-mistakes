pub mod agents_md_max_size;
pub mod banned_renamed_files;
pub mod doc_consistency;
pub mod file_extension_policy;
pub mod forbidden_dependencies;
pub mod lockfile_allowlist;
pub mod nextjs_no_api_routes;
pub mod nextjs_no_caching;
pub mod no_empty_or_comments_only_files;
pub mod no_git_identity_mutation;
pub mod package_json_registry_only;
pub mod require_files_in_subdirs;
pub mod require_storybook_stories;
pub mod require_test_per_subdir;
pub mod required_local_docs;
pub mod rust_max_lines_per_file;
pub mod rust_no_inline_allows;
pub mod rust_no_inline_tests;
pub mod server_route_client_boundary;
pub mod shellcheck_runner;
pub mod strict_package_layout;
pub mod test_no_unmocked_dynamic_imports;
pub mod tsconfig_alias_folder_mapping;
pub mod vitest_test_correspondence;

pub mod filesystem_dispatch;
pub(crate) mod path_filter;
mod run;

use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use filesystem_dispatch::{run_filesystem_rules, run_filesystem_rules_with_files};
pub use run::{run_check, run_check_with_facts};

pub use agents_md_max_size::RULE_ID as AGENTS_MD_MAX_SIZE;
pub use banned_renamed_files::RULE_ID as BANNED_RENAMED_FILES;
pub use doc_consistency::RULE_ID as DOC_CONSISTENCY;
pub use file_extension_policy::RULE_ID as FILE_EXTENSION_POLICY;
pub use forbidden_dependencies::RULE_ID as FORBIDDEN_DEPENDENCIES;
pub use lockfile_allowlist::RULE_ID as LOCKFILE_ALLOWLIST;
pub use nextjs_no_api_routes::RULE_ID as NEXTJS_NO_API_ROUTES;
pub use nextjs_no_caching::RULE_ID as NEXTJS_NO_CACHING;
pub use no_empty_or_comments_only_files::RULE_ID as NO_EMPTY_OR_COMMENTS_ONLY_FILES;
pub use no_git_identity_mutation::RULE_ID as NO_GIT_IDENTITY_MUTATION;
pub use package_json_registry_only::RULE_ID as PACKAGE_JSON_REGISTRY_ONLY;
pub use require_files_in_subdirs::RULE_ID as REQUIRE_FILES_IN_SUBDIRS;
pub use require_storybook_stories::RULE_ID as REQUIRE_STORYBOOK_STORIES;
pub use require_test_per_subdir::RULE_ID as REQUIRE_TEST_PER_SUBDIR;
pub use required_local_docs::REQUIRED_DOC_SECTION_RULE_ID as REQUIRED_DOC_SECTION;
pub use required_local_docs::RULE_ID as REQUIRED_LOCAL_DOCS;
pub use rust_max_lines_per_file::RULE_ID as RUST_MAX_LINES_PER_FILE;
pub use rust_no_inline_allows::RULE_ID as RUST_NO_INLINE_ALLOWS;
pub use rust_no_inline_tests::RULE_ID as RUST_NO_INLINE_TESTS;
pub use server_route_client_boundary::RULE_ID as SERVER_ROUTE_CLIENT_BOUNDARY;
pub use shellcheck_runner::RULE_ID as SHELLCHECK_RUNNER;
pub use strict_package_layout::RULE_ID as STRICT_PACKAGE_LAYOUT;
pub use test_no_unmocked_dynamic_imports::RULE_ID as TEST_NO_UNMOCKED_DYNAMIC_IMPORTS;
pub use tsconfig_alias_folder_mapping::RULE_ID as TSCONFIG_ALIAS_FOLDER_MAPPING;
pub use vitest_test_correspondence::RULE_ID as VITEST_TEST_CORRESPONDENCE;

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
