use super::FrontendApp;
use crate::config::v2::schema::RewriteRule;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn build_app(
    root: &Path,
    project: Option<String>,
    package_root: String,
    visible_paths: &[PathBuf],
    rewrites: &[RewriteRule],
    route_override: Option<String>,
) -> Result<FrontendApp> {
    let route_root = match (
        probe_route_root(root, &package_root, visible_paths),
        route_override,
    ) {
        (Some(route_root), _) => route_root,
        (None, Some(override_root)) => override_root,
        (None, None) => package_root.clone(),
    };
    Ok(FrontendApp {
        project,
        selector_roots: vec![package_root.clone()],
        root: package_root,
        route_root,
        rewrites: rewrites.to_vec(),
    })
}

/// Probe `<package_root>/src/app` then `<package_root>/app`, preferring
/// whichever actually has visible paths under it.
fn probe_route_root(root: &Path, package_root: &str, visible_paths: &[PathBuf]) -> Option<String> {
    for candidate in ["src/app", "app"] {
        let candidate_root = join_relative(package_root, candidate);
        let absolute = crate::codebase::ts_resolver::normalize_path(&root.join(&candidate_root));
        let exists = visible_paths
            .iter()
            .any(|path| crate::codebase::ts_resolver::normalize_path(path).starts_with(&absolute));
        if exists {
            return Some(candidate_root);
        }
    }
    None
}

fn join_relative(base: &str, suffix: &str) -> String {
    if base.is_empty() || base == "." {
        suffix.to_string()
    } else {
        format!("{base}/{suffix}")
    }
}

pub(super) fn relative_string(root: &Path, path: &Path) -> String {
    crate::codebase::ts_source::relative_slash_path(root, path)
}

pub(super) fn playwright_route_override(
    config: &crate::config::v2::schema::NoMistakesConfig,
    project: &str,
) -> Option<String> {
    config.tests.playwright.apps.values().find_map(|binding| {
        (binding.project.as_deref() == Some(project))
            .then(|| binding.frontend_root.clone())
            .flatten()
    })
}
