use super::schema::{NoMistakesConfig, Project, ProjectType, RewriteRule};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// A single frontend (Next.js) application resolved from `.no-mistakes.yml`.
///
/// Playwright rules, `no-mistakes graph`, and `no-mistakes fetches` all need
/// to answer "where is this app's route tree" and "where are its testable
/// selectors". A repository can have more than one such app (for example two
/// Next.js services sharing a monorepo); [`frontend_apps`] resolves the
/// complete, deterministic set instead of picking one, which is what caused
/// <https://github.com/jonathanong/no-mistakes/issues/624>.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontendApp {
    /// The `.no-mistakes.yml` `projects:` key this app was configured under,
    /// or `None` when it was inferred because no `type: nextjs` project is
    /// configured at all.
    pub project: Option<String>,
    /// Repo-relative package root (e.g. `services/web`).
    pub root: String,
    /// Repo-relative App Router route directory (e.g. `services/web/src/app`).
    ///
    /// Resolved by preferring `<root>/src/app` over `<root>/app`, mirroring
    /// Next.js's own `src/`-directory precedence
    /// (<https://nextjs.org/docs/app/building-your-application/configuring/src-directory>).
    /// See <https://github.com/jonathanong/no-mistakes/issues/625>.
    pub route_root: String,
    /// Repo-relative roots scanned for selector usage. Defaults to `[root]`
    /// (the whole package, not just `route_root`) so narrowing route
    /// discovery to `src/app` does not also drop selector coverage for
    /// sibling directories like `src/components`.
    pub selector_roots: Vec<String>,
    pub rewrites: Vec<RewriteRule>,
}

/// Resolve the complete set of configured/inferred frontend apps, in
/// deterministic `projects:` key order.
///
/// - Every `type: nextjs` project becomes one app. An explicit `root:` wins;
///   otherwise the root is inferred by searching for a `next.config.*` file,
///   which only succeeds when the search is unambiguous (see
///   [`crate::codebase::config::infer_nextjs_root_from_visible`]) — an
///   unresolvable root is an error, not a silent fallback.
/// - When no `type: nextjs` project is configured at all, a single anonymous
///   app is inferred the same way; when inference itself finds nothing, the
///   result is an empty list (not an error — only consumers that actually
///   need a frontend app should fail).
pub fn frontend_apps(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Result<Vec<FrontendApp>> {
    let nextjs_projects: Vec<(&str, &Project)> = config
        .projects
        .iter()
        .filter(|(_, project)| project.type_ == Some(ProjectType::Nextjs))
        .map(|(name, project)| (name.as_str(), project))
        .collect();

    if nextjs_projects.is_empty() {
        return Ok(
            crate::codebase::config::infer_nextjs_root_from_visible(root, visible_paths)
                .map(|package_root| {
                    let package_root = relative_string(root, &package_root);
                    vec![build_app(root, None, package_root, visible_paths, &[])]
                })
                .unwrap_or_default(),
        );
    }

    nextjs_projects
        .into_iter()
        .map(|(name, project)| {
            let package_root = match project.root.as_deref() {
                Some(configured) => configured.to_string(),
                None => {
                    let inferred = crate::codebase::config::infer_nextjs_root_from_visible(
                        root,
                        visible_paths,
                    )
                    .ok_or_else(|| {
                        anyhow!(
                            "cannot infer the Next.js app root for project `{name}`: no \
                                     single `next.config.*` file was found in the repository.\n\
                                     Set `projects.{name}.root` explicitly."
                        )
                    })?;
                    relative_string(root, &inferred)
                }
            };
            Ok(build_app(
                root,
                Some(name.to_string()),
                package_root,
                visible_paths,
                &project.rewrites,
            ))
        })
        .collect()
}

/// Same as [`frontend_apps`], but never returns an empty list: when nothing
/// is configured and nothing can be inferred (no `type: nextjs` project, no
/// discoverable `next.config.*`), falls back to a single anonymous app
/// rooted at `<root>/app` — the historical zero-signal default every
/// consumer (Playwright settings, `fetches`, `graph`) already relied on
/// before this module existed. Use this instead of [`frontend_apps`] when a
/// genuinely unconfigured repository must still resolve to *something*
/// rather than fail outright; callers that should treat "no apps" as a real
/// error keep using [`frontend_apps`] directly.
pub fn frontend_apps_or_default(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Result<Vec<FrontendApp>> {
    let apps = frontend_apps(root, config, visible_paths)?;
    if apps.is_empty() {
        Ok(vec![FrontendApp {
            project: None,
            root: String::new(),
            route_root: "app".to_string(),
            selector_roots: vec!["app".to_string()],
            rewrites: Vec::new(),
        }])
    } else {
        Ok(apps)
    }
}

fn build_app(
    root: &Path,
    project: Option<String>,
    package_root: String,
    visible_paths: &[PathBuf],
    rewrites: &[RewriteRule],
) -> FrontendApp {
    let route_root = probe_route_root(root, &package_root, visible_paths);
    FrontendApp {
        project,
        selector_roots: vec![package_root.clone()],
        root: package_root,
        route_root,
        rewrites: rewrites.to_vec(),
    }
}

/// Probe `<package_root>/src/app` then `<package_root>/app`, preferring
/// whichever actually has visible paths under it; falls back to
/// `package_root` itself when neither exists (the previous, still-supported
/// behavior for apps that keep pages outside an `app` directory).
fn probe_route_root(root: &Path, package_root: &str, visible_paths: &[PathBuf]) -> String {
    for candidate in ["src/app", "app"] {
        let candidate_root = join_relative(package_root, candidate);
        let absolute = crate::codebase::ts_resolver::normalize_path(&root.join(&candidate_root));
        let exists = visible_paths
            .iter()
            .any(|path| crate::codebase::ts_resolver::normalize_path(path).starts_with(&absolute));
        if exists {
            return candidate_root;
        }
    }
    package_root.to_string()
}

fn join_relative(base: &str, suffix: &str) -> String {
    if base.is_empty() || base == "." {
        suffix.to_string()
    } else {
        format!("{base}/{suffix}")
    }
}

fn relative_string(root: &Path, path: &Path) -> String {
    crate::codebase::ts_source::relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
