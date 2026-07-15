impl AnalyzeProjectContext {
    fn scope_mut(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<&mut PreparedScope> {
        let raw_key = effective_scope(request, options)?.key;
        let key = self.scope_aliases.get(&raw_key).unwrap_or(&raw_key).clone();
        self.scopes
            .get_mut(&key)
            .with_context(|| format!("prepared analyzeProject scope is missing for `{key:?}`"))
    }

    pub(super) fn graph_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
        direction: Direction,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.graph_report(request, &scoped_options, direction)
    }

    pub(super) fn import_usages_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.import_usages_report(request, &scoped_options)
    }

    pub(super) fn symbols_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.symbols_report(request, &scoped_options)
    }

    pub(super) fn flow_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.flow_report(request, &scoped_options)
    }

    pub(super) fn effects_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.effects_report(request, &scoped_options)
    }

    pub(super) fn rsc_callers_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.rsc_callers_report(request, &scoped_options)
    }

    pub(super) fn project_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.project_report(request, &scoped_options)
    }

    pub(super) fn playwright_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let scope = self.scope_mut(request, options)?;
        let scoped_options = scope.options.clone();
        scope.playwright_report(request, &scoped_options)
    }
}
