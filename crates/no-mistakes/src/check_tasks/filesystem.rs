use super::{rule_configured, CheckTask};
use anyhow::Result;
use no_mistakes::codebase::rules::{self, RuleFinding};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::time::Instant;

const FILESYSTEM_RULE_IDS: &[&str] = &[
    rules::AGENTS_MD_MAX_SIZE,
    rules::BANNED_PATHS,
    rules::github_actions_pinned_hash::RULE_ID,
    rules::BANNED_RENAMED_FILES,
    rules::CONFIG_PATH_REFERENCES,
    rules::DOC_CONSISTENCY,
    rules::FILE_EXTENSION_POLICY,
    rules::FINITE_SET_CONSISTENCY,
    rules::FORBIDDEN_WORKSPACE_CLOSURE,
    rules::INTEGRATION_TEST_NO_MOCKS,
    rules::LOCKFILE_ALLOWLIST,
    rules::MARKDOWN_LINK_DISPLAY_TEXT,
    rules::NO_EMPTY_OR_COMMENTS_ONLY_FILES,
    rules::NO_GIT_IDENTITY_MUTATION,
    rules::PACKAGE_JSON_REGISTRY_ONLY,
    rules::PACKAGE_JSON_WORKSPACE_COVERAGE,
    rules::REQUIRED_COMPANION_IMPORTS,
    rules::REQUIRE_FILES_IN_SUBDIRS,
    rules::REQUIRE_TEST_PER_SUBDIR,
    rules::REQUIRED_DOC_SECTION,
    rules::REQUIRED_LOCAL_DOCS,
    rules::RUST_MAX_LINES_PER_FILE,
    rules::RUST_NO_INLINE_ALLOWS,
    rules::RUST_NO_INLINE_TESTS,
    rules::SHELLCHECK_RUNNER,
    rules::STRICT_PACKAGE_LAYOUT,
    rules::STRUCTURED_CONFIG_POLICY,
    rules::TEST_EMAIL_DOMAIN_POLICY,
    rules::TSCONFIG_ALIAS_FOLDER_MAPPING,
    rules::VITEST_CI_PATH_COVERAGE,
    rules::VITEST_PROJECT_MAPPING,
    rules::VITEST_TEST_CORRESPONDENCE,
    rules::WORKSPACE_PACKAGE_CYCLES,
];

pub(crate) fn run_filesystem_rules_check(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    visible_paths: &no_mistakes::codebase::ts_source::VisiblePathSnapshot,
    sources: std::sync::Arc<no_mistakes::codebase::ts_source::SourceStore>,
    vitest_projects: Option<&rules::PreparedVitestProjectCatalog>,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let start = Instant::now();
    let findings = if enabled {
        rules::run_filesystem_rules_with_config_snapshot_catalog_and_sources(
            root,
            config,
            files,
            visible_paths,
            vitest_projects,
            sources,
        )?
    } else {
        Vec::new()
    };
    Ok(CheckTask {
        findings,
        warning: None,
        duration: start.elapsed(),
    })
}

pub(crate) fn filesystem_rules_configured(config: &NoMistakesConfig) -> bool {
    FILESYSTEM_RULE_IDS
        .iter()
        .any(|rule_id| rule_configured(config, rule_id))
}
