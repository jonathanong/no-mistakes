use crate::check_parallel::DomainResults;
use crate::check_tasks::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::UniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::time::Duration;

pub(crate) struct FinalizeInput<'a> {
    pub(crate) root: &'a std::path::Path,
    pub(crate) config: &'a no_mistakes::config::v2::NoMistakesConfig,
    pub(crate) filesystem_files: &'a [std::path::PathBuf],
    pub(crate) filesystem_rules_enabled: bool,
    pub(crate) react_warning: Option<String>,
    pub(crate) discover_duration: Duration,
    pub(crate) facts_duration: Duration,
    pub(crate) completed: CompletedDomainChecks,
}

pub(crate) struct CheckResults {
    pub(crate) react: Vec<react_traits::Violation>,
    pub(crate) queues: Vec<CheckFinding>,
    pub(crate) rules: Vec<RuleFinding>,
    pub(crate) integration: Vec<IntegrationFinding>,
    pub(crate) codebase: Vec<UniqueExportFinding>,
    pub(crate) warnings: Vec<String>,
    pub(crate) advisories: Vec<RuleFinding>,
    pub(crate) timings: Vec<(&'static str, Duration)>,
}

pub(crate) struct CompletedDomainChecks {
    pub(crate) react: CheckTask<Vec<react_traits::Violation>>,
    pub(crate) queues: CheckTask<Vec<CheckFinding>>,
    pub(crate) rules: CheckTask<Vec<RuleFinding>>,
    pub(crate) integration: CheckTask<Vec<IntegrationFinding>>,
    pub(crate) codebase: CheckTask<Vec<UniqueExportFinding>>,
    pub(crate) filesystem_rules: CheckTask<Vec<RuleFinding>>,
}

pub(crate) fn complete_domain_checks(results: DomainResults) -> Result<CompletedDomainChecks> {
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

pub(crate) fn finalize_domain_checks(input: FinalizeInput<'_>) -> Result<CheckResults> {
    let FinalizeInput {
        root,
        config,
        filesystem_files,
        filesystem_rules_enabled,
        react_warning,
        discover_duration,
        facts_duration,
        completed,
    } = input;
    let react = completed.react;
    let queues = completed.queues;
    let mut rules = completed.rules;
    let integration = completed.integration;
    let codebase = completed.codebase;
    let filesystem_rules = completed.filesystem_rules;
    let warnings = [
        react_warning,
        react.warning,
        queues.warning,
        rules.warning,
        integration.warning,
        codebase.warning,
        filesystem_rules.warning,
    ]
    .into_iter()
    .flatten()
    .collect();
    rules.findings.extend(filesystem_rules.findings);
    let advisories = if filesystem_rules_enabled {
        no_mistakes::codebase::rules::agents_md_max_size::advisories_with_files(
            root,
            config,
            filesystem_files,
        )?
    } else {
        Vec::new()
    };
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
        advisories,
    })
}

pub(crate) fn empty_results(warnings: [Option<String>; 1]) -> CheckResults {
    let warnings = warnings.into_iter().flatten().collect();
    CheckResults {
        react: Vec::new(),
        queues: Vec::new(),
        rules: Vec::new(),
        integration: Vec::new(),
        codebase: Vec::new(),
        warnings,
        advisories: Vec::new(),
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

pub(crate) fn json_value(results: &CheckResults) -> serde_json::Value {
    let CheckResults {
        react,
        queues,
        rules,
        integration,
        codebase,
        warnings,
        advisories,
        timings,
    } = results;
    let _ = timings;
    serde_json::json!({
        "react": react,
        "queues": queues,
        "rules": rules,
        "integration": integration,
        "codebase": codebase,
        "warnings": warnings,
        "advisories": advisories,
    })
}
