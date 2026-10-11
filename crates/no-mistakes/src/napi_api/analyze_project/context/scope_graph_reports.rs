impl PreparedScope {
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
        Ok(crate::cli::json_value(
            &self.traversal.flow_report(&options)?,
        ))
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

impl PreparedScope {
    fn call_sites_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Box<RawValue>> {
        let raw = super::options::command_options(request, options)?;
        let parsed: crate::napi_api::queries::CallSitesOptions = serde_json::from_value(raw)?;
        anyhow::ensure!(!parsed.file.trim().is_empty(), "file is required");
        anyhow::ensure!(
            !parsed.export_name.trim().is_empty(),
            "exportName is required"
        );
        let root = self.traversal.root();
        let file = authoritative_path(root, PathBuf::from(parsed.file));
        let stored = self
            .facts
            .ts
            .get(&file)
            .or_else(|| self.symbol_facts.ts.get(&file))
            .with_context(|| format!("missing facts for {}", file.display()))?;
        anyhow::ensure!(
            !stored.ts.fatal_parse_error,
            "extracting symbols from {}: fatal parser failure",
            file.display()
        );
        let symbols = if crate::ast::legacy_symbols_share_standard_parse(&file) {
            stored.ts.symbols.as_ref()
        } else {
            stored.legacy_symbols.as_ref()
        }
        .with_context(|| format!("missing symbols for {}", file.display()))?;
        let (files, graph) = self
            .ordinary_calls
            .as_ref()
            .and_then(|calls| calls.projection_for(&file))
            .context("ordinary call-sites projection is missing")?;
        let report = crate::codebase::queries::call_sites::project_report(
            root,
            &file,
            &parsed.export_name,
            symbols,
            crate::codebase::queries::call_sites::PreparedCallSitesProjection {
                graph,
                facts: self.traversal.prepared_facts(),
                files: files.indexable(),
            },
        )?;
        Ok(RawValue::from_string(crate::cli::json_string(&report))?)
    }
}
