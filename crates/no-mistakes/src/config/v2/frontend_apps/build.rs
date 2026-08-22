use super::{FrontendApp, RouteRootFallback};
use crate::config::v2::schema::RewriteRule;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn build_app(
    root: &Path,
    project: Option<String>,
    package_root: String,
    visible_paths: &[PathBuf],
    rewrites: &[RewriteRule],
    fallback: RouteRootFallback,
) -> Result<FrontendApp> {
    let route_root = match (
        probe_route_root(root, &package_root, visible_paths),
        fallback,
    ) {
        (Some(route_root), _) => route_root,
        (None, RouteRootFallback::PackageRoot) => package_root.clone(),
        (None, RouteRootFallback::Error) => {
            let name = project.as_deref().unwrap_or("<anonymous>");
            anyhow::bail!(
                "cannot resolve the Next.js App Router directory for project `{name}` \
                 (looked for `{package_root}/src/app` and `{package_root}/app`).\n\
                 Add an `app` or `src/app` directory, or set \
                 `tests.playwright.apps.<playwright-project>.frontendRoot`."
            );
        }
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
