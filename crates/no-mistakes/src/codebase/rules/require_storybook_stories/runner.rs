use super::{
    all_react_component_keys, colocated_test_covered_components, component_disabled,
    directly_covered_components, dynamic_or_mock_boundary_files, effective_story_patterns,
    file_disabled, namespace_import_findings, reachable_story_files, selected_components,
    stale_or_blank_allow_findings, transitive_covered_components, GlobMatcher, Options,
    RuleFinding, RULE_ID,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::{path_filter::RulePathFilter, sort_findings};
use crate::codebase::ts_resolver::{normalize_path, ImportResolver, TsConfig};
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::Path;

pub(super) fn check_with_tsconfig(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &CheckFactMap,
    mut resolve: impl FnMut(&Path) -> Result<TsConfig>,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let root = normalize_path(root);
    let mut findings = Vec::new();
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
        let tsconfig = resolve(&project_root)?;
        findings.extend(check_rule(
            &root,
            &project_root,
            config,
            rule,
            shared,
            &tsconfig,
            inferred_roots,
        )?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn check_rule(
    root: &Path,
    project_root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    shared: &CheckFactMap,
    tsconfig: &TsConfig,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let visible_files = shared
        .files()
        .iter()
        .map(|path| normalize_path(path))
        .collect::<HashSet<_>>();
    let resolver = ImportResolver::new(tsconfig).with_visible(&visible_files);
    let opts: Options = rule.rule_options();
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    let rule_filter = RulePathFilter::new_with_inferred(root, config, rule, &mut inferred_roots)?;
    let include = GlobMatcher::new(&opts.include)?;
    let exclude = GlobMatcher::new(&opts.exclude)?;
    let story_patterns = effective_story_patterns(root, project_root, config, &opts);
    let stories = GlobMatcher::new(&story_patterns)?;
    let allow_files = GlobMatcher::new(opts.allow_files.keys())?;
    let test_filter = crate::codebase::test_filter::TestFileFilter::new(root, config);

    let components = selected_components(
        root,
        project_root,
        shared,
        &opts,
        &include,
        &exclude,
        &test_filter,
    )
    .into_iter()
    .filter(|component| rule_filter.is_match(&component.file))
    .collect::<Vec<_>>();
    let component_keys: HashSet<String> = components.iter().map(|c| c.key.clone()).collect();
    let all_component_keys = all_react_component_keys(project_root, shared);
    let story_files = reachable_story_files(
        project_root,
        shared,
        &stories,
        &resolver,
        &all_component_keys,
    );
    let mut namespace_findings =
        namespace_import_findings(root, project_root, shared, &story_files, &resolver);
    let direct = directly_covered_components(
        project_root,
        shared,
        &story_files,
        &resolver,
        &all_component_keys,
    );
    let covered =
        transitive_covered_components(root, project_root, shared, &direct, &component_keys);
    let test_covered = if opts.allow_colocated_tests {
        colocated_test_covered_components(shared, &components)
    } else {
        Default::default()
    };
    let boundary_files = dynamic_or_mock_boundary_files(project_root, shared, &resolver);

    let mut findings = Vec::new();
    findings.append(&mut namespace_findings);
    findings.extend(stale_or_blank_allow_findings(
        root,
        project_root,
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
                "React component `{}` is selected for Storybook coverage but no reachable story imports it or a parent component that renders it. Add a Storybook story, add an accepted colocated test when `allow_colocated_tests` is enabled, render it through a covered parent component, exclude it from `{RULE_ID}`, or add a documented no-mistakes disable comment.",
                component.export_name
            ),
            import: None,
            target: Some(component.key),
        });
    }

    findings.retain(|finding| rule_filter.is_match(&root.join(&finding.file)));
    Ok(findings)
}
