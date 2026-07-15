impl PreparedScopePlan {
    fn fact_requests(&self) -> [crate::codebase::check_facts::BatchCheckFactRequest; 2] {
        [
            self.primary.batch_request(&self.root),
            self.supplemental.batch_request(&self.root),
        ]
    }

    fn materialize(
        mut self,
        facts: crate::codebase::check_facts::CheckFactMap,
        symbol_facts: crate::codebase::check_facts::CheckFactMap,
    ) -> Result<PreparedScope> {
        let graph_facts = facts.graph_view_with_supplemental(&symbol_facts);
        self.traversal.use_check_facts(&graph_facts);
        // Config facts were prepared before the batch. Seed them before the
        // canonical graph so invalidation retains Playwright occurrences.
        self.traversal.seed_cached_program_facts(&self.configs);
        if self
            .check
            .as_ref()
            .and_then(SharedCheckContext::graph_plan)
            .is_some()
        {
            self.traversal
                .prepare_canonical_graph_with_check_facts(&graph_facts)?;
        }
        let server = has_server_report(&self.options).then(|| {
            crate::server_routes::prepare_analysis_with_shared_facts_and_session(
                &self.root,
                self.traversal.tsconfig(),
                self.traversal.config(),
                facts.files(),
                &facts,
                self.session,
            )
        });
        Ok(PreparedScope {
            options: self.options,
            traversal: self.traversal,
            facts,
            symbol_facts,
            import_usages: self.import_usages,
            server,
            check: self.check,
            playwright: self.playwright,
            queue_reports: HashMap::new(),
            queue_indexed_reports: HashMap::new(),
            queue_traversal_keys: self.queue_traversal_keys,
            server_indexed_reports: HashMap::new(),
            server_traversal_keys: self.server_traversal_keys,
            server_reports: HashMap::new(),
            playwright_analyses: HashMap::new(),
            react_analyses: HashMap::new(),
        })
    }
}

impl ScopeFactPlan {
    fn batch_request(&self, root: &Path) -> crate::codebase::check_facts::BatchCheckFactRequest {
        crate::codebase::check_facts::BatchCheckFactRequest {
            root: root.to_path_buf(),
            files: self.files.clone(),
            graph_files: self.graph_files.clone(),
            plan: self.plan.clone(),
            playwright: self.playwright.clone(),
            sources: std::sync::Arc::clone(&self.sources),
        }
    }
}
