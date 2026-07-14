struct PreparedPlaywrightView {
    settings: crate::playwright::config::Settings,
    fact_plan: crate::codebase::check_facts::PlaywrightFactPlan,
}

struct PreparedScope {
    options: AnalyzeProjectOptions,
    traversal: SharedTraversalContext,
    facts: crate::codebase::check_facts::CheckFactMap,
    symbol_facts: crate::codebase::check_facts::CheckFactMap,
    server: Option<crate::server_routes::PreparedServerAnalysis>,
    check: Option<SharedCheckContext>,
    playwright: HashMap<String, PreparedPlaywrightView>,
    queue_reports: HashMap<String, crate::queue::ProjectReport>,
    queue_indexed_reports: HashMap<String, crate::queue::PreparedProjectReport>,
    queue_traversal_keys: std::collections::HashSet<String>,
    server_indexed_reports: HashMap<String, crate::server_routes::PreparedProjectReport>,
    server_traversal_keys: std::collections::HashSet<String>,
    server_reports: HashMap<String, crate::server_routes::ProjectReport>,
    playwright_analyses: HashMap<String, crate::playwright::analysis::types::Analysis>,
    react_analyses: HashMap<String, Vec<crate::react_traits::ComponentFacts>>,
}

impl PreparedScope {
    fn prepare(
        options: &AnalyzeProjectOptions,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
    ) -> Result<Self> {
        crate::ast::with_request_parse_cache(|| Self::prepare_with_cache(options, visible_paths))
    }

    fn prepare_with_cache(
        options: &AnalyzeProjectOptions,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
    ) -> Result<Self> {
        let root = super::options::resolve_root(options.root.as_deref())?;
        let build_plan = graph_build_plan(options)?;
        let mut traversal = SharedTraversalContext::prepare_with_snapshot(
            root.clone(),
            options.tsconfig.as_deref().map(Path::new),
            options.config.as_deref().map(Path::new),
            build_plan,
            visible_paths,
        )?;
        traversal.add_explicit_roots(&authoritative_report_files(options, &root)?);
        let check = options
            .reports
            .iter()
            .any(|request| request.report_type == "check")
            .then(|| {
                SharedCheckContext::prepare(
                    &root,
                    traversal.config_path(),
                    options.tsconfig.as_deref().map(Path::new),
                    traversal.visible_paths_arc(),
                    traversal.config(),
                    traversal.tsconfig(),
                )
            })
            .transpose()?;
        let mut report_plan = check_fact_plan(options, &traversal)?;
        report_plan
            .graph_context
            .set_visible_files(traversal.graph_files().visible().iter().cloned());
        let mut check_plan = if let Some(check) = &check {
            check.fact_plan()
        } else {
            report_plan.clone()
        };
        if check.is_some() {
            check_plan.include(report_plan.clone());
        }
        let playwright = prepare_playwright_views(options, &traversal, check.as_ref())?;
        let mut files = check
            .as_ref()
            .map(|check| check.fact_files().to_vec())
            .unwrap_or_else(|| traversal.graph_files().all().to_vec());
        let symbol_targets = symbol_target_files(options, &root)?;
        // Without a check report, explicit symbol targets belong to the request's primary fact
        // scope. Mixed requests collect any supplemental ignored targets separately below so
        // unrelated check domains continue to honor automatic discovery and `.gitignore`.
        if check.is_none() {
            files.extend(symbol_targets.iter().cloned());
        }
        files.sort();
        files.dedup();
        let configs = playwright
            .values()
            .flat_map(|view| view.settings.playwright_configs.iter())
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect::<std::collections::HashSet<_>>();
        if !configs.is_empty() {
            files.retain(|path| !configs.contains(path));
        }
        let mut playwright_fact_plan = check
            .as_ref()
            .and_then(SharedCheckContext::playwright_fact_plan);
        for view in playwright.values() {
            match playwright_fact_plan.as_mut() {
                Some(plan) => plan.include(view.fact_plan.clone()),
                None => playwright_fact_plan = Some(view.fact_plan.clone()),
            }
        }
        if let Some(plan) = &playwright_fact_plan {
            files.extend(
                plan.files()
                    .iter()
                    .filter(|path| !configs.contains(*path))
                    .map(|path| crate::codebase::ts_resolver::normalize_path(path))
                    .collect::<Vec<_>>(),
            );
            files.sort();
            files.dedup();
        }
        check_plan
            .graph_context
            .set_visible_files(traversal.graph_files().visible().iter().cloned());
        let graph_files = check
            .as_ref()
            .map(|check| check.graph_files().to_vec())
            .unwrap_or_default();
        let primary_paths = files
            .iter()
            .chain(graph_files.iter())
            .collect::<std::collections::HashSet<_>>();
        let mut supplemental_symbol_files = if check.is_some() {
            symbol_targets
                .into_iter()
                .filter(|path| !primary_paths.contains(path))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        supplemental_symbol_files.sort();
        supplemental_symbol_files.dedup();
        let facts =
            crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
                &root,
                files,
                graph_files,
                check_plan,
                playwright_fact_plan,
            );
        let symbol_facts = crate::codebase::check_facts::collect_check_facts(
            &root,
            supplemental_symbol_files,
            report_plan,
        );
        traversal.use_check_facts(&facts);
        traversal.extend_check_facts(&symbol_facts);
        // Playwright configs were parsed while preparing the shared report/check view. Seed
        // traversal facts from that same request-cached program so a later graph report does
        // not parse the config again merely because configs are outside Playwright fact scope.
        traversal.seed_cached_program_facts(&configs);
        let server = has_server_report(options).then(|| {
            crate::server_routes::prepare_analysis_with_shared_facts(
                &root,
                traversal.tsconfig(),
                traversal.config(),
                facts.files(),
                &facts,
            )
        });
        let (queue_traversal_keys, server_traversal_keys) = traversal_report_keys(options)?;
        Ok(Self {
            options: options.clone(),
            traversal,
            facts,
            symbol_facts,
            server,
            check,
            playwright,
            queue_reports: HashMap::new(),
            queue_indexed_reports: HashMap::new(),
            queue_traversal_keys,
            server_indexed_reports: HashMap::new(),
            server_traversal_keys,
            server_reports: HashMap::new(),
            playwright_analyses: HashMap::new(),
            react_analyses: HashMap::new(),
        })
    }
}

fn traversal_report_keys(
    options: &AnalyzeProjectOptions,
) -> Result<(std::collections::HashSet<String>, std::collections::HashSet<String>)> {
    let mut queue = std::collections::HashSet::new();
    let mut server = std::collections::HashSet::new();
    for request in &options.reports {
        if !matches!(request.report_type.as_str(),
            "queueEdges" | "queueRelated" | "serverRouteEdges" | "serverRouteRelated")
        {
            continue;
        }
        let raw = project_options(request, options)?;
        let parsed: ProjectOptions = serde_json::from_str(&raw)?;
        if matches!(request.report_type.as_str(), "queueEdges" | "queueRelated") {
            queue.insert(canonical_filter_key(&parsed.filters)?);
        } else {
            server.insert(canonical_filter_key(&server_filters(&request.report_type, &parsed))?);
        }
    }
    Ok((queue, server))
}
