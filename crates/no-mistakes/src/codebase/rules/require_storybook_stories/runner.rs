use super::suppression::unsuppressed_component_keys;
use super::{
    all_react_component_keys, colocated_test_covered_components, component_disabled,
    directly_covered_components, dynamic_or_mock_boundary_files, effective_story_patterns,
    file_disabled, namespace_import_findings, reachable_story_files, selected_components,
    stale_or_blank_allow_findings, transitive_covered_components, GlobMatcher, Options, RULE_ID,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics, path_filter::RulePathFilter, sort_findings,
};
use crate::codebase::ts_resolver::{normalize_path, ImportResolution};
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::Path;

struct RuleCheck<'a> {
    root: &'a Path,
    project_root: &'a Path,
    config: &'a NoMistakesConfig,
    rule: &'a RuleDef,
    shared: &'a CheckFactMap,
    resolver: &'a dyn ImportResolution,
    inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    defer_suppression: bool,
    sources: &'a crate::codebase::ts_source::SourceStore,
}

pub(super) use super::diagnostics::check_with_resolver;

pub(super) fn check_diagnostics_with_resolver(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &CheckFactMap,
    resolver: &dyn ImportResolution,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    defer_suppression: bool,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<AnalysisDiagnostics> {
    let root = normalize_path(root);
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        if rule.projects.len() != 1 || rule.applies_to_repository() {
            bail!("{RULE_ID} requires exactly one project target");
        }
        let Some(project) = config.projects.get(&rule.projects[0]) else {
            continue;
        };
        let project_root = project
            .root
            .as_deref()
            .map(|path| root.join(path))
            .unwrap_or_else(|| root.to_path_buf());
        let project_root = normalize_path(&project_root);
        diagnostics.append(check_rule(RuleCheck {
            root: &root,
            project_root: &project_root,
            config,
            rule,
            shared,
            resolver,
            inferred_roots,
            defer_suppression,
            sources,
        })?);
    }
    sort_findings(&mut diagnostics.findings);
    sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

fn check_rule(inputs: RuleCheck<'_>) -> Result<AnalysisDiagnostics> {
    let RuleCheck {
        root,
        project_root,
        config,
        rule,
        shared,
        resolver,
        inferred_roots,
        defer_suppression,
        sources,
    } = inputs;
    let opts: Options = rule.try_rule_options()?;
    let policy = config.effective_rule_analysis(rule)?;
    let low = policy.effort == crate::config::v2::Effort::Low;
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    let rule_filter = RulePathFilter::new_with_inferred(root, config, rule, &mut inferred_roots)?;
    let include = GlobMatcher::new(&opts.include)?;
    let exclude = GlobMatcher::new(&opts.exclude)?;
    let story_patterns = effective_story_patterns(root, project_root, config, &opts, sources);
    let stories = GlobMatcher::new(&story_patterns)?;
    let allow_files = GlobMatcher::new(opts.allow_files.keys())?;
    let test_filter = crate::codebase::test_filter::TestFileFilter::new(root, config);

    let (components, mut unanalyzable) = selected_components(
        root,
        project_root,
        shared,
        super::selection::SelectionOptions {
            options: &opts,
            defer_suppression,
        },
        &include,
        &exclude,
        &test_filter,
    );
    let components = components
        .into_iter()
        .filter(|component| rule_filter.is_match(&component.file))
        .collect::<Vec<_>>();
    let component_keys: HashSet<String> = components.iter().map(|c| c.key.clone()).collect();
    let suppression_filtered_component_keys =
        unsuppressed_component_keys(root, &components, shared);
    let all_component_keys = all_react_component_keys(project_root, shared);
    let story_files = reachable_story_files(
        project_root,
        shared,
        &stories,
        resolver,
        &all_component_keys,
    );
    let mut namespace_findings =
        namespace_import_findings(root, project_root, shared, &story_files, resolver);
    let direct = directly_covered_components(
        project_root,
        shared,
        &story_files,
        resolver,
        &all_component_keys,
    );
    let covered = if low {
        direct.clone()
    } else {
        transitive_covered_components(root, project_root, shared, &direct, &component_keys)
    };
    let test_covered = if opts.allow_colocated_tests {
        colocated_test_covered_components(shared, &components)
    } else {
        Default::default()
    };
    let boundary_files = if low {
        Default::default()
    } else {
        dynamic_or_mock_boundary_files(project_root, shared, resolver)
    };

    let mut findings = Vec::new();
    findings.append(&mut namespace_findings);
    findings.extend(stale_or_blank_allow_findings(
        root,
        project_root,
        &opts,
        &suppression_filtered_component_keys,
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
            || (!defer_suppression
                && (file_disabled(shared, &component.file)
                    || component_disabled(shared, &component.file, component.line)))
        {
            continue;
        }
        // With no covered story seeds, an explicit component cannot acquire
        // transitive coverage; implicit components may still be boundary-exempt.
        if low && (!direct.is_empty() || !component.explicit) {
            unanalyzable.push(super::effort::unanalyzable(component));
            continue;
        }
        findings.push(super::effort::missing(component));
    }

    findings.retain(|finding| rule_filter.is_match(&root.join(&finding.file)));
    let mut diagnostics = AnalysisDiagnostics {
        findings,
        advisories: Vec::new(),
    };
    super::diagnostics::append_unanalyzable(
        &mut diagnostics,
        unanalyzable,
        root,
        project_root,
        &rule_filter,
        &allow_files,
        policy.unanalyzable,
    );
    Ok(diagnostics)
}
