use crate::fetches::analyze::resolve::relative_string;
use crate::fetches::analyze::routes::collect_layout_chain_files_from_visible;
use crate::fetches::cli::Cli;
use crate::fetches::pipeline::aggregate::build_final_report;
use crate::fetches::pipeline::cache::Cache;
use crate::fetches::pipeline::route_analysis::{check_route_matches, RouteMatchContext};
use crate::fetches::pipeline::target::{resolve_target_file, TargetSpec};
use crate::fetches::report::types::{FinalReport, RouteReport};
use anyhow::Result;
use no_mistakes::routes;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) fn run_with_base_root(base_root: &Path, cli: &Cli) -> Result<FinalReport> {
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(
        no_mistakes::diagnostics::current(),
    );
    run_with_base_root_and_session(base_root, cli, &session)
}

pub(crate) fn run_with_base_root_and_session(
    base_root: &Path,
    cli: &Cli,
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
) -> Result<FinalReport> {
    let requested_root = base_root.join(&cli.root);
    let root = requested_root
        .canonicalize()
        .unwrap_or_else(|_| requested_root.clone());
    if !root.is_dir() {
        anyhow::bail!("root directory does not exist: {}", root.display());
    }

    let snapshot = session.visible_paths(&root);
    let visible_paths = snapshot.paths_for(&root);
    let v2 = session.config(&root, cli.config.as_deref())?;
    // Falls back to a single `<root>/app` app when nothing is configured or
    // inferable, matching the pre-existing zero-signal default; a genuinely
    // ambiguous multi-app repository with no binding still errors (see
    // `frontend_apps`).
    let apps = no_mistakes::config::v2::frontend_apps_or_default(&root, &v2, &visible_paths)?;
    let visible_files = visible_paths
        .iter()
        .map(|path| no_mistakes::codebase::ts_resolver::normalize_path(path))
        .collect::<no_mistakes::fx::PathSet>();
    let stems = ["page", "route"];

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut parsed_files = no_mistakes::fetch::ParsedFileCache::default();
    let target_specs = resolve_targets(base_root, &root, &cli.targets)?;

    let mut reports = Vec::new();
    let mut matched_targets: HashSet<String> = HashSet::new();
    for app in &apps {
        let frontend_root = root.join(&app.route_root);
        if !frontend_root.is_dir() {
            anyhow::bail!(
                "frontend root directory does not exist: {} (app: {})",
                frontend_root.display(),
                app.project.as_deref().unwrap_or("<default>"),
            );
        }
        let route_paths = snapshot.paths_for(&frontend_root);
        let mut all_routes =
            routes::collect_routes_from_visible_paths(&frontend_root, &route_paths, &stems);
        let virtual_routes = routes::rewrites::expand_rewrites(&app.rewrites, &all_routes);
        all_routes.extend(virtual_routes);

        let analyzed = analyze_routes(
            all_routes,
            AnalyzeRoutesContext {
                target_specs: &target_specs,
                frontend_root: &frontend_root,
                root: &root,
                cache: &mut cache,
                session,
                parsed_files: &mut parsed_files,
                visible_files: &visible_files,
            },
        );
        let (app_reports, app_matched_targets) = analyzed?;
        reports.extend(app_reports);
        matched_targets.extend(app_matched_targets);
    }

    verify_targets_matched(&target_specs, &matched_targets)?;

    Ok(build_final_report(reports))
}

fn resolve_targets(base_root: &Path, root: &Path, targets: &[String]) -> Result<Vec<TargetSpec>> {
    let mut target_specs = Vec::new();
    let mut unique_targets = HashSet::new();
    for target in targets {
        if unique_targets.insert(target.clone()) {
            // Targets that look like route patterns (e.g. "/users") won't resolve as files;
            // that's expected — `file: None` causes route-pattern matching downstream.
            let file = resolve_target_file(root, target)
                .or_else(|_| resolve_target_file(base_root, target))
                .ok();
            target_specs.push(TargetSpec {
                raw: target.clone(),
                file,
            });
        }
    }
    Ok(target_specs)
}

struct AnalyzeRoutesContext<'a> {
    target_specs: &'a [TargetSpec],
    frontend_root: &'a Path,
    root: &'a Path,
    cache: &'a mut Cache,
    session: &'a no_mistakes::codebase::analysis_session::AnalysisSession,
    parsed_files: &'a mut no_mistakes::fetch::ParsedFileCache,
    visible_files: &'a no_mistakes::fx::PathSet,
}

fn analyze_routes(
    all_routes: Vec<routes::Route>,
    context: AnalyzeRoutesContext<'_>,
) -> Result<(Vec<RouteReport>, HashSet<String>)> {
    let AnalyzeRoutesContext {
        target_specs,
        frontend_root,
        root,
        cache,
        session,
        parsed_files,
        visible_files,
    } = context;
    let mut reports = Vec::new();
    let mut matched_targets: HashSet<String> = HashSet::new();

    for route in all_routes {
        let route_is_page = route.file.file_stem().and_then(|s| s.to_str()) == Some("page");
        let wrapper_files = if route_is_page {
            collect_layout_chain_files_from_visible(&route.file, frontend_root, visible_files)
                .into_iter()
                .filter_map(|path| path.canonicalize().ok())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let (matched, newly_matched) = check_route_matches(
            &route,
            target_specs,
            &wrapper_files,
            RouteMatchContext {
                cache,
                session,
                parsed_files,
                root,
                visible_files,
            },
        )?;

        for t in newly_matched {
            matched_targets.insert(t);
        }

        if !matched {
            continue;
        }

        let fetches =
            no_mistakes::fetch::collect_route_fetches_from_visible_with_facts_and_session(
                session,
                &route,
                frontend_root,
                root,
                cache,
                parsed_files,
                visible_files,
            )?;

        reports.push(RouteReport {
            route: route.pattern,
            file: relative_string(root, &route.file),
            api_calls: fetches,
        });
    }

    Ok((reports, matched_targets))
}

fn verify_targets_matched(
    target_specs: &[TargetSpec],
    matched_targets: &HashSet<String>,
) -> Result<()> {
    let unique_target_raws: HashSet<_> = target_specs.iter().map(|t| t.raw.as_str()).collect();
    let mut unmatched: Vec<_> = unique_target_raws
        .iter()
        .copied()
        .filter(|target| !matched_targets.contains(*target))
        .collect();
    if !unmatched.is_empty() {
        unmatched.sort();
        return Err(anyhow::anyhow!("Error: targets not found: {:?}", unmatched));
    }
    Ok(())
}
