fn render_queue_report(
    report_type: &str,
    options: &ProjectOptions,
    report: &crate::queue::ProjectReport,
) -> Result<Value> {
    match report_type {
        "queues" => Ok(json_value(report)),
        "queueEdges" => {
            let depth = crate::cli::root_scoped_edge_depth(&options.files, options.depth);
            Ok(json_value(&crate::cli::edge_view(
                &report.edges,
                &options.files,
                depth,
            )))
        }
        "queueRelated" => {
            if options.files.is_empty() {
                bail!("files must contain at least one file");
            }
            let direction =
                crate::napi_api::options::parse_queue_direction(options.direction.as_deref())?;
            Ok(json_value(&crate::queue::related(
                report,
                &options.files,
                direction,
            )))
        }
        "queueCheck" => Ok(json_value(&report.check)),
        _ => bail!("unknown queue report type: {report_type}"),
    }
}

fn server_filters(report_type: &str, options: &ProjectOptions) -> Vec<String> {
    let mut filters = options.filters.clone();
    if report_type == "serverContracts" {
        filters.extend(crate::napi_api::options::project_roots(options));
    }
    filters
}

fn render_server_report(
    report_type: &str,
    options: &ProjectOptions,
    prepared: &crate::server_routes::PreparedServerAnalysis,
    report: &crate::server_routes::ProjectReport,
    filters: &[String],
) -> Result<Value> {
    match report_type {
        "serverRoutes" => Ok(json_value(report)),
        "serverRouteList" => {
            let routes = report
                .routes
                .iter()
                .filter(|route| {
                    options.files.is_empty()
                        || options
                            .files
                            .iter()
                            .any(|file| file == &route.file || file == &route.route)
                })
                .collect::<Vec<_>>();
            Ok(json_value(&routes))
        }
        "serverRouteEdges" => {
            let roots = crate::napi_api::options::project_roots(options);
            let depth = crate::cli::root_scoped_edge_depth(&roots, options.depth);
            Ok(json_value(&crate::cli::edge_view(
                &report.edges,
                &roots,
                depth,
            )))
        }
        "serverRouteRelated" => {
            let roots = crate::napi_api::options::project_roots(options);
            if roots.is_empty() {
                bail!("files or roots must contain at least one entry");
            }
            let direction =
                crate::napi_api::options::parse_server_direction(options.direction.as_deref())?;
            Ok(json_value(&crate::server_routes::related(
                report, &roots, direction,
            )))
        }
        "serverContracts" => {
            let contracts =
                crate::server_routes::analyze_contracts_with_prepared(prepared, report, filters)?;
            Ok(json_value(&contracts))
        }
        _ => bail!("unknown server report type: {report_type}"),
    }
}

fn render_playwright_report(
    report_type: &str,
    options: &PlaywrightOptions,
    root: &Path,
    analysis: &crate::playwright::analysis::types::Analysis,
) -> Result<Value> {
    let files = options.files.iter().map(PathBuf::from).collect::<Vec<_>>();
    match report_type {
        "playwrightCheck" => Ok(json_value(&analysis.coverage)),
        "playwrightEdges" => Ok(json_value(&analysis.edges)),
        "playwrightRelated" => {
            if files.is_empty() {
                bail!("files must contain at least one file");
            }
            Ok(json_value(
                &crate::playwright::analysis::output::build_related_report(
                    root,
                    &analysis.edges.edges,
                    &files,
                ),
            ))
        }
        "playwrightTests" => Ok(json_value(
            &crate::playwright::analysis::tests_report::build_tests_report(
                &analysis.edges.edges,
                &files,
                root,
            ),
        )),
        _ => bail!("unknown Playwright report type: {report_type}"),
    }
}

fn json_value<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("prepared N-API report serialization never fails")
}
