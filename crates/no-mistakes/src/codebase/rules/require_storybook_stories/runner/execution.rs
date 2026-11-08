use super::*;

pub(super) fn check_rule(inputs: RuleCheck<'_>) -> Result<AnalysisDiagnostics> {
    let RuleCheck {
        session,
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
    let selected_sources = super::super::source_failures::SourceFailures::new(shared);
    let catalog_sources = super::super::source_failures::SourceFailures::new(shared);
    let test_filter = crate::codebase::test_filter::TestFileFilter::new(root, config);
    let (components, mut unanalyzable) = selected_components(
        root,
        project_root,
        shared,
        super::super::selection::SelectionOptions {
            options: &opts,
            defer_suppression,
            demand: Some((&selected_sources, &rule_filter, &allow_files)),
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
        Some(&catalog_sources),
    );
    let (mut incomplete, mut story_errors) =
        super::super::diagnostics::malformed_stories(root, shared, &story_files, defer_suppression);
    let visible_error = super::super::diagnostics::visible_story_error(
        root,
        project_root,
        shared,
        &story_errors,
        &rule_filter,
        &allow_files,
    );
    unanalyzable.append(&mut story_errors);
    let mut namespace_findings =
        namespace_import_findings(root, project_root, shared, &story_files, resolver);
    let direct = directly_covered_components(
        project_root,
        shared,
        &story_files,
        resolver,
        &all_component_keys,
        Some(&catalog_sources),
    );
    let covered = if low {
        direct.clone()
    } else {
        transitive_covered_components(
            root,
            project_root,
            shared,
            &direct,
            &component_keys,
            Some(&catalog_sources),
        )
    };
    let test_covered = if opts.allow_colocated_tests {
        colocated_test_covered_components(shared, &components)
    } else {
        Default::default()
    };
    let needs_boundary = !catalog_sources.failed()
        && components.iter().any(|component| {
            !component.explicit
                && !covered.contains(&component.key)
                && !test_covered.contains(&component.key)
                && !opts.allow_components.contains_key(&component.key)
                && !allow_files.is_match(&component.project_file)
        });
    let boundary_files = if low {
        Default::default()
    } else {
        dynamic_or_mock_boundary_files(
            project_root,
            shared,
            resolver,
            needs_boundary.then_some(&catalog_sources),
        )
    };
    incomplete |= catalog_sources.failed();
    let mut findings = Vec::new();
    findings.append(&mut namespace_findings);
    findings.extend(super::allow::missing_findings(
        root,
        project_root,
        config,
        rule,
        sources,
        &opts,
        (&include, &exclude),
    ));
    let scope = super::super::findings::allow_scope(root, sources, &components);
    findings.extend(stale_or_blank_allow_findings(
        root,
        project_root,
        &opts,
        &suppression_filtered_component_keys,
        &allow_files,
        shared,
        Some((&scope, &selected_sources, &catalog_sources)),
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
        if incomplete {
            unanalyzable.extend(
                (!visible_error)
                    .then(|| super::super::diagnostics::incomplete_component(component)),
            );
            continue;
        }
        // Without story seeds, only implicit components need boundary analysis.
        if low && (!direct.is_empty() || !component.explicit) {
            unanalyzable.push(super::super::effort::unanalyzable(component));
            continue;
        }
        findings.push(super::super::effort::missing(component));
    }
    findings.retain(|finding| {
        finding.rule == crate::codebase::rules::allow_audit::RULE_ID
            || rule_filter.is_match(&root.join(&finding.file))
    });
    let mut diagnostics = AnalysisDiagnostics {
        findings,
        ..Default::default()
    };
    super::super::diagnostics::append_unanalyzable(
        &mut diagnostics,
        unanalyzable,
        root,
        project_root,
        &rule_filter,
        &allow_files,
        policy.unanalyzable,
    );
    selected_sources.finish(session)?;
    catalog_sources.finish(session)?;
    Ok(diagnostics)
}
