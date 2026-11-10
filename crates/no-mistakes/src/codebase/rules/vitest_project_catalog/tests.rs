use super::PreparedVitestProjectCatalog;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::integration_tests::types::ConfigProject;

impl PreparedVitestProjectCatalog {
    pub(crate) fn from_load_result(
        config_projects: Result<Vec<ConfigProject>, CheckInputFailure>,
    ) -> Self {
        Self {
            config_projects,
            explicit_projects: Vec::new(),
        }
    }
}
