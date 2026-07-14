pub(super) struct AnalyzeProjectContext {
    scopes: HashMap<EffectiveScopeKey, PreparedScope>,
    scope_aliases: HashMap<EffectiveScopeKey, EffectiveScopeKey>,
}

impl AnalyzeProjectContext {
    pub(super) fn prepare(options: &AnalyzeProjectOptions) -> Result<Self> {
        if options.reports.is_empty() {
            return Ok(Self {
                scopes: HashMap::new(),
                scope_aliases: HashMap::new(),
            });
        }
        let master_root = super::options::resolve_root(options.root.as_deref())?;
        let master_snapshot = std::sync::Arc::new(
            crate::codebase::ts_source::VisiblePathSnapshot::new(&master_root),
        );
        let mut visible_by_root = HashMap::new();
        let mut scope_aliases = HashMap::new();
        let mut grouped = std::collections::BTreeMap::<
            EffectiveScopeKey,
            (EffectiveScope, Vec<AnalyzeReportRequest>),
        >::new();
        for request in &options.reports {
            let raw = effective_scope(request, options)?;
            let visible_paths = visible_by_root
                .entry(raw.root.clone())
                .or_insert_with(|| {
                    if raw.root.starts_with(&master_root) {
                        master_snapshot.clone()
                    } else {
                        std::sync::Arc::new(crate::codebase::ts_source::VisiblePathSnapshot::new(
                            &raw.root,
                        ))
                    }
                })
                .clone();
            let raw_key = raw.key.clone();
            let effective = raw.normalize_automatic_paths(&visible_paths)?;
            scope_aliases.insert(raw_key, effective.key.clone());
            grouped
                .entry(effective.key.clone())
                .or_insert_with(|| (effective, Vec::new()))
                .1
                .push(request.clone());
        }
        let mut scopes = HashMap::new();
        for (key, (effective, reports)) in grouped {
            let visible_paths = visible_by_root
                .get(&effective.root)
                .cloned()
                .expect("effective scope snapshot is prepared");
            let scoped_options = AnalyzeProjectOptions {
                root: Some(effective.root.display().to_string()),
                tsconfig: effective
                    .tsconfig
                    .as_ref()
                    .map(|path| path.display().to_string()),
                config: effective
                    .config
                    .as_ref()
                    .map(|path| path.display().to_string()),
                filters: options.filters.clone(),
                reports,
            };
            scopes.insert(key, PreparedScope::prepare(&scoped_options, visible_paths)?);
        }
        Ok(Self {
            scopes,
            scope_aliases,
        })
    }

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
