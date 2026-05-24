use super::RuleFinding;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::ts_resolver::{normalize_path, ImportResolver};
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::Path;

mod colocated_tests;
mod config;
mod coverage;
mod coverage_graph;
mod findings;
mod selection;
mod types;

use colocated_tests::covered_components as colocated_test_covered_components;
use config::{effective_story_patterns, resolve_tsconfig};
use coverage::{all_react_component_keys, directly_covered_components, reachable_story_files};
use coverage_graph::{dynamic_or_mock_boundary_files, transitive_covered_components};
use findings::{namespace_import_findings, stale_or_blank_allow_findings};
use selection::{component_disabled, file_disabled, selected_components};
use types::{GlobMatcher, Options};

pub const RULE_ID: &str = "require-storybook-stories";

pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let files =
        crate::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    let facts = crate::codebase::check_facts::collect_check_facts(
        root,
        files,
        CheckFactPlan {
            react: true,
            symbols: true,
            storybook: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    check_with_facts(root, config, tsconfig_path, &facts)
}

pub(crate) fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let root = normalize_path(root);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        findings.extend(check_rule(&root, config, rule, shared, tsconfig_path)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn check_rule(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    shared: &CheckFactMap,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    if rule.projects.len() != 1 || rule.applies_to_repository() {
        bail!("{RULE_ID} requires exactly one project target");
    }
    let Some(project) = config.projects.get(&rule.projects[0]) else {
        return Ok(Vec::new());
    };
    let project_root = project
        .root
        .as_deref()
        .map(|path| root.join(path))
        .unwrap_or_else(|| root.to_path_buf());
    let project_root = normalize_path(&project_root);
    let tsconfig = resolve_tsconfig(&project_root, tsconfig_path)?;
    let resolver = ImportResolver::new(&tsconfig);
    let opts: Options = rule.rule_options();
    let include = GlobMatcher::new(&opts.include)?;
    let exclude = GlobMatcher::new(&opts.exclude)?;
    let story_patterns = effective_story_patterns(root, &project_root, config, &opts);
    let stories = GlobMatcher::new(&story_patterns)?;
    let allow_files = GlobMatcher::new(opts.allow_files.keys())?;
    let test_filter = crate::codebase::test_filter::TestFileFilter::new(root, config);

    let components = selected_components(
        root,
        &project_root,
        shared,
        &opts,
        &include,
        &exclude,
        &test_filter,
    );
    let component_keys: HashSet<String> = components.iter().map(|c| c.key.clone()).collect();
    let all_component_keys = all_react_component_keys(&project_root, shared);
    let story_files = reachable_story_files(
        &project_root,
        shared,
        &stories,
        &resolver,
        &all_component_keys,
    );
    let mut namespace_findings =
        namespace_import_findings(root, &project_root, shared, &story_files, &resolver);
    let direct = directly_covered_components(
        &project_root,
        shared,
        &story_files,
        &resolver,
        &all_component_keys,
    );
    let covered =
        transitive_covered_components(root, &project_root, shared, &direct, &component_keys);
    let test_covered = if opts.allow_colocated_tests {
        colocated_test_covered_components(shared, &components)
    } else {
        Default::default()
    };
    let boundary_files = dynamic_or_mock_boundary_files(&project_root, shared, &resolver);

    let mut findings = Vec::new();
    findings.append(&mut namespace_findings);
    findings.extend(stale_or_blank_allow_findings(
        root,
        &project_root,
        &opts,
        &component_keys,
        &allow_files,
        shared,
    ));

    for component in components {
        if covered.contains(&component.key) || test_covered.contains(&component.key) {
            continue;
        }
        if !component.explicit
            && boundary_files.contains(&component.file)
            && !direct.contains(&component.key)
        {
            continue;
        }
        if opts.allow_components.contains_key(&component.key)
            || allow_files.is_match(&component.project_file)
            || file_disabled(shared, &component.file)
            || component_disabled(shared, &component.file, component.line)
        {
            continue;
        }
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: component.repo_file,
            line: component.line,
            message: format!(
                "React component `{}` is selected for Storybook coverage but no reachable story imports it or a parent component that renders it. Add a Storybook story, render it through a covered parent component, exclude it from `{RULE_ID}`, or add a documented no-mistakes disable comment.",
                component.export_name
            ),
            import: None,
            target: Some(component.key),
        });
    }

    Ok(findings)
}

#[cfg(test)]
mod tests;
