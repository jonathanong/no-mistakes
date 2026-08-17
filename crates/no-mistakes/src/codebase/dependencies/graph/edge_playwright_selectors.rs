/// Build selector dependencies after the initial graph exists so text-locator
/// reachability can reuse its RouteImport edges instead of constructing a
/// second graph.
pub(super) struct PlaywrightSelectorEdgeInputs<'a> {
    all_files: &'a [PathBuf],
    facts: Option<&'a dyn TsFactLookup>,
    partial_graph: Option<&'a DepGraph>,
    graph_tsconfig: Option<&'a TsConfig>,
    snapshot: &'a crate::playwright::fsutil::VisiblePathSnapshot,
    prepared_settings: &'a [crate::playwright::config::Settings],
    interner: &'a PathInterner,
}

impl PlaywrightSelectorEdgeInputs<'_> {
    fn route_import(&self) -> Option<(&DepGraph, &crate::codebase::ts_resolver::TsConfig)> {
        self.partial_graph.zip(self.graph_tsconfig)
    }
}

include!("edge_playwright_selector_collect.rs");
include!("edge_playwright_selector_collect_text.rs");

/// Build selector edges for every resolved frontend app. `prepared_settings`
/// empty means no app was prepared — fall back to loading exactly one
/// `Settings` from disk, the pre-multi-app behavior. A prepared caller
/// instead supplies one `Settings` per app (see `PreparedGraphConfig`), so a
/// multi-`type: nextjs`-app repository's selector edges aren't limited to a
/// single arbitrarily-chosen app.
///
/// One app's analysis failing does not discard edges already found for other
/// apps: it is skipped in favor of the others, the same tolerance the route
/// collector (`collect_playwright_route_edges_from_snapshot`) already has —
/// a single broken app's settings should not blank out an otherwise-working
/// multi-app graph build.
pub(super) fn collect_playwright_selector_edges_with_graph(
    root: &Path,
    config_path: Option<&Path>,
    inputs: PlaywrightSelectorEdgeInputs<'_>,
) -> Result<Vec<Edge>> {
    if inputs.prepared_settings.is_empty() {
        let settings = crate::playwright::config::load_settings_from_visible(
            root,
            config_path,
            &[],
            None,
            None,
            inputs.snapshot,
        )?;
        return collect_playwright_selector_edges_for_settings(root, &settings, &inputs);
    }
    // Apps are independent after the base graph exists: each settings
    // projection can scan selectors without waiting on the others.
    let observer = crate::diagnostics::current();
    let mut edges: Vec<Edge> = inputs
        .prepared_settings
        .par_iter()
        .flat_map(|settings| {
            crate::diagnostics::with_observer(observer.clone(), || {
                crate::diagnostics::with_timing_kind(
                    crate::diagnostics::TimingKind::Parallel,
                    || {
                        crate::ast::with_owned_request_parse_cache(|| {
                            collect_playwright_selector_edges_for_settings(root, settings, &inputs)
                                .unwrap_or_default()
                        })
                    },
                )
            })
        })
        .collect();
    edges.sort();
    edges.dedup();
    Ok(edges)
}

fn selector_edges_from_analysis(
    root: &Path,
    all_files: &[PathBuf],
    analysis: &crate::playwright::analysis::types::Analysis,
    interner: &PathInterner,
) -> Vec<Edge> {
    selector_edges_from_playwright_edges(root, all_files, &analysis.edges.edges, interner)
}

fn selector_edges_from_playwright_edges(
    root: &Path,
    all_files: &[PathBuf],
    playwright_edges: &[crate::playwright::analysis::types::Edge],
    interner: &PathInterner,
) -> Vec<Edge> {
    let file_set: std::collections::HashSet<&Path> =
        all_files.iter().map(PathBuf::as_path).collect();
    let mut edges = Vec::new();
    for playwright_edge in playwright_edges {
        if let Some((from, to, kind)) = selector_dep_edge(root, playwright_edge, interner) {
            if from.as_file().is_some_and(|path| file_set.contains(path))
                && to.as_file().is_some_and(|path| file_set.contains(path))
            {
                edges.push((from, to, kind));
            }
        }
    }
    edges
}

fn selector_dep_edge(
    root: &Path,
    edge: &crate::playwright::analysis::types::Edge,
    interner: &PathInterner,
) -> Option<Edge> {
    let (app_file_rel, test_file_rel) = match edge {
        crate::playwright::analysis::types::Edge::Selector {
            app_file,
            test_file,
            ..
        } => (app_file.as_str(), test_file.as_str()),
        crate::playwright::analysis::types::Edge::LocatorText {
            app_file,
            test_file,
            ..
        } => (app_file.as_str(), test_file.as_str()),
        _ => return None,
    };
    Some((
        NodeId::file_in(interner, root.join(test_file_rel)),
        NodeId::file_in(interner, root.join(app_file_rel)),
        EdgeKind::Selector,
    ))
}
