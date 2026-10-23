impl PreparedScope {
    /// Initialize only symbol-dependents traversals before parallel reports.
    /// Their shared symbol index uses Rayon, while the traversal cache owns
    /// the tsconfig diagnostics emitted during that index build.
    fn seed_symbol_dependent_traversals(&self) -> Result<()> {
        let cwd = std::env::current_dir().context("reading current directory")?;
        for request in &self.options.reports {
            if !matches!(
                super::graph_direction(&request.report_type),
                Some(Direction::Dependents)
            ) {
                continue;
            }
            let args = super::traverse_args(request, &self.options)?;
            if args.include_symbols
                || !crate::codebase::dependencies::traversal_needs_symbol_facts(&args)
            {
                continue;
            }
            // The exact traversal key caches entries, diagnostics and provenance.
            // Errors still surface at the report's original dispatch position.
            let _ = crate::codebase::dependencies::collect_and_filter_entries_prepared(
                &args,
                Direction::Dependents,
                &cwd,
                &self.traversal,
            );
        }
        Ok(())
    }
}
