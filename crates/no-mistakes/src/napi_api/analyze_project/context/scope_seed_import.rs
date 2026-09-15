impl PreparedScope {
    /// Reports that cannot use the lazy import graph still share one canonical
    /// `DepGraph`. Build it on the preparing thread so `reports.par_iter()`
    /// only projects. Nested rayon inside that parallel loop deadlocks when a
    /// worker waits on the graph `OnceLock` that another worker is building.
    fn seed_canonical_graph_if_needed(&self) -> Result<()> {
        if !self.needs_canonical_graph()? {
            return Ok(());
        }
        self.traversal.graph_shared()?;
        Ok(())
    }

    fn needs_canonical_graph(&self) -> Result<bool> {
        for request in &self.options.reports {
            match super::graph_direction(&request.report_type) {
                Some(Direction::Dependents) => return Ok(true),
                Some(Direction::Deps) => {
                    let args = super::traverse_args(request, &self.options)?;
                    if args.include_symbols
                        || !crate::codebase::dependencies::relationships_are_import_only(
                            &args.relationships,
                        )
                    {
                        return Ok(true);
                    }
                }
                None => {
                    if matches!(
                        request.report_type.as_str(),
                        "flow" | "effects" | "rscCallers" | "check"
                    ) {
                        return Ok(true);
                    }
                    if request.report_type == "symbols"
                        && request
                            .options
                            .get("mode")
                            .and_then(serde_json::Value::as_str)
                            == Some("signature-impact")
                    {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
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
