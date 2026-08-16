impl PreparedScope {
    fn queue_report(&self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        let key = canonical_filter_key(&options.filters)?;
        let traversal = matches!(report_type, "queueEdges" | "queueRelated")
            || self.queue_traversal_keys.contains(&key);
        let root = self.traversal.root();
        let tsconfig_catalog = self.traversal.tsconfig_catalog();
        let session = self.traversal.session_arc();
        let facts = &self.facts;
        let mut queue_reports = self
            .queue_reports
            .lock()
            .expect("queue report cache is poisoned");
        let mut queue_indexed_reports = self
            .queue_indexed_reports
            .lock()
            .expect("queue indexed report cache is poisoned");
        let report = cached_analysis(
            &mut queue_reports,
            &mut queue_indexed_reports,
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
        )?;
        match report {
            CachedAnalysis::Plain(report) => render_queue_report(report_type, options, report, None),
            CachedAnalysis::Indexed(indexed) => {
                let traversal_report = matches!(report_type, "queueEdges" | "queueRelated");
                render_queue_report(
                    report_type,
                    options,
                    indexed.report(),
                    traversal_report.then_some(indexed),
                )
            }
        }
    }

    fn server_report(&self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        let prepared = self.server.as_ref().context("server analysis was not prepared")?;
        let filters = server_filters(report_type, options);
        let key = canonical_filter_key(&filters)?;
        let traversal = matches!(report_type, "serverRouteEdges" | "serverRouteRelated")
            || self.server_traversal_keys.contains(&key);
        let mut server_reports = self
            .server_reports
            .lock()
            .expect("server report cache is poisoned");
        let mut server_indexed_reports = self
            .server_indexed_reports
            .lock()
            .expect("server indexed report cache is poisoned");
        let report = cached_analysis(
            &mut server_reports,
            &mut server_indexed_reports,
            &key,
            traversal,
            || crate::server_routes::analyze_project_with_prepared(prepared, &filters),
            || {
                crate::server_routes::analyze_project_with_prepared_indexed(prepared, &filters)
            },
        )?;
        match report {
            CachedAnalysis::Plain(report) => {
                render_server_report(report_type, options, prepared, report, None, &filters)
            }
            CachedAnalysis::Indexed(indexed) => {
                let traversal_report =
                    matches!(report_type, "serverRouteEdges" | "serverRouteRelated");
                render_server_report(
                    report_type,
                    options,
                    prepared,
                    indexed.report(),
                    traversal_report.then_some(indexed),
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
            return Ok(serde_json::to_value(
                crate::react_traits::pipeline::usages::run_usages_with_loaded_config_and_facts(
                    self.traversal.root(),
                    self.traversal.config(),
                    target,
                    &options.targets,
                    &include,
                    &self.facts,
                )?,
            )?);
        }
        let key = canonical_filter_key(&options.targets)?;
        let analysis = {
            let cache = self
                .react_analyses
                .lock()
                .expect("react analysis cache is poisoned");
            cache.get(&key).cloned()
        };
        let analysis = match analysis {
            Some(analysis) => analysis,
            None => {
                let computed = crate::react_traits::pipeline::run_with_facts::run_analyze_with_loaded_config_and_facts(
                    self.traversal.root(),
                    self.traversal.config(),
                    &options.targets,
                    &self.facts,
                )?;
                self.react_analyses
                    .lock()
                    .expect("react analysis cache is poisoned")
                    .entry(key)
                    .or_insert(computed)
                    .clone()
            }
        };
        if report_type == "reactAnalyze" {
            return Ok(serde_json::to_value(analysis)?);
        }
        let prepared = crate::react_traits::prepare_check_from_loaded_config(
            self.traversal.config(),
            options.assert_no_fetch,
        );
        Ok(serde_json::to_value(
            crate::react_traits::run_check_with_prepared_facts(
                self.traversal.root(),
                &options.targets,
                &self.facts,
                &prepared,
            )?,
        )?)
    }
}

enum CachedAnalysis<'a, Plain, Indexed> {
    Plain(&'a Plain),
    Indexed(&'a Indexed),
}

fn cached_analysis<'a, Plain, Indexed>(
    plain: &'a mut HashMap<String, Plain>,
    indexed: &'a mut HashMap<String, Indexed>,
    key: &str,
    traversal: bool,
    analyze_plain: impl FnOnce() -> Result<Plain>,
    analyze_indexed: impl FnOnce() -> Result<Indexed>,
) -> Result<CachedAnalysis<'a, Plain, Indexed>> {
    if traversal {
        if !indexed.contains_key(key) {
            indexed.insert(key.to_owned(), analyze_indexed()?);
        }
        return Ok(CachedAnalysis::Indexed(
            indexed.get(key).expect("indexed report is cached"),
        ));
    }
    if let Some(report) = indexed.get(key) {
        return Ok(CachedAnalysis::Indexed(report));
    }
    if !plain.contains_key(key) {
        plain.insert(key.to_owned(), analyze_plain()?);
    }
    Ok(CachedAnalysis::Plain(
        plain.get(key).expect("plain report is cached"),
    ))
}

fn canonical_filter_key(filters: &[String]) -> Result<String> {
    let mut filters = filters.to_vec();
    filters.sort();
    filters.dedup();
    Ok(serde_json::to_string(&filters)?)
}
