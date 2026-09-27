//! Supplemental route facts retain runner and helper ownership. They never
//! pretend that an integration test is a browser navigation.
use crate::config::v2::schema::RouteCoverageSource;
use crate::playwright::playwright_tests::TestOccurrence;
use std::path::PathBuf;

pub(crate) fn configured(config: &crate::config::v2::NoMistakesConfig) -> bool {
    !config.tests.playwright.route_coverage_sources.is_empty()
        || config
            .tests
            .playwright
            .apps
            .values()
            .any(|app| !app.route_coverage_sources.is_empty())
}

mod edges;
mod extract;
mod links;
mod literals;
mod prepare;
mod receivers;
#[cfg(test)]
mod tests;
pub(crate) use edges::edges;
pub(crate) use links::{prepare_links, PreparedLinks};
pub(crate) use prepare::runner_plan;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct RouteOccurrence {
    pub(crate) source: RouteCoverageSource,
    pub(crate) occurrence: TestOccurrence<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct RouteLink {
    pub(crate) source: RouteCoverageSource,
    pub(crate) entry_file: PathBuf,
    pub(crate) declaration_file: PathBuf,
    pub(crate) occurrence: TestOccurrence<String>,
}

pub(crate) use extract::collect;

pub(crate) fn validate(sources: &[RouteCoverageSource]) -> anyhow::Result<()> {
    for source in sources {
        anyhow::ensure!(
            !source.project.is_empty()
                && !source.include.is_empty()
                && !source.routes.is_empty()
                && !source.helpers.is_empty(),
            "routeCoverageSources requires project, include, routes, and helpers"
        );
        for route in &source.routes {
            anyhow::ensure!(
                route.starts_with('/')
                    && !route.contains(['*', '[', ']', '{', '}', '?', '#'])
                    && !route.starts_with("//"),
                "routeCoverageSources.routes requires exact canonical route identifiers: {route}"
            );
        }
        for helper in &source.helpers {
            anyhow::ensure!(!helper.module.is_empty() && !helper.export.is_empty()
                && helper.method.as_ref().is_none_or(|method| !method.is_empty()),
                "routeCoverageSources.helpers requires a module, export, and a nonempty optional method");
        }
    }
    Ok(())
}
