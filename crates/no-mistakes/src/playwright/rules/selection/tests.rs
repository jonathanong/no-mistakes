use super::*;
use crate::config::v2::schema::{PlaywrightAppBinding, RewriteRule, RuleDef, RuleTestTargets};

fn app(project: &str) -> FrontendApp {
    FrontendApp {
        project: Some(project.to_string()),
        root: project.to_string(),
        route_root: format!("{project}/app"),
        selector_roots: vec![project.to_string()],
        rewrites: Vec::new(),
    }
}

fn rule(rule_id: &str, playwright: Vec<&str>, projects: Vec<&str>) -> RuleDef {
    RuleDef {
        rule: rule_id.to_string(),
        projects: projects.into_iter().map(str::to_string).collect(),
        tests: RuleTestTargets {
            playwright: playwright.into_iter().map(str::to_string).collect(),
            ..RuleTestTargets::default()
        },
        ..RuleDef::default()
    }
}

/// No `type: nextjs` project configured at all: the selection's app is
/// `None`, not an error — settings resolution falls back to the pre-#624
/// bare default.
#[test]
fn no_binding_no_apps_resolves_to_none() {
    let config = NoMistakesConfig {
        rules: vec![rule(PLAYWRIGHT_COVERAGE, vec!["web"], vec![])],
        ..NoMistakesConfig::default()
    };

    let selections = rule_selections(&config, &[]).unwrap();

    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0].app, None);
}

/// Exactly one frontend app and no explicit binding: it's used automatically.
#[test]
fn no_binding_single_app_resolves_automatically() {
    let config = NoMistakesConfig {
        rules: vec![rule(PLAYWRIGHT_COVERAGE, vec!["web"], vec![])],
        ..NoMistakesConfig::default()
    };
    let apps = vec![app("control-web")];

    let selections = rule_selections(&config, &apps).unwrap();

    assert_eq!(selections[0].app.as_deref(), Some("control-web"));
}

/// The #624 case: two frontend apps configured, no binding anywhere. This
/// used to silently pick whichever project's Next.js settings sorted first;
/// it must now be a clear, actionable error instead.
#[test]
fn no_binding_multiple_apps_is_an_error() {
    let config = NoMistakesConfig {
        rules: vec![rule(PLAYWRIGHT_COVERAGE, vec!["web"], vec![])],
        ..NoMistakesConfig::default()
    };
    let apps = vec![app("agent-web"), app("control-web")];

    let error = rule_selections(&config, &apps).unwrap_err();

    let message = format!("{error:#}");
    assert!(message.contains("agent-web"), "{message}");
    assert!(message.contains("control-web"), "{message}");
    assert!(
        message.contains("tests.playwright.apps.web.project"),
        "{message}"
    );
}

/// `rules[].projects` names exactly one frontend app: that binds the
/// selection, even with other ambiguous apps present.
#[test]
fn rule_projects_naming_one_app_binds() {
    let config = NoMistakesConfig {
        rules: vec![rule(
            PLAYWRIGHT_COVERAGE,
            vec!["control"],
            vec!["control-web"],
        )],
        ..NoMistakesConfig::default()
    };
    let apps = vec![app("agent-web"), app("control-web")];

    let selections = rule_selections(&config, &apps).unwrap();

    assert_eq!(selections[0].app.as_deref(), Some("control-web"));
}

/// `rules[].projects` mixes a frontend app with an unrelated (non-frontend)
/// project name: the frontend app still binds; the other name is irrelevant
/// to app resolution (it may exist purely for the rule's own path-filter
/// scope).
#[test]
fn rule_projects_mixed_with_non_frontend_project_binds_to_the_frontend_one() {
    let config = NoMistakesConfig {
        rules: vec![rule(
            PLAYWRIGHT_COVERAGE,
            vec!["control"],
            vec!["control-web", "shared-ui-lib"],
        )],
        ..NoMistakesConfig::default()
    };
    let apps = vec![app("control-web")];

    let selections = rule_selections(&config, &apps).unwrap();

    assert_eq!(selections[0].app.as_deref(), Some("control-web"));
}

/// `rules[].projects` names only non-frontend projects: this does not fall
/// through to the sole-app default (which would silently analyze an app the
/// rule never named) — it's an explicit error.
#[test]
fn rule_projects_naming_no_frontend_app_is_an_error() {
    let config = NoMistakesConfig {
        rules: vec![rule(
            PLAYWRIGHT_COVERAGE,
            vec!["control"],
            vec!["backend-api"],
        )],
        ..NoMistakesConfig::default()
    };
    let apps = vec![app("control-web")];

    let error = rule_selections(&config, &apps).unwrap_err();

    assert!(format!("{error:#}").contains("backend-api"));
}

/// Two rule applications targeting the same Playwright project disagree on
/// which app to bind: a Playwright project can exercise at most one app, so
/// this is an error rather than "last one wins".
#[test]
fn conflicting_rule_projects_for_the_same_playwright_project_is_an_error() {
    let config = NoMistakesConfig {
        rules: vec![
            rule(PLAYWRIGHT_COVERAGE, vec!["shared"], vec!["control-web"]),
            rule(
                PLAYWRIGHT_UNIQUE_TEST_IDS,
                vec!["shared"],
                vec!["agent-web"],
            ),
        ],
        ..NoMistakesConfig::default()
    };
    let apps = vec![app("agent-web"), app("control-web")];

    let error = rule_selections(&config, &apps).unwrap_err();

    let message = format!("{error:#}");
    assert!(message.contains("agent-web"), "{message}");
    assert!(message.contains("control-web"), "{message}");
}

/// An unbound rule with `tests.playwright.apps` fans out to each Playwright
/// project instead of erroring on multiple frontend apps.
#[test]
fn unbound_rule_fans_out_over_configured_apps() {
    let mut config = NoMistakesConfig {
        rules: vec![rule(PLAYWRIGHT_COVERAGE, vec![], vec![])],
        ..NoMistakesConfig::default()
    };
    config.tests.playwright.apps.insert(
        "control".to_string(),
        PlaywrightAppBinding {
            project: Some("control-web".to_string()),
            ..PlaywrightAppBinding::default()
        },
    );
    config.tests.playwright.apps.insert(
        "agent".to_string(),
        PlaywrightAppBinding {
            project: Some("agent-web".to_string()),
            ..PlaywrightAppBinding::default()
        },
    );
    let apps = vec![app("agent-web"), app("control-web")];

    let selections = rule_selections(&config, &apps).unwrap();
    let mut names: Vec<_> = selections
        .iter()
        .map(|selection| {
            (
                selection.playwright_project.as_deref(),
                selection.app.as_deref(),
            )
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            (Some("agent"), Some("agent-web")),
            (Some("control"), Some("control-web")),
        ]
    );
}

/// `tests.playwright.apps.<project>.project` wins outright, even over
/// ambiguous apps and even when no rule names any project at all.
#[test]
fn explicit_apps_binding_overrides_ambiguity() {
    let mut config = NoMistakesConfig {
        rules: vec![rule(PLAYWRIGHT_COVERAGE, vec!["control"], vec![])],
        ..NoMistakesConfig::default()
    };
    config.tests.playwright.apps.insert(
        "control".to_string(),
        PlaywrightAppBinding {
            project: Some("control-web".to_string()),
            ..PlaywrightAppBinding::default()
        },
    );
    let apps = vec![app("agent-web"), app("control-web")];

    let selections = rule_selections(&config, &apps).unwrap();

    assert_eq!(selections[0].app.as_deref(), Some("control-web"));
}

/// `tests.playwright.apps.<project>.project` naming a typo'd/unconfigured
/// project must fail loudly instead of silently binding to a nonexistent
/// app (which would fall through settings resolution to the ambiguity-free
/// bare default, hiding the mistake).
#[test]
fn explicit_apps_binding_naming_an_unconfigured_project_is_an_error() {
    let mut config = NoMistakesConfig {
        rules: vec![rule(PLAYWRIGHT_COVERAGE, vec!["control"], vec![])],
        ..NoMistakesConfig::default()
    };
    config.tests.playwright.apps.insert(
        "control".to_string(),
        PlaywrightAppBinding {
            project: Some("typo-web".to_string()),
            ..PlaywrightAppBinding::default()
        },
    );
    let apps = vec![app("agent-web"), app("control-web")];

    let error = rule_selections(&config, &apps).unwrap_err();

    let message = format!("{error:#}");
    assert!(message.contains("typo-web"), "{message}");
    assert!(message.contains("agent-web"), "{message}");
    assert!(message.contains("control-web"), "{message}");
}

/// An apps entry that sets frontendRoot, selectorRoots, and rewrites without
/// naming a `project` is fully explicit: settings resolution does not need
/// a frontend app, so unbound-rule fan-out must not fail on app count.
#[test]
fn fully_explicit_apps_binding_skips_app_ambiguity() {
    let mut config = NoMistakesConfig {
        rules: vec![rule(PLAYWRIGHT_COVERAGE, vec![], vec![])],
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
    let apps = vec![app("agent-web"), app("control-web")];

    let selections = rule_selections(&config, &apps).unwrap();

    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0].playwright_project.as_deref(), Some("control"));
    assert_eq!(selections[0].app, None);
}
