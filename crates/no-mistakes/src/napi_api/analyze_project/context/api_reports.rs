impl AnalyzeProjectContext {
    fn scope(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<&PreparedScope> {
        let raw_key = effective_scope(request, options)?.key;
        let key = self.scope_aliases.get(&raw_key).unwrap_or(&raw_key);
        self.scopes
            .get(key)
            .with_context(|| format!("prepared analyzeProject scope is missing for `{key:?}`"))
    }

    pub(super) fn graph_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
        direction: Direction,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.graph_report(request, &scope.options, direction)
    }

    pub(super) fn import_usages_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.import_usages_report(request, &scope.options)
    }

    pub(super) fn symbols_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.symbols_report(request, &scope.options)
    }

    pub(super) fn flow_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.flow_report(request, &scope.options)
    }

    pub(super) fn effects_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.effects_report(request, &scope.options)
    }

    pub(super) fn rsc_callers_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.rsc_callers_report(request, &scope.options)
    }

    pub(super) fn project_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.project_report(request, &scope.options)
    }

    pub(super) fn playwright_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope(request, options)?;
        scope.playwright_report(request, &scope.options)
    }
}
