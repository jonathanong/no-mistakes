use crate::check_parallel::{run_domain_checks, DomainResults};
use crate::check_tasks::{
    filesystem_rules_configured, queues_configured, unique_exports_configured, CheckTask,
};
use anyhow::Result;
use no_mistakes_core::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use no_mistakes_core::codebase::rules::RuleFinding;
use no_mistakes_core::codebase::unique_exports::UniqueExportFinding;
use no_mistakes_core::config::v2::load_v2_config;
use no_mistakes_core::integration_tests::IntegrationFinding;
use no_mistakes_core::queue::CheckFinding;
use no_mistakes_core::react_traits;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub(crate) struct CheckResults {
    pub(crate) react: Vec<react_traits::Violation>,
    pub(crate) queues: Vec<CheckFinding>,
    pub(crate) rules: Vec<RuleFinding>,
    pub(crate) integration: Vec<IntegrationFinding>,
    pub(crate) codebase: Vec<UniqueExportFinding>,
    pub(crate) warnings: Vec<String>,
    pub(crate) timings: Vec<(&'static str, Duration)>,
}

struct CompletedDomainChecks {
    react: CheckTask<Vec<react_traits::Violation>>,
    queues: CheckTask<Vec<CheckFinding>>,
    rules: CheckTask<Vec<RuleFinding>>,
    integration: CheckTask<Vec<IntegrationFinding>>,
    codebase: CheckTask<Vec<UniqueExportFinding>>,
    filesystem_rules: CheckTask<Vec<RuleFinding>>,
}

impl CheckResults {
    pub(crate) fn has_findings(&self) -> bool {
        !self.react.is_empty()
            || !self.queues.is_empty()
            || !self.rules.is_empty()
            || !self.integration.is_empty()
            || !self.codebase.is_empty()
    }
}

pub(crate) fn run_all(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
) -> Result<CheckResults> {
    let root = root.canonicalize().unwrap_or(root);
    let config = load_v2_config(&root, config_path.as_deref())?;
    let queues_enabled = queues_configured(&config);
    let unique_exports_enabled = unique_exports_configured(&config);
    let rules_enabled = test_dynamic_imports_configured(&config);
    let filesystem_rules_enabled = filesystem_rules_configured(&config);
    let integration_enabled = integration_configured(&config);
    let react_enabled = react_traits::check_enabled(&root, config_path.as_deref(), false)?;
    let react_warning = None;
    let plan = fact_plan(
        react_enabled,
        queues_enabled,
        rules_enabled,
        integration_enabled,
        unique_exports_enabled,
    );
    if !plan_requests_facts(&plan) && !filesystem_rules_enabled {
        return Ok(empty_results([react_warning]));
    }
    let discover_start = Instant::now();
    let skip_directories = config.filesystem.skip_directories.clone();
    let discovered = crate::check_discovery::discover_check_files(
        &root,
        &config,
        &skip_directories,
        unique_exports_enabled,
    );
    let discover_duration = discover_start.elapsed();
    let facts_start = Instant::now();
    // When only filesystem rules are enabled, no TS/JS parsing is needed.
    let (fs_files, facts) = if plan_requests_facts(&plan) {
        let fs = if filesystem_rules_enabled {
            discovered.clone()
        } else {
            Vec::new()
        };
        let f = collect_check_facts(&root, discovered, plan);
        (fs, f)
    } else {
        (discovered, Default::default())
    };
    let facts_duration = facts_start.elapsed();

    let (react, queues, rules, integration, codebase, filesystem_rules) = run_domain_checks(
        &root,
        &config_path,
        &tsconfig_path,
        react_enabled,
        queues_enabled,
        unique_exports_enabled,
        filesystem_rules_enabled,
        fs_files,
        &facts,
    );

    let completed = complete_domain_checks((
        react,
        queues,
        rules,
        integration,
        codebase,
        filesystem_rules,
    ))?;
    let react = completed.react;
    let queues = completed.queues;
    let mut rules = completed.rules;
    let integration = completed.integration;
    let codebase = completed.codebase;
    let filesystem_rules = completed.filesystem_rules;
    let warnings = [
        react_warning,
        react.warning.clone(),
        queues.warning.clone(),
        rules.warning.clone(),
        integration.warning.clone(),
        codebase.warning.clone(),
        filesystem_rules.warning.clone(),
    ]
    .into_iter()
    .flatten()
    .collect();

    rules.findings.extend(filesystem_rules.findings);

    Ok(CheckResults {
        timings: vec![
            ("discover", discover_duration),
            ("parse_extract", facts_duration),
            ("react", react.duration),
            ("queues", queues.duration),
            ("rules", rules.duration),
            ("integration", integration.duration),
            ("codebase", codebase.duration),
            ("filesystem_rules", filesystem_rules.duration),
        ],
        react: react.findings,
        queues: queues.findings,
        rules: rules.findings,
        integration: integration.findings,
        codebase: codebase.findings,
        warnings,
    })
}

fn complete_domain_checks(results: DomainResults) -> Result<CompletedDomainChecks> {
    let (react, queues, rules, integration, codebase, filesystem_rules) = results;
    Ok(CompletedDomainChecks {
        react: react?,
        queues: queues?,
        rules: rules?,
        integration: integration?,
        codebase: codebase?,
        filesystem_rules: filesystem_rules?,
    })
}

fn fact_plan(
    react: bool,
    queue: bool,
    rules: bool,
    integration: bool,
    unique_exports: bool,
) -> CheckFactPlan {
    CheckFactPlan {
        imports: rules,
        symbols: unique_exports,
        react,
        queue,
        integration,
        dynamic_imports: rules,
        source: rules || unique_exports,
    }
}

fn plan_requests_facts(plan: &CheckFactPlan) -> bool {
    plan.imports
        || plan.symbols
        || plan.react
        || plan.queue
        || plan.integration
        || plan.dynamic_imports
        || plan.source
}

fn empty_results(warnings: [Option<String>; 1]) -> CheckResults {
    let warnings = warnings.into_iter().flatten().collect();
    CheckResults {
        react: Vec::new(),
        queues: Vec::new(),
        rules: Vec::new(),
        integration: Vec::new(),
        codebase: Vec::new(),
        warnings,
        timings: vec![
            ("discover", Duration::ZERO),
            ("parse_extract", Duration::ZERO),
            ("react", Duration::ZERO),
            ("queues", Duration::ZERO),
            ("rules", Duration::ZERO),
            ("integration", Duration::ZERO),
            ("codebase", Duration::ZERO),
            ("filesystem_rules", Duration::ZERO),
        ],
    }
}

fn test_dynamic_imports_configured(
    config: &no_mistakes_core::config::v2::NoMistakesConfig,
) -> bool {
    crate::check_tasks::rule_configured(
        config,
        no_mistakes_core::codebase::rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
    )
}

fn integration_configured(config: &no_mistakes_core::config::v2::NoMistakesConfig) -> bool {
    let vitest_configured = !config.tests.vitest.suites.is_empty();
    let playwright_configured = !config.tests.playwright.suites.is_empty();
    if vitest_configured {
        return true;
    }
    if playwright_configured {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use no_mistakes_core::codebase::rules::{RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_TESTS};

    #[test]
    fn run_all_keeps_filesystem_files_when_fact_collection_is_needed() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check-runner/facts-and-filesystem");
        let config = root.join(".no-mistakes.yml");

        let results = run_all(root, Some(config), None).unwrap();

        assert!(results.has_findings());
        assert!(results
            .rules
            .iter()
            .any(|finding| finding.rule == RUST_MAX_LINES_PER_FILE));
        assert_eq!(results.rules.len(), 2);
        let mut rule_ids = vec![
            results.rules[0].rule.as_str(),
            results.rules[1].rule.as_str(),
        ];
        rule_ids.sort();
        assert_eq!(
            rule_ids,
            vec![RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_TESTS]
        );
    }

    #[test]
    fn run_all_surfaces_react_enabled_config_errors() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check-runner/react-config-error");
        let config = root.join(".no-mistakes.yml");

        let err = run_all(root, Some(config), None)
            .err()
            .expect("expected react config error");

        assert!(err.to_string().contains("failed to parse"));
    }

    #[test]
    fn integration_configured_covers_vitest_and_playwright_suites() {
        let empty = no_mistakes_core::config::v2::NoMistakesConfig::default();
        assert!(!integration_configured(&empty));

        let mut vitest = no_mistakes_core::config::v2::NoMistakesConfig::default();
        vitest.tests.vitest.suites.push(Default::default());
        assert!(integration_configured(&vitest));

        let mut playwright = no_mistakes_core::config::v2::NoMistakesConfig::default();
        playwright.tests.playwright.suites.push(Default::default());
        assert!(integration_configured(&playwright));
    }

    #[test]
    fn complete_domain_checks_surfaces_each_domain_error() {
        assert_domain_error(err_react(), "react");
        assert_domain_error(err_queues(), "queues");
        assert_domain_error(err_rules(), "rules");
        assert_domain_error(err_integration(), "integration");
        assert_domain_error(err_codebase(), "codebase");
        assert_domain_error(err_filesystem_rules(), "filesystem_rules");
    }

    fn assert_domain_error(results: DomainResults, expected: &str) {
        let err = complete_domain_checks(results)
            .err()
            .expect("expected domain check error");
        assert_eq!(err.to_string(), expected);
    }

    fn empty_task<T>(findings: T) -> CheckTask<T> {
        CheckTask {
            findings,
            warning: None,
            duration: Duration::ZERO,
        }
    }

    fn ok_react() -> anyhow::Result<CheckTask<Vec<react_traits::Violation>>> {
        Ok(empty_task(Vec::new()))
    }

    fn ok_queues() -> anyhow::Result<CheckTask<Vec<CheckFinding>>> {
        Ok(empty_task(Vec::new()))
    }

    fn ok_rules() -> anyhow::Result<CheckTask<Vec<RuleFinding>>> {
        Ok(empty_task(Vec::new()))
    }

    fn ok_integration() -> anyhow::Result<CheckTask<Vec<IntegrationFinding>>> {
        Ok(empty_task(Vec::new()))
    }

    fn ok_codebase() -> anyhow::Result<CheckTask<Vec<UniqueExportFinding>>> {
        Ok(empty_task(Vec::new()))
    }

    fn err_react() -> DomainResults {
        (
            Err(anyhow!("react")),
            ok_queues(),
            ok_rules(),
            ok_integration(),
            ok_codebase(),
            ok_rules(),
        )
    }

    fn err_queues() -> DomainResults {
        (
            ok_react(),
            Err(anyhow!("queues")),
            ok_rules(),
            ok_integration(),
            ok_codebase(),
            ok_rules(),
        )
    }

    fn err_rules() -> DomainResults {
        (
            ok_react(),
            ok_queues(),
            Err(anyhow!("rules")),
            ok_integration(),
            ok_codebase(),
            ok_rules(),
        )
    }

    fn err_integration() -> DomainResults {
        (
            ok_react(),
            ok_queues(),
            ok_rules(),
            Err(anyhow!("integration")),
            ok_codebase(),
            ok_rules(),
        )
    }

    fn err_codebase() -> DomainResults {
        (
            ok_react(),
            ok_queues(),
            ok_rules(),
            ok_integration(),
            Err(anyhow!("codebase")),
            ok_rules(),
        )
    }

    fn err_filesystem_rules() -> DomainResults {
        (
            ok_react(),
            ok_queues(),
            ok_rules(),
            ok_integration(),
            ok_codebase(),
            Err(anyhow!("filesystem_rules")),
        )
    }
}
