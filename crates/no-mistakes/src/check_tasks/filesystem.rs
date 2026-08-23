use super::{rule_configured, CheckTask};
use anyhow::Result;
use no_mistakes::codebase::rules::{self, RuleFinding};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

const FILESYSTEM_RULE_IDS: &[&str] = &[
    rules::AGENTS_MD_MAX_SIZE,
    rules::BANNED_PATHS,
    rules::GITHUB_ACTIONS_ACTION_TIMEOUT_PAIR,
    rules::GITHUB_ACTIONS_COMPOSITE_STEP_SCHEMA,
    rules::GITHUB_ACTIONS_JOB_TIMEOUTS,
    rules::GITHUB_ACTIONS_TEST_TIMEOUT_LITERALS,
    rules::github_actions_pinned_hash::RULE_ID,
    rules::VERSION_PIN_CONSISTENCY,
    rules::BANNED_RENAMED_FILES,
    rules::CONFIG_PATH_REFERENCES,
    rules::CSHARP_MAX_LINES_PER_FILE,
    rules::DOC_CONSISTENCY,
    rules::FILE_EXTENSION_POLICY,
    rules::FINITE_SET_CONSISTENCY,
    rules::FORBIDDEN_WORKSPACE_CLOSURE,
    rules::INTEGRATION_TEST_NO_MOCKS,
    rules::LOCKFILE_ALLOWLIST,
    rules::MARKDOWN_CHILD_LINKS,
    rules::MARKDOWN_EVAL_TESTS,
    rules::MARKDOWN_LINK_DISPLAY_TEXT,
    rules::MARKDOWN_MERMAID_VALIDATION,
    rules::MARKDOWN_REACHABILITY,
    rules::MARKDOWN_STRUCTURE_BUDGET,
    rules::NEXTJS_REDIRECT_DESTINATIONS,
    rules::NO_EMPTY_OR_COMMENTS_ONLY_FILES,
    rules::NO_GIT_IDENTITY_MUTATION,
    rules::NO_MISTAKES_CONFIG,
    rules::NO_RAW_EPHEMERAL_PORT,
    rules::PACKAGE_JSON_REGISTRY_ONLY,
    rules::PACKAGE_JSON_WORKSPACE_COVERAGE,
    rules::POSTGRES_CONSTRAINT_VALIDATE,
    rules::POSTGRES_FK_INDEX,
    rules::POSTGRES_NO_GENERATED_COLUMN_WRITES,
    rules::POSTGRES_REDUNDANT_INDEX,
    rules::PRODUCTION_DEPENDENCY_DECLARATIONS,
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
    rules::TEST_NO_DEPENDENCY_PINS,
    rules::POSTGRES_LOCK_ORDERING,
    rules::POSTGRES_NO_OFFSET,
    rules::TSCONFIG_ALIAS_FOLDER_MAPPING,
    rules::TSCONFIG_FILE_COVERAGE,
    rules::TSCONFIG_GATE_COVERAGE,
    rules::VITEST_CI_PATH_COVERAGE,
    rules::VITEST_PROJECT_MAPPING,
    rules::VITEST_TEST_CORRESPONDENCE,
    rules::WORKSPACE_PACKAGE_CYCLES,
];

pub(crate) fn run_filesystem_rules_check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.filesystem_rules",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || {
            run_enabled_filesystem_rules(
                root,
                config,
                enabled,
                files,
                prepared,
                facts,
                defer_suppression,
            )
        },
    );
    let findings = findings?;
    Ok(CheckTask {
        findings,
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        warning: None,
        duration,
    })
}

fn run_enabled_filesystem_rules(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    if !enabled {
        return Ok(Vec::new());
    }
    if defer_suppression {
        rules::run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression(
            root, config, files, prepared, facts,
        )
    } else {
        rules::run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
            root, config, files, prepared, facts,
        )
    }
}

pub(crate) fn filesystem_rules_configured(config: &NoMistakesConfig) -> bool {
    FILESYSTEM_RULE_IDS
        .iter()
        .any(|rule_id| rule_configured(config, rule_id))
}
