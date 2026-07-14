impl PreparedScope {
    pub(super) fn graph_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
        direction: Direction,
    ) -> Result<Value> {
        let args = super::traverse_args(request, options)?;
        let cwd = std::env::current_dir().context("reading current directory")?;
        let result = crate::codebase::dependencies::collect_and_filter_entries_shared(
            &args,
            direction,
            &cwd,
            &mut self.traversal,
        )?;
        let json = crate::codebase::dependencies::result_json(&args, &result)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub(super) fn import_usages_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let value = super::import_usages_options(request, options)?;
        let options = serde_json::from_str(value.as_str())?;
        let args = crate::napi_api::codebase::build_import_usages_args(options);
        let cwd = std::env::current_dir().context("reading current directory")?;
        let root = self.traversal.root().to_path_buf();
        let report = crate::codebase::import_usages::collect_with_facts(
            &args,
            &root,
            &cwd,
            self.traversal.facts(),
        )?;
        Ok(serde_json::to_value(report)?)
    }

    pub(super) fn symbols_report(
        &mut self,
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
        let (entries, roots) =
            crate::codebase::symbols::collect_entries_with_prepared_facts(
            &args,
            self.traversal.root(),
            self.traversal.tsconfig(),
            self.traversal.graph_files().visible(),
            &self.facts,
            &self.symbol_facts,
            &session,
        )?;
        let mut output = Vec::new();
        crate::codebase::symbols::output::write_json(&roots, &entries, &mut output)?;
        Ok(serde_json::from_slice(&output)?)
    }

    pub(super) fn flow_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let raw = super::flow_options(request, options)?;
        let parsed: crate::napi_api::options::FlowOptions = serde_json::from_str(&raw)?;
        let options = crate::napi_api::project::build_flow_options(parsed)?;
        Ok(serde_json::to_value(self.traversal.flow_report(&options)?)?)
    }

    pub(super) fn effects_report(
        &mut self,
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
        )?;
        Ok(serde_json::to_value(self.traversal.effects_report(
            &selection,
            Path::new(entry),
            parsed.depth,
        )?)?)
    }

    pub(super) fn rsc_callers_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let parsed = super::options::rsc_callers_options(request, options)?;
        let component = parsed
            .component
            .as_deref()
            .context("component is required for rsc-callers")?;
        Ok(serde_json::to_value(
            self.traversal
                .rsc_callers_report(Path::new(component), parsed.depth)?,
        )?)
    }
}
