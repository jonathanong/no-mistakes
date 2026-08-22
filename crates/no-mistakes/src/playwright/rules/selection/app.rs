use crate::config::v2::NoMistakesConfig;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;

/// Resolve which single frontend app a Playwright-project selection
/// exercises, in precedence order:
///
/// 1. `tests.playwright.apps.<playwright_project>.project` — fully explicit,
///    always wins outright (no ambiguity check against rule bindings).
/// 2. An apps entry that sets `frontendRoot`, `selectorRoots`, and `rewrites`
///    without naming a `project` needs no frontend app; return `None`.
/// 3. The frontend-app names collected from every contributing rule
///    application's `projects:` list. Exactly one match binds; zero matches
///    (the list named only non-frontend projects) or more than one match
///    (rules disagree, or one rule names two apps) is an error rather than a
///    silent guess.
/// 4. No rule named a project at all: the sole configured frontend app, or
///    `None` when there are zero apps (not ambiguous — nothing to choose
///    between), or an error when there is more than one.
pub(super) fn resolve_selection_app(
    config: &NoMistakesConfig,
    playwright_project: Option<&str>,
    rule_bound_names: BTreeSet<String>,
    app_names: &BTreeSet<&str>,
) -> Result<Option<String>> {
    let label = playwright_project.unwrap_or("<default>");

    if let Some(explicit) = playwright_project
        .and_then(|project| config.tests.playwright.apps.get(project))
        .and_then(|binding| binding.project.clone())
    {
        if !app_names.is_empty() && !app_names.contains(explicit.as_str()) {
            return Err(anyhow!(
                "`tests.playwright.apps.{label}.project` names `{explicit}`, which is not a \
                 configured `type: nextjs` project ({}).\nFix one of:\n  \
                 - correct the name\n  \
                 - add `projects.{explicit}` with `type: nextjs`",
                app_names.iter().copied().collect::<Vec<_>>().join(", "),
            ));
        }
        return Ok(Some(explicit));
    }

    if playwright_project
        .and_then(|project| config.tests.playwright.apps.get(project))
        .is_some_and(binding_is_fully_explicit)
    {
        return Ok(None);
    }

    if !rule_bound_names.is_empty() {
        return named_rule_app(label, rule_bound_names, app_names);
    }

    match app_names.len() {
        0 => Ok(None),
        1 => Ok(app_names.iter().next().map(|name| (*name).to_string())),
        _ => Err(anyhow!(
            "cannot resolve which frontend app Playwright project `{label}` exercises: {} \
             `type: nextjs` projects are configured ({}).\nFix one of:\n  \
             - add `projects: [<nextjs project>]` to the rule application\n  \
             - set `tests.playwright.apps.{label}.project`",
            app_names.len(),
            app_names.iter().copied().collect::<Vec<_>>().join(", "),
        )),
    }
}

fn binding_is_fully_explicit(binding: &crate::config::v2::schema::PlaywrightAppBinding) -> bool {
    binding.project.is_none()
        && binding.frontend_root.is_some()
        && !binding.selector_roots.is_empty()
        && !binding.rewrites.is_empty()
}

fn named_rule_app(
    label: &str,
    rule_bound_names: BTreeSet<String>,
    app_names: &BTreeSet<&str>,
) -> Result<Option<String>> {
    let named: BTreeSet<&String> = rule_bound_names
        .iter()
        .filter(|name| app_names.contains(name.as_str()))
        .collect();
    match named.len() {
        0 => Err(anyhow!(
            "rule `projects:` for Playwright project `{label}` ({}) does not name any \
             configured `type: nextjs` project; playwright-coverage, \
             playwright-unique-test-ids, playwright-unique-html-ids, and \
             playwright-prefer-test-id-locators need a frontend app to analyze.",
            rule_bound_names
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        )),
        1 => Ok(named.into_iter().next().cloned()),
        _ => Err(anyhow!(
            "Playwright project `{label}` is bound to more than one frontend app ({}) across \
             its rule applications; a Playwright project can exercise at most one app. Set \
             `tests.playwright.apps.{label}.project` to pick one explicitly.",
            named
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )),
    }
}
