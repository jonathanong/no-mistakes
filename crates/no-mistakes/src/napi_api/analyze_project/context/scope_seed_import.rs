impl PreparedScope {
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
        let mut union: Option<crate::codebase::dependencies::TraverseArgs> = None;
        for request in &self.options.reports {
            match super::graph_direction(&request.report_type) {
                Some(Direction::Dependents) => continue,
                Some(Direction::Deps) => {}
                None => continue,
            }
            let args = super::traverse_args(request, &self.options)?;
            if args.include_symbols
                || !crate::codebase::dependencies::relationships_are_import_only(
                    &args.relationships,
                )
            {
                continue;
            }
            match &mut union {
                None => union = Some(args),
                Some(existing) => {
                    existing.files.extend(args.files);
                    existing.file_symbols.extend(args.file_symbols);
                    existing
                        .file_entrypoints_are_structured
                        .extend(args.file_entrypoints_are_structured);
                    for relationship in args.relationships {
                        if !existing.relationships.contains(&relationship) {
                            existing.relationships.push(relationship);
                        }
                    }
                }
            }
        }
        let Some(mut args) = union else {
            return Ok(());
        };
        if args.files.is_empty() {
            return Ok(());
        }
        args.depth = None;
        self.traversal.seed_lazy_import_graph_from_args(&args, &cwd)
    }
}
