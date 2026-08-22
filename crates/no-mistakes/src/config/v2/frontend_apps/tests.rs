use super::{frontend_apps, frontend_apps_lenient};
use crate::codebase::ts_source::discover_visible_paths;
use crate::config::v2::discover::load_v2_config;
use crate::config::v2::schema::{NoMistakesConfig, PlaywrightAppBinding};
use std::path::Path;

fn fixture(sub: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-cases/config-v2")
        .join(sub)
        .join("fixture")
}

/// #625: `src/app` is preferred over `app` when both could theoretically
/// exist, mirroring Next.js's own `src/`-directory precedence.
#[test]
fn route_root_prefers_src_app_over_app() {
    let dir = fixture("frontend-apps-src-app");
    let cfg = load_v2_config(&dir, None).unwrap();
    let visible = discover_visible_paths(&dir);
    let apps = frontend_apps(&dir, &cfg, &visible).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].project.as_deref(), Some("control-web"));
    assert_eq!(apps[0].root, "services/web");
    assert_eq!(apps[0].route_root, "services/web/src/app");
    // #625's regression: selectorRoots stays at the package root, not the
    // narrowed route root, so sibling directories like `src/components`
    // keep selector coverage.
    assert_eq!(apps[0].selector_roots, vec!["services/web".to_string()]);
}

/// Back-compat: repos on the plain `<root>/app` layout still resolve to
/// `app`, not `src/app` (which does not exist here).
#[test]
fn route_root_falls_back_to_app_layout() {
    let dir = fixture("frontend-apps-app-layout");
    let cfg = load_v2_config(&dir, None).unwrap();
    let visible = discover_visible_paths(&dir);
    let apps = frontend_apps(&dir, &cfg, &visible).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].route_root, "web/app");
}

/// Neither `src/app` nor `app` exists on a named `type: nextjs` project:
/// `frontend_apps` still falls back to the package root so graph/check
/// consumers keep working. Playwright `frontendRoot` overrides still win
/// when configured.
#[test]
fn named_project_without_app_dir_falls_back_to_package_root() {
    let dir = fixture("frontend-apps-no-app-dir");
    let cfg = load_v2_config(&dir, None).unwrap();
    let visible = discover_visible_paths(&dir);
    let apps = frontend_apps(&dir, &cfg, &visible).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].root, "web");
    assert_eq!(apps[0].route_root, "web");
}

#[test]
fn named_project_uses_playwright_frontend_root_override() {
    let dir = fixture("frontend-apps-no-app-dir");
    let mut cfg = load_v2_config(&dir, None).unwrap();
    cfg.tests.playwright.apps.insert(
        "chromium".to_string(),
        PlaywrightAppBinding {
            project: Some("web".to_string()),
            frontend_root: Some("web/pages".to_string()),
            ..PlaywrightAppBinding::default()
        },
    );
    let visible = discover_visible_paths(&dir);
    let apps = frontend_apps(&dir, &cfg, &visible).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].route_root, "web/pages");
}

/// No `type: nextjs` project is configured at all: a single anonymous app is
/// inferred from a unique `next.config.*` file.
#[test]
fn infers_a_single_anonymous_app_with_no_configured_project() {
    let dir = fixture("frontend-apps-inferred");
    let cfg = NoMistakesConfig::default();
    let visible = discover_visible_paths(&dir);
    let apps = frontend_apps(&dir, &cfg, &visible).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].project, None);
    assert_eq!(apps[0].root, "");
    assert_eq!(apps[0].route_root, "src/app");
}

/// No `type: nextjs` project and no `next.config.*` file: the app list is
/// empty rather than an error, since inference genuinely found nothing.
#[test]
fn empty_when_nothing_can_be_inferred() {
    let dir = fixture("frontend-apps-no-app-dir");
    let cfg = NoMistakesConfig::default();
    let visible = discover_visible_paths(&dir);
    let apps = frontend_apps(&dir, &cfg, &visible).unwrap();
    assert!(apps.is_empty());
}

/// An unset `root:` that cannot be inferred uniquely (two `next.config.*`
/// files) is an explicit error naming the project, not a silent guess.
#[test]
fn ambiguous_inferred_root_is_an_error() {
    let dir = fixture("frontend-apps-ambiguous");
    let cfg = load_v2_config(&dir, None).unwrap();
    let visible = discover_visible_paths(&dir);
    let error = frontend_apps(&dir, &cfg, &visible).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("project `web`"), "{message}");
    assert!(message.contains("projects.web.root"), "{message}");
}

/// The #624 regression at the resolver layer: two `type: nextjs` projects
/// each produce their own independent app — distinct roots, route roots, and
/// rewrites — instead of one project's settings silently winning. `agent-web`
/// sorts before `control-web`, which is the exact ordering that made
/// `agent-web` win everything before this fix.
#[test]
fn multiple_nextjs_projects_resolve_independently() {
    let dir = fixture("frontend-apps-multi-project");
    let cfg = load_v2_config(&dir, None).unwrap();
    let visible = discover_visible_paths(&dir);
    let apps = frontend_apps(&dir, &cfg, &visible).unwrap();
    assert_eq!(apps.len(), 2);

    let agent = apps
        .iter()
        .find(|app| app.project.as_deref() == Some("agent-web"))
        .unwrap();
    assert_eq!(agent.root, "services/agent-web");
    assert_eq!(agent.route_root, "services/agent-web/src/app");
    assert_eq!(agent.rewrites.len(), 1);
    assert_eq!(agent.rewrites[0].source, "/agent-posts/:slug*");

    let control = apps
        .iter()
        .find(|app| app.project.as_deref() == Some("control-web"))
        .unwrap();
    assert_eq!(control.root, "services/control-web");
    assert_eq!(control.route_root, "services/control-web/src/app");
    assert_eq!(control.rewrites.len(), 1);
    assert_eq!(control.rewrites[0].source, "/control-posts/:slug*");
}

/// `frontend_apps` fails the whole call when any one of several `type:
/// nextjs` projects can't resolve its root — the strict contract Playwright
/// rule resolution needs. `frontend_apps_lenient` instead skips only the
/// unresolvable project, so a best-effort consumer (`no-mistakes graph`'s
/// rewrite union) keeps the valid app's rewrite instead of losing every
/// app's rewrites to one sibling's ambiguity.
#[test]
fn lenient_resolution_skips_only_the_unresolvable_app() {
    let dir = fixture("frontend-apps-partial-failure");
    let cfg = load_v2_config(&dir, None).unwrap();
    let visible = discover_visible_paths(&dir);

    assert!(frontend_apps(&dir, &cfg, &visible).is_err());

    let apps = frontend_apps_lenient(&dir, &cfg, &visible);
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].project.as_deref(), Some("good-web"));
    assert_eq!(apps[0].rewrites.len(), 1);
    assert_eq!(apps[0].rewrites[0].source, "/good-posts/:slug*");
}
