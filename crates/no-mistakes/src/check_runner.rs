use crate::check_parallel::{run_domain_checks, DomainCheckInputs};
use crate::check_tasks::{
    filesystem_rules_configured, queues_configured, unique_exports_configured,
};
use anyhow::Result;
use enabled::{fact_plan, integration_configured, plan_requests_facts};
use no_mistakes::codebase::check_facts::collect_check_facts;
use no_mistakes::config::v2::load_v2_config;
use no_mistakes::react_traits;
use std::path::PathBuf;
use std::time::Instant;

mod enabled;
mod results;

pub(crate) use results::CheckResults;
use results::{complete_domain_checks, empty_results};

pub(crate) fn run_all(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
) -> Result<CheckResults> {
    let root = root.canonicalize().unwrap_or(root);
    let config = load_v2_config(&root, config_path.as_deref())?;
    let queues_enabled = queues_configured(&config);
    let unique_exports_enabled = unique_exports_configured(&config);
    let enabled = enabled::ConfiguredChecks::from_config(&config);
    let filesystem_rules_enabled = filesystem_rules_configured(&config);
    let integration_enabled = integration_configured(&config);
    let react_enabled = react_traits::check_enabled(&root, config_path.as_deref(), false)?;
    let react_warning = None;
    let plan = fact_plan(enabled::EnabledChecks {
        react: react_enabled,
        queue: queues_enabled,
        dynamic_import_rules: enabled.dynamic_import_rules,
        boundary_rules: enabled.boundary_rules,
        nextjs_api_routes: enabled.nextjs_api_routes,
        nextjs_caching: enabled.nextjs_caching,
        storybook_stories: enabled.storybook_stories,
        integration: integration_enabled,
        unique_exports: unique_exports_enabled,
    });
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

    let (react, queues, rules, integration, codebase, filesystem_rules) =
        run_domain_checks(DomainCheckInputs {
            root: &root,
            config_path: &config_path,
            tsconfig_path: &tsconfig_path,
            react_enabled,
            queues_enabled,
            unique_exports_enabled,
            filesystem_rules_enabled,
            discovered_files: fs_files,
            facts: &facts,
        });

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

#[cfg(test)]
mod tests;
