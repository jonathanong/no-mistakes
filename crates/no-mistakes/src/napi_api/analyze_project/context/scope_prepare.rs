impl PreparedScopePlan {
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
        session.insert_visible_paths(&root, visible_paths.clone());
        let (import_usages, import_usage_files) =
            prepare_import_usage_views(options, &root, &session)?;
        let build_plan = graph_build_plan(options)?;
        let framework_plan = framework_preparation_plan(options, build_plan)?;
        let include_check_plan = options
            .reports
            .iter()
            .any(|request| request.report_type == "check");
        let mut traversal =
            SharedTraversalContext::prepare_with_snapshot_session_check_and_framework_plan(
                root.clone(),
                options.tsconfig.as_deref().map(Path::new),
                options.config.as_deref().map(Path::new),
                build_plan,
                SnapshotTraversalPreparation {
                    visible_paths,
                    session: session.clone(),
                    include_check_plan,
                    framework_plan,
                },
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
            files.extend(import_usage_files.iter().cloned());
        }
        files.sort();
        files.dedup();
        let configs = playwright
            .values()
            .flat_map(|view| view.settings.playwright_configs.iter())
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect::<std::collections::HashSet<_>>();
        let symbol_configs = symbol_targets
            .iter()
            .filter(|path| configs.contains(*path))
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        if !configs.is_empty() {
            files.retain(|path| !configs.contains(path) || symbol_configs.contains(path));
        }
        files.sort();
        files.dedup();
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
            // Explicit Playwright config symbol targets are staged directly
            // into primary facts without widening the check file scope.
            .chain(symbol_configs.iter())
            .collect::<std::collections::HashSet<_>>();
        let mut supplemental_report_files = if check.is_some() {
            let mut supplemental = authoritative_report_files;
            supplemental.extend(import_usage_files);
            supplemental
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
        let (queue_traversal_keys, server_traversal_keys) = traversal_report_keys(options)?;
        Ok(Self {
            options: options.clone(),
            root,
            traversal,
            primary: ScopeFactPlan {
                files,
                graph_files,
                plan: check_plan,
                playwright: playwright_fact_plan,
                sources: std::sync::Arc::clone(&sources),
            },
            supplemental: ScopeFactPlan {
                files: supplemental_report_files,
                graph_files: Vec::new(),
                plan: supplemental_plan,
                playwright: None,
                sources,
            },
            configs,
            import_usages,
            check,
            playwright,
            queue_traversal_keys,
            server_traversal_keys,
            session,
        })
    }
}
