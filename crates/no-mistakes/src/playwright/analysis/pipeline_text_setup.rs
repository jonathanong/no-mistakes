use crate::codebase::dependencies::graph::{
    DepGraph, GraphBuildPlan, GraphFiles, RouteReachableFiles, TsFactLookup,
};
use crate::codebase::ts_resolver::TsConfig;
use crate::playwright::analysis::app_text::collect_app_text_targets;
use crate::playwright::analysis::route_reachability::{
    collect_route_reachable_files, collect_route_source_files,
};
use crate::playwright::analysis::text_edges::AppTextIndex;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::config::Settings;
use crate::playwright::routes::Route;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct TextResolutionSetup {
    pub(crate) app_text_targets: Arc<Vec<AppTextTarget>>,
    pub(crate) app_text_index: AppTextIndex,
    pub(crate) route_reachable_files: Arc<RouteReachableFiles>,
    pub(crate) has_matching_text_candidate: bool,
}

pub(crate) struct TextResolutionInputs<'a> {
    pub(crate) facts: Option<&'a dyn TsFactLookup>,
    pub(crate) graph_file_universe: Option<&'a [PathBuf]>,
    pub(crate) route_import_candidate: Option<(&'a DepGraph, &'a TsConfig)>,
    pub(crate) routes: &'a [Route],
    pub(crate) has_eligible_text_locator: bool,
    pub(crate) has_text_candidate: &'a dyn Fn(&[AppTextTarget], &AppTextIndex) -> bool,
    pub(crate) has_route_reachability_demand: &'a dyn Fn(&[AppTextTarget], &AppTextIndex) -> bool,
}

pub(crate) fn build_text_resolution_setup(
    root: &Path,
    settings: &Settings,
    inputs: TextResolutionInputs<'_>,
) -> Result<TextResolutionSetup> {
    let TextResolutionInputs {
        facts,
        graph_file_universe,
        route_import_candidate,
        routes,
        has_eligible_text_locator,
        has_text_candidate,
        has_route_reachability_demand,
    } = inputs;
    if !has_eligible_text_locator {
        return Ok(empty_text_setup());
    }
    let app_text_targets =
        crate::perf_trace::trace("playwright.app_text_targets", || match facts {
            Some(facts) => {
                facts.get_or_compute_app_text_targets(&|| collect_app_text_targets(root, settings))
            }
            None => collect_app_text_targets(root, settings).map(Arc::new),
        })?;
    let app_text_index = AppTextIndex::new(app_text_targets.as_slice());
    let has_matching_text_candidate = !app_text_targets.is_empty()
        && has_text_candidate(app_text_targets.as_slice(), &app_text_index);
    if !has_matching_text_candidate
        || !has_route_reachability_demand(app_text_targets.as_slice(), &app_text_index)
    {
        return Ok(TextResolutionSetup {
            app_text_targets,
            app_text_index,
            route_reachable_files: Arc::new(Default::default()),
            has_matching_text_candidate,
        });
    }
    let compute = || {
        let source_files = collect_route_source_files(root, settings)?;
        let supplied_graph = supplied_route_import_graph(
            root,
            settings,
            route_import_candidate,
            &source_files.graph_files,
        )?;
        let owned_graph = match supplied_graph {
            Some(_) => None,
            None => Some(build_route_import_graph(
                root,
                settings,
                facts,
                graph_file_universe,
                &source_files.graph_files,
            )?),
        };
        collect_route_reachable_files(
            root,
            settings,
            routes,
            supplied_graph
                .or(owned_graph.as_ref())
                .expect("route-import graph is supplied or built"),
            &source_files,
        )
    };
    let route_reachable_files =
        crate::perf_trace::trace("playwright.route_reachable_files", || match facts {
            Some(facts) => facts.get_or_compute_route_reachable_files(&compute),
            None => compute().map(Arc::new),
        })?;
    Ok(TextResolutionSetup {
        app_text_targets,
        app_text_index,
        route_reachable_files,
        has_matching_text_candidate,
    })
}

fn empty_text_setup() -> TextResolutionSetup {
    TextResolutionSetup {
        app_text_targets: Arc::new(Vec::new()),
        app_text_index: AppTextIndex::default(),
        route_reachable_files: Arc::new(Default::default()),
        has_matching_text_candidate: false,
    }
}

fn supplied_route_import_graph<'a>(
    root: &Path,
    settings: &Settings,
    candidate: Option<(&'a DepGraph, &TsConfig)>,
    source_files: &[PathBuf],
) -> Result<Option<&'a DepGraph>> {
    let Some((graph, graph_tsconfig)) = candidate else {
        return Ok(None);
    };
    let route_tsconfig = load_route_import_tsconfig(root, settings)?;
    Ok((route_tsconfig == *graph_tsconfig
        && source_files.iter().all(|file| graph.contains_file(file)))
    .then_some(graph))
}

pub(crate) fn build_route_import_graph(
    root: &Path,
    settings: &Settings,
    facts: Option<&dyn TsFactLookup>,
    graph_file_universe: Option<&[PathBuf]>,
    route_source_files: &[PathBuf],
) -> Result<DepGraph> {
    let tsconfig = load_route_import_tsconfig(root, settings)?;
    let mut paths = graph_file_universe
        .or_else(|| facts.and_then(TsFactLookup::graph_files))
        .map(<[PathBuf]>::to_vec)
        .unwrap_or_else(|| GraphFiles::discover(root).all().to_vec());
    paths.extend_from_slice(route_source_files);
    paths.sort();
    paths.dedup();
    DepGraph::build_with_plan_files_config_and_facts(
        root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            ..GraphBuildPlan::default()
        },
        &GraphFiles::from_files(paths),
        None,
        facts,
    )
}

pub(crate) fn load_route_import_tsconfig(root: &Path, settings: &Settings) -> Result<TsConfig> {
    let frontend_root = root.join(&settings.frontend_root);
    Ok(crate::codebase::ts_resolver::find_tsconfig(&frontend_root)
        .or_else(|| crate::codebase::ts_resolver::find_tsconfig(root))
        .map(|path| crate::codebase::ts_resolver::load_tsconfig(&path))
        .transpose()?
        .unwrap_or_else(|| TsConfig {
            dir: root.to_path_buf(),
            paths: Vec::new(),
            paths_dir: root.to_path_buf(),
            base_url: None,
        }))
}
