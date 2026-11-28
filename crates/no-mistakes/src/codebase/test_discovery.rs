mod cargo_targets;
mod dotnet_projects;
mod filters;
mod lang_projects;
mod lang_targets;
mod ownership;
pub(crate) use super::testing_analysis::project_patterns;
mod projects;
mod reserved;
mod swift_projects;
mod targets;
pub(crate) mod traversal;
mod types;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub use super::testing_analysis::plan::FrameworkPreparationPlan;
pub(crate) use filters::{compile_project_filter, fallback_runner_match};
pub use filters::{fallback_test_path, ProjectTestFilter};
use ownership::owning_projects;
pub use project_patterns::project_dependency_patterns;
pub use targets::TestExecutionTarget;
pub use types::{DiscoveredTests, PreparedRunnerProject, TestRunner};
include!("test_discovery/prepared.rs");
include!("test_discovery/prepared_catalog.rs");
include!("test_discovery/prepared_vitest_resolution.rs");
include!("test_discovery/prepared_vitest_reparse.rs");
include!("test_discovery/prepared_vitest_setup.rs");
include!("test_discovery/discover.rs");
include!("test_discovery/api.rs");

pub fn literal_path_glob(path: &str) -> String {
    let mut escaped = String::with_capacity(path.len());
    for ch in path.chars() {
        if matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

fn resolve_tsconfig_lossy(
    root: &Path,
    visible_paths: &[PathBuf],
) -> crate::codebase::ts_resolver::TsConfig {
    crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, visible_paths)
        .unwrap_or_else(|_| crate::codebase::ts_resolver::TsConfig {
            dir: root.to_path_buf(),
            paths: Vec::new(),
            paths_dir: root.to_path_buf(),
            base_url: None,
        })
}
