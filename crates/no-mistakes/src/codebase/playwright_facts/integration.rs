use super::occurrence::TestOccurrence;
use crate::config::v2::schema::RouteCoverageSource;
use std::collections::BTreeMap;
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

pub(crate) type PreparedLinks = BTreeMap<RouteCoverageSource, Result<Vec<RouteLink>, String>>;

pub(crate) fn validate(sources: &[RouteCoverageSource]) -> anyhow::Result<()> {
    for source in sources {
        anyhow::ensure!(
            source.empty_required_field().is_none(),
            "routeCoverageSources requires project, include, routes, and helpers"
        );
        if let Some((_, route)) = source.first_noncanonical_route() {
            anyhow::bail!(
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
