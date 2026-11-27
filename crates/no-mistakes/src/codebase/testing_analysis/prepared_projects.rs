use super::{
    discovered_tests::DiscoveredTests, project_filter::ProjectTestFilter, runner::TestRunner,
};
use crate::codebase::dependencies::graph::VitestSetupProject;
use crate::codebase::dotnet::DotnetFactMap;
use crate::codebase::swift::SwiftFactMap;
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::facts::TsFactMap;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::integration_routes::RouteOccurrence;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(crate) trait PreparedProjects: Send + Sync {
    fn discover_tests(
        &self,
        root: &Path,
        config: &NoMistakesConfig,
        runner: TestRunner,
        visible_paths: &[PathBuf],
        tsconfig: &TsConfig,
    ) -> anyhow::Result<DiscoveredTests>;

    fn project_filters(&self) -> Vec<(TestRunner, ProjectTestFilter)>;
    fn graph_facts(&self) -> &TsFactMap;
    fn graph_route_occurrences(&self) -> &BTreeMap<PathBuf, Vec<RouteOccurrence>>;
    fn dotnet_facts(&self) -> Option<&DotnetFactMap>;
    fn swift_facts(&self) -> Option<&SwiftFactMap>;
    fn vitest_setup_projects(&self) -> Vec<VitestSetupProject>;
}
