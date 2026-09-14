impl PreparedScope {
    /// Import-only `dependencies` reports share one lazy walk of the union of
    /// their entry files. Per-report projection then reuses those facts.
    fn seed_import_only_dependency_union(&mut self) -> Result<()> {
        let cwd = std::env::current_dir().context("reading current directory")?;
        let mut union: Option<crate::codebase::dependencies::TraverseArgs> = None;
        for request in &self.options.reports {
            match super::graph_direction(&request.report_type) {
                Some(Direction::Dependents) => return Ok(()),
                Some(Direction::Deps) => {}
                None => continue,
            }
            let args = super::traverse_args(request, &self.options)?;
            if args.include_symbols
                || !crate::codebase::dependencies::relationships_are_import_only(
                    &args.relationships,
                )
            {
                return Ok(());
            }
            match &mut union {
                None => union = Some(args),
                Some(existing) => {
                    existing.files.extend(args.files);
                    for relationship in args.relationships {
                        if !existing.relationships.contains(&relationship) {
                            existing.relationships.push(relationship);
                        }
                    }
                }
            }
        }
        let Some(args) = union else {
            return Ok(());
        };
        if args.files.is_empty() {
            return Ok(());
        }
        crate::codebase::dependencies::collect_and_filter_entries_shared(
            &args,
            Direction::Deps,
            &cwd,
            &mut self.traversal,
        )?;
        Ok(())
    }
}
