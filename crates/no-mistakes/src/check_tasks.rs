use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::{self, RuleFinding};
use no_mistakes::codebase::unique_exports::{self, UniqueExportFinding};
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::integration_tests::{self, IntegrationFinding};
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const FILESYSTEM_RULE_IDS: &[&str] = &[
    rules::AGENTS_MD_MAX_SIZE,
    rules::BANNED_PATHS,
    rules::github_actions_pinned_hash::RULE_ID,
    rules::BANNED_RENAMED_FILES,
    rules::CONFIG_PATH_REFERENCES,
    rules::DOC_CONSISTENCY,
    rules::FILE_EXTENSION_POLICY,
    rules::FINITE_SET_CONSISTENCY,
    rules::LOCKFILE_ALLOWLIST,
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
    rules::TSCONFIG_ALIAS_FOLDER_MAPPING,
    rules::VITEST_CI_PATH_COVERAGE,
    rules::VITEST_PROJECT_MAPPING,
    rules::VITEST_TEST_CORRESPONDENCE,
    rules::WORKSPACE_PACKAGE_CYCLES,
];

pub(crate) struct CheckTask<T> {
    pub(crate) findings: T,
    pub(crate) warning: Option<String>,
    pub(crate) duration: Duration,
}

pub(crate) fn run_react_check(
    root: &std::path::Path,
    config: Option<&std::path::Path>,
    enabled: bool,
    facts: &CheckFactMap,
) -> Result<CheckTask<Vec<react_traits::Violation>>> {
    let start = Instant::now();
    let (findings, warning) = if enabled {
        match react_traits::run_check_with_facts(root, config, &[], false, facts) {
            Ok(findings) => (findings, None),
            Err(err) => (
                Vec::new(),
                Some(format!("warning: react check skipped: {err:#}")),
            ),
        }
    } else {
        (Vec::new(), None)
    };
    Ok(CheckTask {
        findings,
        warning,
        duration: start.elapsed(),
    })
}

pub(crate) fn run_queue_check(
    root: &std::path::Path,
    tsconfig: Option<&std::path::Path>,
    enabled: bool,
    facts: &CheckFactMap,
) -> Result<CheckTask<Vec<CheckFinding>>> {
    let start = Instant::now();
    let findings = if enabled {
        no_mistakes::queue::analyze_project_with_facts(root, tsconfig, &[], facts)?.check
    } else {
        Vec::new()
    };
    Ok(CheckTask {
        findings,
        warning: None,
        duration: start.elapsed(),
    })
}

pub(crate) fn run_rules_check(
    root: &std::path::Path,
    config: Option<&std::path::Path>,
    tsconfig: Option<&std::path::Path>,
    facts: &CheckFactMap,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let start = Instant::now();
    let (findings, warning) = match rules::run_check_with_facts(root, config, tsconfig, facts) {
        Ok(findings) => (findings, None),
        Err(err) => (
            Vec::new(),
            Some(format!("warning: rules check skipped: {err:#}")),
        ),
    };
    Ok(CheckTask {
        findings,
        warning,
        duration: start.elapsed(),
    })
}

pub(crate) fn run_integration_check(
    root: &std::path::Path,
    config: Option<&std::path::Path>,
    facts: &CheckFactMap,
) -> Result<CheckTask<Vec<IntegrationFinding>>> {
    let start = Instant::now();
    let findings = integration_tests::check_with_facts(root, config, facts)?;
    Ok(CheckTask {
        findings,
        warning: None,
        duration: start.elapsed(),
    })
}

pub(crate) fn run_codebase_check(
    root: &std::path::Path,
    config: Option<&std::path::Path>,
    tsconfig: Option<&std::path::Path>,
    enabled: bool,
    facts: &CheckFactMap,
) -> Result<CheckTask<Vec<UniqueExportFinding>>> {
    let start = Instant::now();
    let findings = if enabled {
        unique_exports::analyze_project_with_facts(root, config, tsconfig, facts)?
    } else {
        Vec::new()
    };
    Ok(CheckTask {
        findings,
        warning: None,
        duration: start.elapsed(),
    })
}

pub(crate) fn queues_configured(config: &NoMistakesConfig) -> bool {
    config
        .projects
        .values()
        .any(|project| !project.queues.enqueues.is_empty() || !project.queues.workers.is_empty())
}

pub(crate) fn run_filesystem_rules_check(
    root: &std::path::Path,
    config: Option<&std::path::Path>,
    enabled: bool,
    files: &[PathBuf],
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let start = Instant::now();
    let findings = if enabled {
        rules::run_filesystem_rules_with_files(root, config, files)?
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

pub(crate) fn forbidden_dependencies_configured(config: &NoMistakesConfig) -> bool {
    rule_configured(config, rules::FORBIDDEN_DEPENDENCIES)
}

pub(crate) fn unique_exports_configured(config: &NoMistakesConfig) -> bool {
    rule_configured(config, unique_exports::RULE_ID)
}

pub(crate) fn rule_configured(config: &NoMistakesConfig, rule_id: &str) -> bool {
    config.rule_configured(rule_id)
}
