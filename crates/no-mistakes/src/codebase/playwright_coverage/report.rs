fn collect_report_with_frontend_root(
    root: &Path,
    frontend_root: &Path,
    test_globs: Vec<String>,
    all_files: &[PathBuf],
) -> Result<CoverageReport> {
    let mut routes = defs_frontend::collect_frontend_routes_from_files(frontend_root, all_files);
    if let Ok(v2) = crate::config::v2::load_v2_config(root, None) {
        let view = crate::config::v2::ConfigView::new(&v2);
        let virtual_routes =
            crate::routes::rewrites::expand_rewrites_as_tuples(view.nextjs_rewrites(), &routes);
        routes.extend(virtual_routes);
    }
    let visits: Vec<(String, String)> = collect_playwright_visits(root, &test_globs, all_files)?
        .into_iter()
        .map(|visit| (visit.url, relative_string(root, &visit.file)))
        .collect();

    let mut route_coverages: Vec<RouteCoverage> = routes
        .into_par_iter()
        .map(|(file, route)| {
            let mut tests: Vec<RouteTestHit> = visits
                .iter()
                .filter(|(url, _)| matcher::matches(url, &route))
                .map(|(url, rel_file)| RouteTestHit {
                    file: rel_file.clone(),
                    url: url.clone(),
                })
                .collect();
            tests.sort();
            RouteCoverage {
                route,
                file: relative_string(root, &file),
                covered: !tests.is_empty(),
                tests,
            }
        })
        .collect();

    route_coverages.sort_by(compare_route_coverage);

    let total = route_coverages.len();
    let covered = route_coverages.iter().filter(|route| route.covered).count();
    let uncovered = total.saturating_sub(covered);
    let coverage_percent = if total == 0 {
        100.0
    } else {
        (covered as f64 / total as f64) * 100.0
    };

    Ok(CoverageReport {
        summary: CoverageSummary {
            total,
            covered,
            uncovered,
            coverage_percent,
        },
        routes: route_coverages,
    })
}

fn compare_route_coverage(a: &RouteCoverage, b: &RouteCoverage) -> Ordering {
    let route_order = a.route.cmp(&b.route);
    if route_order != Ordering::Equal {
        return route_order;
    }
    a.file.cmp(&b.file)
}

fn collect_playwright_visits(
    root: &Path,
    test_globs: &[String],
    all_files: &[PathBuf],
) -> Result<Vec<PlaywrightVisit>> {
    let globset = build_globset(test_globs)?;
    let mut visits: Vec<PlaywrightVisit> = all_files
        .par_iter()
        .filter(|path| {
            path.strip_prefix(root)
                .map(|rel| globset.is_match(rel))
                .unwrap_or(false)
        })
        .flat_map_iter(|path| {
            let Ok(source) = std::fs::read_to_string(path) else {
                return Vec::new();
            };
            crate::codebase::dependencies::graph::playwright::extract_playwright_urls(&source)
                .into_iter()
                .map(|url| PlaywrightVisit {
                    file: path.clone(),
                    url,
                })
                .collect::<Vec<_>>()
        })
        .collect();
    visits.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.url.cmp(&b.url)));
    visits.dedup_by(|a, b| a.file == b.file && a.url == b.url);
    Ok(visits)
}

fn build_globset(globs: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob).context(format!("invalid glob `{glob}`"))?);
    }
    Ok(builder.build()?)
}

fn relative_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}
