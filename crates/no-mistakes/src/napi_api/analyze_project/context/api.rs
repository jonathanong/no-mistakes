pub(super) struct AnalyzeProjectContext {
    scopes: HashMap<EffectiveScopeKey, PreparedScope>,
    scope_aliases: HashMap<EffectiveScopeKey, EffectiveScopeKey>,
}

impl AnalyzeProjectContext {
    pub(super) fn prepare(options: &AnalyzeProjectOptions) -> Result<Self> {
        crate::ast::with_request_parse_cache(|| Self::prepare_with_cache(options))
    }

    fn prepare_with_cache(options: &AnalyzeProjectOptions) -> Result<Self> {
        if options.reports.is_empty() {
            return Ok(Self {
                scopes: HashMap::new(),
                scope_aliases: HashMap::new(),
            });
        }
        let master_root = super::options::resolve_root(options.root.as_deref())?;
        // N-API workers normally have no observer, so this remains a
        // zero-instrumentation session. In-process benchmark/test callers may
        // explicitly scope one without changing the structured response.
        let session =
            crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
        let master_snapshot = std::sync::Arc::new(
            crate::codebase::ts_source::VisiblePathSnapshot::new_observed(
                &master_root,
                session.observer().cloned(),
            ),
        );
        let mut visible_by_root = std::collections::BTreeMap::new();
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
                        std::sync::Arc::new(
                            crate::codebase::ts_source::VisiblePathSnapshot::new_observed(
                                &raw.root,
                                session.observer().cloned(),
                            ),
                        )
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
        for (root, snapshot) in &visible_by_root {
            session.insert_visible_paths(root, snapshot.clone());
        }
        let mut scope_plans = Vec::new();
        for (key, (effective, reports)) in grouped {
            let visible_paths = visible_by_root
                .get(&effective.root)
                .cloned()
                .expect("effective scope snapshot is prepared");
            let scoped_options = AnalyzeProjectOptions {
                root: Some(effective.root.display().to_string()),
                tsconfig: (!effective.automatic_tsconfig)
                    .then(|| effective
                    .tsconfig
                    .as_ref()
                    .map(|path| path.display().to_string()))
                    .flatten(),
                config: effective
                    .config
                    .as_ref()
                    .map(|path| path.display().to_string()),
                filters: options.filters.clone(),
                reports,
            };
            scope_plans.push((
                key,
                PreparedScopePlan::prepare(&scoped_options, visible_paths, session.clone())?,
            ));
        }
        let fact_requests = scope_plans
            .iter()
            .flat_map(|(_, scope)| scope.fact_requests())
            .collect();
        let mut collected = crate::codebase::check_facts::collect_check_fact_batch_with_session(
            &session,
            fact_requests,
        )
        .into_iter();
        let mut scopes = HashMap::new();
        for (key, plan) in scope_plans {
            let facts = collected.next().expect("primary scope facts are collected");
            let supplemental = collected
                .next()
                .expect("supplemental scope facts are collected");
            scopes.insert(key, plan.materialize(facts, supplemental)?);
        }
        // Every effective scope may seed facts from programs parsed while the
        // scope plans were prepared. Retain those programs until all scopes
        // have materialized, then release them before report execution.
        crate::ast::clear_request_parse_cache();
        Ok(Self {
            scopes,
            scope_aliases,
        })
    }
}
