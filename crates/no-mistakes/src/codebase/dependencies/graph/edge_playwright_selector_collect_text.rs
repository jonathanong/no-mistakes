struct SelectorTextEdgeInputs<'a> {
    root: &'a Path,
    settings: &'a crate::playwright::config::Settings,
    inputs: &'a PlaywrightSelectorEdgeInputs<'a>,
    routes: &'a [crate::routes::Route],
    context: &'a crate::playwright::analysis::context::TestAnalysisContext<'a>,
    pending: Vec<crate::playwright::analysis::pipeline_test_analysis::PendingTestFileAnalysis>,
    has_eligible_text_locator: bool,
    test_policy: crate::playwright::playwright_tests::TestPolicy,
}

fn finish_selector_text_edges(
    text: SelectorTextEdgeInputs<'_>,
) -> Result<crate::playwright::analysis::types::TestFileAnalysis> {
    let SelectorTextEdgeInputs {
        root,
        settings,
        inputs,
        routes,
        context,
        pending,
        has_eligible_text_locator,
        test_policy,
    } = text;
    let text_setup = crate::playwright::analysis::pipeline_text_setup::build_text_resolution_setup(
        root,
        settings,
        crate::playwright::analysis::pipeline_text_setup::TextResolutionInputs {
            facts: inputs.facts,
            graph_file_universe: Some(inputs.all_files),
            route_import_candidate: inputs.route_import(),
            routes,
            snapshot: inputs.snapshot,
            has_eligible_text_locator,
            has_text_candidate: &|targets, index| {
                crate::playwright::analysis::pipeline_test_analysis::has_text_locator_candidate(
                    &pending,
                    targets,
                    index,
                    test_policy,
                )
            },
            has_route_reachability_demand: &|targets, index| {
                crate::playwright::analysis::pipeline_test_analysis::has_route_reachability_demand(
                    root,
                    &pending,
                    targets,
                    index,
                    test_policy,
                )
            },
        },
    )?;
    let text_context = text_setup.has_matching_text_candidate.then_some(
        crate::playwright::analysis::text_edges::TextEdgeContext {
            app_text_targets: text_setup.app_text_targets.as_slice(),
            app_text_index: &text_setup.app_text_index,
            route_reachable_files: &text_setup.route_reachable_files,
            test_policy,
        },
    );
    Ok(
        crate::playwright::analysis::pipeline_test_analysis::finish_test_file_analysis(
            pending,
            context,
            text_context.as_ref(),
        ),
    )
}

fn selector_wrapper_resolution(
    root: &Path,
    settings: &crate::playwright::config::Settings,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
    route_import_candidate: Option<(&DepGraph, &crate::codebase::ts_resolver::TsConfig)>,
) -> Result<Option<crate::codebase::check_facts::PlaywrightModuleResolution>> {
    if settings.selector_wrappers.is_empty() {
        return Ok(None);
    }
    let paths = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let tsconfig = match route_import_candidate {
        Some((_, tsconfig)) => tsconfig.clone(),
        None => crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
            None, root, &paths, &sources,
        )?,
    };
    let workspace = crate::codebase::workspaces::load_indexed_from_source_store(root, &sources)
        .unwrap_or_default();
    Ok(Some(
        crate::codebase::check_facts::PlaywrightModuleResolution::new(
            Arc::new(tsconfig),
            Arc::new(workspace),
            Arc::new(paths.iter().cloned().collect()),
        ),
    ))
}
