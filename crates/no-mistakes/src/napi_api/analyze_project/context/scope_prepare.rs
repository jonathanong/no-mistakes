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
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        crate::ast::with_request_parse_cache(|| {
            Self::prepare_with_cache(options, visible_paths, session)
        })
    }

    fn prepare_with_cache(
        options: &AnalyzeProjectOptions,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        let root = super::options::resolve_root(options.root.as_deref())?;
        let build_plan = graph_build_plan(options)?;
        let framework_plan = framework_preparation_plan(options, build_plan)?;
        let mut traversal = SharedTraversalContext::
            prepare_with_snapshot_session_check_and_framework_plan(
            root.clone(),
            options.tsconfig.as_deref().map(Path::new),
            options.config.as_deref().map(Path::new),
            build_plan,
            visible_paths,
            session.clone(),
            options
                .reports
                .iter()
                .any(|request| request.report_type == "check"),
            framework_plan,
        )?;
        let authoritative_report_files = authoritative_report_files(options, &root)?;
        traversal.add_explicit_roots(&authoritative_report_files);
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
        let mut supplemental_report_files = if check.is_some() {
            authoritative_report_files
                .into_iter()
                .filter(|path| !primary_paths.contains(path))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        supplemental_report_files.sort();
        supplemental_report_files.dedup();
        let sources = traversal.source_store();
        let mut supplemental_plan = report_plan;
        supplemental_plan.dynamic_imports |= check_plan.dynamic_imports;
        supplemental_plan.source |= check_plan.source;
        let facts = crate::codebase::check_facts::
            collect_check_facts_with_graph_files_playwright_sources_and_session(
                &session,
                &root,
                (files, graph_files),
                check_plan,
                playwright_fact_plan,
                std::sync::Arc::clone(&sources),
            );
        let symbol_facts = crate::codebase::check_facts::
            collect_check_facts_with_graph_files_playwright_sources_and_session(
                &session,
                &root,
                (supplemental_report_files, Vec::new()),
                supplemental_plan,
                None,
                sources,
            );
        let graph_facts = facts.graph_view_with_supplemental(&symbol_facts);
        let facts = facts.scoped_view_with_supplemental(&symbol_facts);
        traversal.use_check_facts(&graph_facts);
        // Playwright configs were parsed while preparing the shared report/check view. Seed
        // traversal facts before the canonical graph so invalidation cannot force a later
        // TS-only rebuild that loses shared Playwright occurrences.
        traversal.seed_cached_program_facts(&configs);
        if check
            .as_ref()
            .and_then(SharedCheckContext::graph_plan)
            .is_some()
        {
            traversal.prepare_canonical_graph_with_check_facts(&graph_facts)?;
        }
        crate::ast::clear_request_parse_cache();
        let server = has_server_report(options).then(|| {
            crate::server_routes::prepare_analysis_with_shared_facts_and_session(
                &root,
                traversal.tsconfig(),
                traversal.config(),
                facts.files(),
                &facts,
                session.clone(),
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
