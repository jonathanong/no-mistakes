impl PreparedScope {
    fn queue_report(&self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        let key = canonical_filter_key(&options.filters)?;
        let traversal = matches!(report_type, "queueEdges" | "queueRelated")
            || self.queue_traversal_keys.contains(&key);
        let root = self.traversal.root();
        let tsconfig_catalog = self.traversal.tsconfig_catalog();
        let session = self.traversal.session_arc();
        let facts = &self.facts;
        let report = cached_analysis(
            &self.queue_reports,
            &self.queue_indexed_reports,
            &key,
            traversal,
            || {
                crate::queue::analyze_project_with_prepared_facts_and_catalog_and_session(
                    root,
                    tsconfig_catalog,
                    &options.filters,
                    facts,
                    &session,
                )
            },
            || {
                crate::queue::analyze_project_with_prepared_facts_indexed_and_catalog_and_session(
                    root,
                    tsconfig_catalog,
                    &options.filters,
                    facts,
                    &session,
                )
            },
        );
        let report = report?;
        match report {
            CachedAnalysis::Plain(report) => {
                render_queue_report(report_type, options, &report, None)
            }
            CachedAnalysis::Indexed(indexed) => {
                let traversal_report = matches!(report_type, "queueEdges" | "queueRelated");
                render_queue_report(
                    report_type,
                    options,
                    indexed.report(),
                    traversal_report.then_some(&indexed),
                )
            }
        }
    }

    fn server_report(&self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        let prepared = self
            .server
            .as_ref()
            .context("server analysis was not prepared")?;
        let filters = server_filters(report_type, options);
        let key = canonical_filter_key(&filters)?;
        let traversal = matches!(report_type, "serverRouteEdges" | "serverRouteRelated")
            || self.server_traversal_keys.contains(&key);
        let report = cached_analysis(
            &self.server_reports,
            &self.server_indexed_reports,
            &key,
            traversal,
            || crate::server_routes::analyze_project_with_prepared(prepared, &filters),
            || crate::server_routes::analyze_project_with_prepared_indexed(prepared, &filters),
        );
        let report = report?;
        match report {
            CachedAnalysis::Plain(report) => {
                render_server_report(report_type, options, prepared, &report, None, &filters)
            }
            CachedAnalysis::Indexed(indexed) => {
                let traversal_report =
                    matches!(report_type, "serverRouteEdges" | "serverRouteRelated");
                render_server_report(
                    report_type,
                    options,
                    prepared,
                    indexed.report(),
                    traversal_report.then_some(&indexed),
                    &filters,
                )
            }
        }
    }

    fn react_report(&self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        if report_type == "reactUsages" {
            let target = options
                .target
                .as_deref()
                .context("target is required for react usages")?;
            let include = crate::react_traits::UsagesInclude::parse(options.include.as_deref())?;
            let usages =
                crate::react_traits::pipeline::usages::run_usages_with_loaded_config_and_facts(
                    self.traversal.root(),
                    self.traversal.config(),
                    target,
                    &options.targets,
                    &include,
                    &self.facts,
                );
            let usages = usages?;
            return Ok(serde_json::to_value(usages)?);
        }
        let key = canonical_filter_key(&options.targets)?;
        let analysis = cached_once(&self.react_analyses, &key, || {
            crate::react_traits::pipeline::run_with_facts::run_analyze_with_loaded_config_and_facts(
                self.traversal.root(),
                self.traversal.config(),
                &options.targets,
                &self.facts,
            )
        })?;
        if report_type == "reactAnalyze" {
            return Ok(serde_json::to_value(analysis)?);
        }
        let prepared = crate::react_traits::prepare_check_from_loaded_config(
            self.traversal.config(),
            options.assert_no_fetch,
        );
        let findings = crate::react_traits::run_check_with_prepared_facts(
            self.traversal.root(),
            &options.targets,
            &self.facts,
            &prepared,
        );
        Ok(serde_json::to_value(findings?)?)
    }
}

enum CachedAnalysis<Plain, Indexed> {
    Plain(Plain),
    Indexed(Indexed),
}

fn cached_once<T: Clone>(
    cache: &ReportCache<T>,
    key: &str,
    compute: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let cell = {
        let mut cache = cache.lock().expect("report cache is poisoned");
        cache
            .entry(key.to_owned())
            .or_insert_with(|| std::sync::Arc::new(std::sync::OnceLock::new()))
            .clone()
    };
    cell.get_or_init(|| {
        compute().map_err(|error| std::sync::Arc::<str>::from(format!("{error:#}")))
    })
    .clone()
    .map_err(|message| anyhow::anyhow!("{message}"))
}

fn cached_analysis<Plain, Indexed>(
    plain: &ReportCache<Plain>,
    indexed: &ReportCache<Indexed>,
    key: &str,
    traversal: bool,
    analyze_plain: impl FnOnce() -> Result<Plain>,
    analyze_indexed: impl FnOnce() -> Result<Indexed>,
) -> Result<CachedAnalysis<Plain, Indexed>>
where
    Plain: Clone,
    Indexed: Clone,
{
    if traversal {
        return cached_once(indexed, key, analyze_indexed).map(CachedAnalysis::Indexed);
    }
    cached_once(plain, key, analyze_plain).map(CachedAnalysis::Plain)
}

fn canonical_filter_key(filters: &[String]) -> Result<String> {
    let mut filters = filters.to_vec();
    filters.sort();
    filters.dedup();
    Ok(serde_json::to_string(&filters)?)
}
