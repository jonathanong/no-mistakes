use super::*;
use crate::config::v2::schema::{Project, ProjectType, RewriteRule, RuleDef, RuleTestTargets};
use crate::playwright::fsutil::VisiblePathSnapshot;
use std::collections::BTreeMap;

fn nextjs_project(root: &str) -> Project {
    Project {
        type_: Some(ProjectType::Nextjs),
        root: Some(root.to_string()),
        ..Project::default()
    }
}

/// A Playwright project whose binding sets `frontendRoot`, `selectorRoots`,
/// and `rewrites` all explicitly needs no frontend-app resolution at all —
/// not even when the repository configures two ambiguous `type: nextjs`
/// projects and neither is named.
#[test]
fn all_explicit_bypasses_app_resolution_even_when_ambiguous() {
    let mut config = NoMistakesConfig {
        projects: BTreeMap::from([
            ("agent-web".to_string(), nextjs_project("agent")),
            ("control-web".to_string(), nextjs_project("control")),
        ]),
        ..NoMistakesConfig::default()
    };
    config.tests.playwright.apps.insert(
        "control".to_string(),
        PlaywrightAppBinding {
            frontend_root: Some("explicit/route".to_string()),
            selector_roots: vec!["explicit/selectors".to_string()],
            rewrites: vec![RewriteRule {
                source: "/a".to_string(),
                destination: "/b".to_string(),
            }],
            ..PlaywrightAppBinding::default()
        },
    );
    let root = Path::new("/repo");
    let snapshot = VisiblePathSnapshot::from_paths(root, &[]);

    let settings = settings_from_v2(
        root,
        &config,
        &[],
        Some("control".to_string()),
        None,
        &snapshot,
    )
    .unwrap();

    assert_eq!(settings.frontend_root, "explicit/route");
    assert_eq!(
        settings.selector_roots,
        vec!["explicit/selectors".to_string()]
    );
    assert_eq!(settings.rewrites.len(), 1);
}

/// `tests.playwright.apps.<project>.project` names a project that isn't
/// actually a configured (or resolvable) `type: nextjs` project.
#[test]
fn explicit_binding_project_not_found_is_an_error() {
    let mut config = NoMistakesConfig {
        projects: BTreeMap::from([("web".to_string(), nextjs_project("web"))]),
        ..NoMistakesConfig::default()
    };
    config.tests.playwright.apps.insert(
        "control".to_string(),
        PlaywrightAppBinding {
            project: Some("missing-app".to_string()),
            ..PlaywrightAppBinding::default()
        },
    );
    let root = Path::new("/repo");
    let snapshot = VisiblePathSnapshot::from_paths(root, &[]);

    let error = settings_from_v2(
        root,
        &config,
        &[],
        Some("control".to_string()),
        None,
        &snapshot,
    )
    .unwrap_err();

    assert!(format!("{error:#}").contains("missing-app"));
}

/// A direct `settings_from_v2` call (bypassing rule/selection-level app
/// resolution, e.g. the standalone `playwright check` CLI) with two
/// ambiguous apps and no binding at all must still fail loudly rather than
/// guess.
#[test]
fn ambiguous_apps_with_no_binding_is_an_error_on_direct_calls() {
    let config = NoMistakesConfig {
        projects: BTreeMap::from([
            ("agent-web".to_string(), nextjs_project("agent")),
            ("control-web".to_string(), nextjs_project("control")),
        ]),
        ..NoMistakesConfig::default()
    };
    let root = Path::new("/repo");
    let snapshot = VisiblePathSnapshot::from_paths(root, &[]);

    let error = settings_from_v2(root, &config, &[], None, None, &snapshot).unwrap_err();

    let message = format!("{error:#}");
    assert!(message.contains("agent-web"), "{message}");
    assert!(message.contains("control-web"), "{message}");
}

/// The deepest zero-signal fallback prefers a nested `<nextjs_root>/app/app`
/// when it exists — a legacy quirk predating any real Next.js project
/// awareness, retained unchanged (see the doc comment on
/// `default_frontend_root`).
#[test]
fn default_frontend_root_prefers_a_nested_app_app_when_present() {
    let root = Path::new("/repo");
    let visible = vec![root.join("app/app/page.tsx")];

    assert_eq!(default_frontend_root(root, "app", &visible), "app/app");
}

#[test]
fn default_frontend_root_falls_back_to_the_bare_literal() {
    let root = Path::new("/repo");

    assert_eq!(default_frontend_root(root, "app", &[]), "app");
}

/// A rule's own `tests.playwright: [...]` list is enough to opt a repository
/// into v2 app resolution, even with zero top-level `tests.playwright.*`
/// settings — `rules[].projects` is the documented default binding
/// mechanism, and it must not silently fall through to the bare-literal
/// `settings_from_defaults` path just because no other Playwright setting
/// happens to be configured.
#[test]
fn rule_scoped_playwright_target_alone_opts_into_v2_resolution() {
    let config = NoMistakesConfig {
        projects: BTreeMap::from([("web".to_string(), nextjs_project("web/src/app"))]),
        rules: vec![RuleDef {
            rule: "playwright-coverage".to_string(),
            projects: vec!["web".to_string()],
            tests: RuleTestTargets {
                playwright: vec!["main".to_string()],
                ..RuleTestTargets::default()
            },
            ..RuleDef::default()
        }],
        ..NoMistakesConfig::default()
    };
    let root = Path::new("/repo");
    let snapshot = VisiblePathSnapshot::from_paths(root, &[]);

    let settings =
        settings_from_loaded_v2(root, &config, &[], None, Some("web".to_string()), &snapshot)
            .unwrap();

    assert_eq!(settings.frontend_root, "web/src/app");
}
