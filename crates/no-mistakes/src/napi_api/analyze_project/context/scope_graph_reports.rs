impl PreparedScope {
    fn resolve_check_dependencies_report(&self, request: &AnalyzeReportRequest) -> Result<Value> {
        let ids = request
            .options
            .get("dependencyReportIds")
            .context("dependencyReportIds is required")?
            .as_array()
            .context("dependencyReportIds must be an array of dependency report IDs")?;
        anyhow::ensure!(!ids.is_empty(), "dependencyReportIds must contain at least one ID");
        let mut files = std::collections::BTreeSet::new();
        let mut seen_ids = std::collections::HashSet::new();
        let cwd = std::env::current_dir().context("reading current directory")?;
        for id in ids {
            let id = id
                .as_str()
                .filter(|id| !id.is_empty())
                .context("dependencyReportIds must contain non-empty strings")?;
            anyhow::ensure!(seen_ids.insert(id), "dependencyReportIds contains duplicate ID `{id}`");
            let dependencies = self
                .options
                .reports
                .iter()
                .filter(|candidate| {
                    candidate.report_type == "dependencies" && candidate.id.as_deref() == Some(id)
                })
                .collect::<Vec<_>>();
            anyhow::ensure!(dependencies.len() <= 1, "dependency report ID `{id}` is ambiguous");
            let dependency = dependencies.into_iter().next().with_context(|| {
                    format!("dependencyReportIds references no dependencies report with id `{id}` in this analysis scope")
                })?;
            let args = super::traverse_args(dependency, &self.options)?;
            anyhow::ensure!(
                !args.include_symbols
                    && args.relationships.iter().all(|relationship| matches!(
                        relationship,
                        crate::codebase::dependencies::RelationshipArg::Import
                            | crate::codebase::dependencies::RelationshipArg::ImportStatic
                            | crate::codebase::dependencies::RelationshipArg::ImportDynamic
                            | crate::codebase::dependencies::RelationshipArg::ImportType
                            | crate::codebase::dependencies::RelationshipArg::ImportRequire
                            | crate::codebase::dependencies::RelationshipArg::Workspace
                    )),
                "dependency report `{id}` must use import or workspace relationships without includeSymbols"
            );
            let result = crate::codebase::dependencies::collect_and_filter_entries_prepared(
                &args,
                Direction::Deps,
                &cwd,
                &self.traversal,
            )?;
            files.extend(crate::codebase::dependencies::explicit_existing_entry_files(
                &args,
                self.traversal.root(),
                &cwd,
            ));
            files.extend(result.file_paths().map(ToOwned::to_owned));
        }
        let session = self.traversal.session_arc();
        let visible_paths = self.traversal.visible_paths().paths_for(self.traversal.root());
        let visible: crate::fx::PathSet = visible_paths.iter().cloned().collect();
        let source_store = self.traversal.source_store();
        let report = crate::codebase::queries::resolve_check::batch_report_from_prepared_facts(
            self.traversal.root(),
            files,
            self.traversal.prepared_facts(),
            &visible,
            &source_store,
            self.options.tsconfig.as_deref().map(|_| self.traversal.tsconfig()),
            &session,
        )?;
        Ok(crate::cli::json_value(&report))
    }

    pub(super) fn graph_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
        direction: Direction,
    ) -> Result<Box<RawValue>> {
        let args = super::traverse_args(request, options)?;
        crate::codebase::dependencies::validate_candidate_bounds(&args, direction)?;
        let cwd = std::env::current_dir().context("reading current directory")?;
        let result = crate::codebase::dependencies::collect_and_filter_entries_prepared(
            &args,
            direction,
            &cwd,
            &self.traversal,
        );
        let result = result?;
        let bytes = crate::codebase::dependencies::result_json_bytes(&args, &result)?;
        json_raw_bytes(bytes)
    }

    pub(super) fn import_usages_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let value = super::import_usages_options(request, options)?;
        let root = self.traversal.root().to_path_buf();
        let prepared = self
            .import_usages
            .get(&value)
            .context("prepared importUsages file universe is missing")?;
        let report = crate::codebase::import_usages::collect_with_facts(
            &root,
            prepared,
            self.traversal.prepared_facts(),
        );
        let report = report?;
        Ok(crate::cli::json_value(&report))
    }

    pub(super) fn symbols_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let raw = super::symbols_options(request, options)?;
        let parsed: crate::napi_api::options::SymbolOptions = serde_json::from_str(&raw)?;
        let args = crate::napi_api::codebase::build_symbols_args(parsed)?;
        if args.mode == crate::codebase::symbols::SymbolsMode::SignatureImpact {
            let output = self.traversal.signature_impact_json(&args)?;
            return Ok(serde_json::from_str(&output)?);
        }
        let session = self.traversal.session_arc();
        let collected = crate::codebase::symbols::collect_entries_with_prepared_facts(
            &args,
            self.traversal.root(),
            self.traversal.tsconfig_catalog(),
            self.traversal.graph_files(),
            &self.facts,
            &self.symbol_facts,
            &session,
        );
        let (entries, roots) = collected?;
        let mut output = Vec::new();
        crate::codebase::symbols::output::write_json(&roots, &entries, &mut output)?;
        Ok(serde_json::from_slice(&output)?)
    }

    pub(super) fn flow_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let raw = super::flow_options(request, options)?;
        let parsed: crate::napi_api::options::FlowOptions = serde_json::from_str(&raw)?;
        let options = crate::napi_api::project::build_flow_options(parsed)?;
        Ok(crate::cli::json_value(&self.traversal.flow_report(&options)?))
    }

    pub(super) fn effects_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let parsed = super::options::effects_options(request, options)?;
        let kind = parsed
            .kind
            .as_deref()
            .context("kind is required for effects")?;
        let entry = parsed
            .entry
            .as_deref()
            .context("entry is required for effects")?;
        let selection = crate::effects_query::selection_from_config(
            self.traversal.config(),
            kind,
            &parsed.categories,
        );
        let selection = selection?;
        let report = self
            .traversal
            .effects_report(&selection, Path::new(entry), parsed.depth)?;
        Ok(crate::cli::json_value(&report))
    }

    pub(super) fn rsc_callers_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let parsed = super::options::rsc_callers_options(request, options)?;
        let component = parsed
            .component
            .as_deref()
            .context("component is required for rsc-callers")?;
        let report = self
            .traversal
            .rsc_callers_report(Path::new(component), parsed.depth)?;
        Ok(crate::cli::json_value(&report))
    }
}
