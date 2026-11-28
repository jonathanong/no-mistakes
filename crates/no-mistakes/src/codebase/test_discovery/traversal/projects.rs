use super::super::{
    discover_tests_from_prepared_projects, DiscoveredTests, PreparedTestProjects, ProjectTestFilter,
};
use super::*;
use crate::codebase::dependencies::graph::VitestSetupProject;
use crate::codebase::dotnet::DotnetFactMap;
use crate::codebase::playwright_facts::integration::RouteOccurrence;
use crate::codebase::swift::SwiftFactMap;
use crate::codebase::ts_source::facts::TsFactMap;
use std::collections::BTreeMap;

impl PreparedProjects for PreparedTestProjects {
    fn discover_tests(
        &self,
        root: &Path,
        config: &NoMistakesConfig,
        runner: TestRunner,
        visible_paths: &[PathBuf],
        tsconfig: &TsConfig,
    ) -> Result<DiscoveredTests> {
        discover_tests_from_prepared_projects(root, config, runner, self, visible_paths, tsconfig)
    }

    fn project_filters(&self) -> Vec<(TestRunner, ProjectTestFilter)> {
        self.project_filters()
    }

    fn graph_facts(&self) -> &TsFactMap {
        self.graph_facts()
    }

    fn graph_route_occurrences(&self) -> &BTreeMap<PathBuf, Vec<RouteOccurrence>> {
        self.graph_route_occurrences()
    }

    fn dotnet_facts(&self) -> Option<&DotnetFactMap> {
        self.dotnet_facts()
    }

    fn swift_facts(&self) -> Option<&SwiftFactMap> {
        self.swift_facts()
    }

    fn vitest_setup_projects(&self) -> Vec<VitestSetupProject> {
        self.vitest_setup_projects()
    }
}
