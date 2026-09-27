use crate::codebase::dependencies::graph::TsFactLookup;
use crate::playwright::analysis::types::{Edge, RouteCoverageAttribution};
use crate::playwright::{
    analysis::routes_index::{route_index, route_specificity},
    matcher,
};
use crate::playwright::{config::Settings, fsutil::relative_string, routes::Route};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::sync::Arc;

/// Project prepared integration ownership into route edges. Literal matching
/// uses normal specificity before applying the exact canonical allowlist.
pub(crate) fn edges(
    root: &Path,
    settings: &Settings,
    routes: &[Route],
    facts: Option<&dyn TsFactLookup>,
) -> Result<Vec<Edge>> {
    let mut edges = Vec::new();
    let index = route_index(root, routes);
    for source in &settings.route_coverage_sources {
        for path in &source.routes {
            anyhow::ensure!(
                routes.iter().any(|route| route.pattern == *path),
                "routeCoverageSources route `{path}` is not a canonical route of this app"
            );
        }
        let links = facts
            .and_then(|facts| facts.integration_route_links(source))
            .ok_or_else(|| {
                anyhow!("routeCoverageSources requires prepared integration route facts")
            })?
            .as_ref()
            .map_err(|error| anyhow!("{error}"))?;
        for link in links {
            let segments = matcher::reference_segments(&link.occurrence.value);
            let matching = index
                .candidates(&segments)
                .into_iter()
                .filter(|route| matcher::matches_segments(&segments, &route.segments))
                .collect::<Vec<_>>();
            let specificity = matching
                .iter()
                .map(|route| route_specificity(&route.segments))
                .max();
            for route in matching.into_iter().filter(|route| {
                Some(route_specificity(&route.segments)) == specificity
                    && source.routes.contains(route.pattern.as_ref())
            }) {
                edges.push(Edge::Route {
                    test_file: Arc::new(relative_string(root, &link.entry_file)),
                    test_name: link.occurrence.test_name.clone().map(Arc::new),
                    describe_path: Arc::new(link.occurrence.describe_path.clone()),
                    route_file: route.route_file.clone(),
                    route: route.pattern.clone(),
                    url: Arc::new(link.occurrence.value.clone()),
                    hook: false,
                    line: link.occurrence.line,
                    attribution: Some(RouteCoverageAttribution {
                        framework: source.framework,
                        project: source.project.clone(),
                        declaration_file: relative_string(root, &link.declaration_file),
                    }),
                });
            }
        }
    }
    edges.sort();
    edges.dedup();
    Ok(edges)
}
