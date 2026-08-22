use super::schema::{NoMistakesConfig, Project, ProjectType, RewriteRule};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

mod build;
use build::{build_app, playwright_route_override, relative_string};

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
    let nextjs_projects = configured_nextjs_projects(config);
    if nextjs_projects.is_empty() {
        return Ok(inferred_anonymous_app(root, visible_paths)
            .into_iter()
            .collect());
    }
    nextjs_projects
        .into_iter()
        .map(|(name, project)| resolve_named_app(root, name, project, config, visible_paths))
        .collect()
}

/// Same as [`frontend_apps`], but resolves each configured `type: nextjs`
/// project independently: a project whose root can't be inferred is skipped
/// rather than failing the whole call. Only for callers that treat the
/// result as a best-effort convenience list (`no-mistakes graph`'s rewrite
/// union) rather than the authoritative app set a Playwright rule needs to
/// resolve against — those must keep using [`frontend_apps`], whose
/// fail-fast `Result` is what makes an ambiguous/unresolvable app an actual
/// configuration error instead of a silently smaller app set.
pub fn frontend_apps_lenient(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Vec<FrontendApp> {
    let nextjs_projects = configured_nextjs_projects(config);
    if nextjs_projects.is_empty() {
        return inferred_anonymous_app(root, visible_paths)
            .into_iter()
            .collect();
    }
    nextjs_projects
        .into_iter()
        .filter_map(|(name, project)| {
            resolve_named_app(root, name, project, config, visible_paths).ok()
        })
        .collect()
}

fn configured_nextjs_projects(config: &NoMistakesConfig) -> Vec<(&str, &Project)> {
    config
        .projects
        .iter()
        .filter(|(_, project)| project.type_ == Some(ProjectType::Nextjs))
        .map(|(name, project)| (name.as_str(), project))
        .collect()
}

fn inferred_anonymous_app(root: &Path, visible_paths: &[PathBuf]) -> Option<FrontendApp> {
    let package_root =
        crate::codebase::config::infer_nextjs_root_from_visible(root, visible_paths)?;
    let package_root = relative_string(root, &package_root);
    Some(
        build_app(
            root,
            None,
            package_root,
            visible_paths,
            &[],
            RouteRootFallback::PackageRoot,
            None,
        )
        .expect("anonymous app package-root fallback never fails"),
    )
}

fn resolve_named_app(
    root: &Path,
    name: &str,
    project: &Project,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Result<FrontendApp> {
    let package_root = match project.root.as_deref() {
        Some(configured) => configured.to_string(),
        None => {
            let inferred =
                crate::codebase::config::infer_nextjs_root_from_visible(root, visible_paths)
                    .ok_or_else(|| {
                        anyhow!(
                            "cannot infer the Next.js app root for project `{name}`: the \
                             repository has either no `next.config.*` file or more than \
                             one, so a single app root can't be chosen.\n\
                             Set `projects.{name}.root` explicitly."
                        )
                    })?;
            relative_string(root, &inferred)
        }
    };
    build_app(
        root,
        Some(name.to_string()),
        package_root,
        visible_paths,
        &project.rewrites,
        RouteRootFallback::Error,
        playwright_route_override(config, name),
    )
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

pub(super) enum RouteRootFallback {
    PackageRoot,
    Error,
}

#[cfg(test)]
mod tests;
