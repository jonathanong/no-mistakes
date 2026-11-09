fn collect_file_facts_from_source(
    session: super::FactRequest<'_>,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    source: Arc<str>,
    retain_parse: bool,
) -> Option<CheckFileFacts> {
    if plan.storybook && is_mdx_file(path) {
        let stored_source = should_store_source(plan, path).then(|| Arc::clone(&source));
        let storybook =
            crate::codebase::check_facts::family_scope::applies(&plan.storybook_paths, path)
                .then(|| crate::codebase::storybook::extract_mdx_source(&source));
        return Some(CheckFileFacts {
            ts: Arc::new(ts_source(stored_source.clone())),
            source: stored_source,
            storybook,
            server_route_client_boundary: plan.server_route_client_boundary.then(Default::default),
            ..CheckFileFacts::default()
        });
    }
    if plan.raw_source
        && should_store_source(plan, path)
        && !requires_parse(root, plan, path, playwright)
    {
        return Some(CheckFileFacts {
            ts: Arc::new(ts_source(Some(Arc::clone(&source)))),
            source: Some(Arc::clone(&source)),
            ..CheckFileFacts::default()
        });
    }
    if !requires_parse(root, plan, path, playwright) {
        return Some(CheckFileFacts::default());
    }
    crate::diagnostics::hot_files::trace(
        crate::diagnostics::hot_files::FilePass::Parse,
        path,
        source.len() as u64,
        || {
            crate::diagnostics::hot_files::with_file(path, || {
                let legacy_symbols = plan
                    .legacy_symbol_paths
                    .contains(&crate::codebase::ts_resolver::normalize_path(path));
                let collect = |program: &oxc_ast::ast::Program<'_>,
                               parsed_source: &str,
                               parse_error: Option<String>,
                               fatal_parse_error: bool| {
                    if let Some(parse_error) = parse_error {
                        let stored_source =
                            should_store_source(plan, path).then(|| Arc::clone(&source));
                        let mut ts = super::file_parse_error::ts_facts(
                            plan,
                            stored_source.clone(),
                            parsed_source,
                            program,
                            parse_error.clone(),
                        );
                        ts.fatal_parse_error = fatal_parse_error;
                        let symbols = (legacy_symbols && (plan.symbols || plan.graph.symbols))
                            .then(|| {
                                Arc::new(crate::codebase::ts_symbols::extract_symbols_from_program(
                                    program,
                                    parsed_source,
                                ))
                            });
                        if let Some(symbols) = &symbols {
                            ts.symbols = Some(Arc::clone(symbols));
                        }
                        let integration_runner_config =
                            plan.integration_runner_configs.as_ref().and_then(|plan| {
                                plan.parse_error(
                                    path,
                                    format!("failed to parse {}: {parse_error}", path.display()),
                                )
                            });
                        let embedded_sql = embedded_sql::prepared_embedded_sql(
                            root,
                            path,
                            parsed_source,
                            program,
                            plan,
                        );
                        return CheckFileFacts {
                            lint_suppressions: plan.collect_lint_suppressions(
                                path,
                                program,
                                parsed_source,
                            ),
                            sql_source_prerequisites: super::SqlSourcePrerequisites::failed_source(
                                root, path, plan,
                            ),
                            sql_identifier_syntax: sql_identifier::invalid(
                                root,
                                path,
                                parsed_source,
                                program,
                                plan,
                            ),
                            ts: Arc::new(ts),
                            source: stored_source,
                            symbols,
                            integration_runner_config,
                            embedded_sql,
                            parse_error: Some(parse_error),
                            parsed: true,
                            server_route_client_boundary: plan
                                .server_route_client_boundary
                                .then(Default::default),
                            ..CheckFileFacts::default()
                        };
                    }
                    // Fatal parser results always carry a diagnostic and return above.
                    program::collect_file_facts_from_program_with_discovery(
                        root,
                        path,
                        plan,
                        playwright,
                        parsed_source,
                        program,
                        (should_store_source(plan, path)
                            || sql_identifier::policy_selected(root, path, plan))
                        .then(|| Arc::clone(&source)),
                        session.collect_tests(plan, path),
                    )
                };
                let collected = if legacy_symbols {
                    session.with_legacy_symbols_program(path, &source, |program, source, error| {
                        collect(program, source, error, false)
                    })
                } else {
                    session.with_recovered_program_status(path, &source, collect)
                };
                let facts = match collected {
                    Ok(facts) => Some(facts),
                    Err(error) => {
                        let stored_source =
                            should_store_source(plan, path).then(|| Arc::clone(&source));
                        let parse_error = if legacy_symbols {
                            error.to_string()
                        } else {
                            format!("unsupported file type: {}", path.display())
                        };
                        let facts = CheckFileFacts {
                            ts: Arc::new(TsFileFacts {
                                fatal_parse_error: legacy_symbols,
                                parse_error: Some(parse_error.clone()),
                                source: stored_source.clone(),
                                ..TsFileFacts::default()
                            }),
                            source: stored_source,
                            parse_error: Some(parse_error),
                            server_route_client_boundary: plan
                                .server_route_client_boundary
                                .then(Default::default),
                            ..CheckFileFacts::default()
                        };
                        Some(with_failed_source(root, path, plan, facts))
                    }
                };
                if !retain_parse {
                    crate::ast::evict_request_parse_cache_path(path);
                }
                facts
            })
        },
    )
}
