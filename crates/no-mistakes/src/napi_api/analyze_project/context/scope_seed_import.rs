impl PreparedScope {
    /// Playwright analysis uses nested rayon work. Initialize each distinct
    /// analysis before `reports.par_iter()` so report workers only read cached
    /// results and cannot starve the initializer on a small rayon pool.
    fn seed_playwright_analyses(&self) -> Result<()> {
        for request in self
            .options
            .reports
            .iter()
        .filter(|request| super::is_playwright_report(&request.report_type))
        {
            let raw = playwright_options(request, &self.options)?;
            let options: PlaywrightOptions = serde_json::from_str(&raw)?;
            self.playwright_analysis(&options)?;
        }
        Ok(())
    }

    /// Reports that cannot use the lazy import graph still share prepared
    /// `DepGraph`s. Build those graphs on the preparing thread so
    /// `reports.par_iter()` only projects. Nested rayon inside that parallel
    /// loop deadlocks when a worker waits on the graph `OnceLock` that another
    /// worker is building.
    ///
    /// Prewarm the same plan each report will request: a symbols-enabled
    /// `build_plan` is a different cache key from `from_allowed` without
    /// symbols, and seeding the wrong one adds a second graph build.
    fn seed_canonical_graph_if_needed(&self) -> Result<()> {
        for request in &self.options.reports {
            self.prewarm_graph_for_report(request)?;
        }
        Ok(())
    }

    fn prewarm_graph_for_report(&self, request: &AnalyzeReportRequest) -> Result<()> {
        match super::graph_direction(&request.report_type) {
            Some(direction) => {
                let args = super::traverse_args(request, &self.options)?;
                let import_only = !args.include_symbols
                    && crate::codebase::dependencies::relationships_are_import_only(
                        &args.relationships,
                    );
                if import_only && matches!(direction, Direction::Deps) {
                    return Ok(());
                }
                let allowed =
                    crate::codebase::dependencies::relationship_filter(&args.relationships);
                let has_call = allowed.as_ref().is_some_and(|set| {
                    set.contains(&crate::codebase::dependencies::EdgeKind::Call)
                });
                if has_call || args.include_symbols {
                    self.traversal.graph_shared()?;
                } else if self.traversal.build_plan().symbols {
                    self.traversal
                        .request_graph_without_symbols_shared(allowed.as_ref())?;
                } else {
                    self.traversal.graph_shared()?;
                }
                Ok(())
            }
            None => {
                if matches!(
                    request.report_type.as_str(),
                    "flow" | "effects" | "rscCallers" | "check"
                ) || (request.report_type == "symbols"
                    && request
                        .options
                        .get("mode")
                        .and_then(serde_json::Value::as_str)
                        == Some("signature-impact"))
                {
                    self.traversal.graph_shared()?;
                }
                Ok(())
            }
        }
    }

    /// Import-only `dependencies` reports share one reachable import graph.
    /// Each report then projects its own closure from that graph.
    fn seed_import_only_dependency_graph(&mut self) -> Result<()> {
        let cwd = std::env::current_dir().context("reading current directory")?;
        let plan = classify_import_seed_requests(&self.options)?;
        apply_exclusive_candidate_inventory(&self.options, &mut self.traversal)?;
        if let Some(mut args) = plan.unbounded {
            if !args.files.is_empty() {
                args.depth = None;
                self.traversal
                    .seed_lazy_import_graph_from_args(&args, &cwd)?;
            }
        }
        for args in plan.bounded.into_values() {
            if args.files.is_empty() {
                continue;
            }
            self.traversal
                .seed_bounded_lazy_import_graph_from_args(&args, &cwd)?;
        }
        Ok(())
    }
}

struct ImportSeedPlan {
    unbounded: Option<crate::codebase::dependencies::TraverseArgs>,
    bounded: std::collections::HashMap<
        crate::codebase::dependencies::BoundedImportKey,
        crate::codebase::dependencies::TraverseArgs,
    >,
    other_graph_consumer: bool,
}

impl ImportSeedPlan {
    fn exclusive_bounds(&self) -> Option<&crate::codebase::dependencies::TraverseArgs> {
        (!self.other_graph_consumer && self.unbounded.is_none() && self.bounded.len() == 1)
            .then(|| self.bounded.values().next())
            .flatten()
    }
}

fn apply_exclusive_candidate_inventory(
    options: &AnalyzeProjectOptions,
    traversal: &mut crate::codebase::dependencies::SharedTraversalContext,
) -> Result<()> {
    if traversal.candidate_inventory_applied() {
        return Ok(());
    }
    if let Some(args) = classify_import_seed_requests(options)?.exclusive_bounds() {
        traversal.apply_candidate_inventory(args)?;
    }
    Ok(())
}

fn classify_import_seed_requests(options: &AnalyzeProjectOptions) -> Result<ImportSeedPlan> {
    let mut plan = ImportSeedPlan {
        unbounded: None,
        bounded: std::collections::HashMap::new(),
        other_graph_consumer: false,
    };
    for request in &options.reports {
        if request.report_type == "resolveCheckDependencies" {
            continue;
        }
        match super::graph_direction(&request.report_type) {
            Some(Direction::Dependents) => {
                let args = super::traverse_args(request, options)?;
                crate::codebase::dependencies::validate_candidate_bounds(
                    &args,
                    Direction::Dependents,
                )?;
                plan.other_graph_consumer = true;
                continue;
            }
            Some(Direction::Deps) => {}
            None => {
                plan.other_graph_consumer = true;
                continue;
            }
        }
        let args = super::traverse_args(request, options)?;
        crate::codebase::dependencies::validate_candidate_bounds(&args, Direction::Deps)?;
        if args.include_symbols
            || !crate::codebase::dependencies::relationships_are_import_only(&args.relationships)
        {
            plan.other_graph_consumer = true;
            continue;
        }
        if args.has_candidate_bounds() {
            match plan
                .bounded
                .entry(crate::codebase::dependencies::BoundedImportKey::from_args(
                    &args,
                )) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(args);
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    union_import_args(entry.get_mut(), args);
                }
            }
            continue;
        }
        match &mut plan.unbounded {
            None => plan.unbounded = Some(args),
            Some(existing) => union_import_args(existing, args),
        }
    }
    Ok(plan)
}

fn union_import_args(
    existing: &mut crate::codebase::dependencies::TraverseArgs,
    args: crate::codebase::dependencies::TraverseArgs,
) {
    existing.files.extend(args.files);
    existing.file_symbols.extend(args.file_symbols);
    existing.depth = match (existing.depth, args.depth) {
        (Some(left), Some(right)) => Some(left.max(right)),
        _ => None,
    };
    existing
        .file_entrypoints_are_structured
        .extend(args.file_entrypoints_are_structured);
    for relationship in args.relationships {
        if !existing.relationships.contains(&relationship) {
            existing.relationships.push(relationship);
        }
    }
}
