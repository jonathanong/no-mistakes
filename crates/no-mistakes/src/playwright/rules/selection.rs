use super::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::config::v2::{FrontendApp, NoMistakesConfig};
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(super) struct RuleSelection {
    pub(super) playwright_project: Option<String>,
    /// The `.no-mistakes.yml` `projects:` key of the frontend app this
    /// selection exercises, or `None` when there are no configured/inferred
    /// frontend apps at all (settings resolution then falls back to the
    /// pre-#624 bare default). Resolved once here so every consumer of a
    /// selection (rule execution, fact-plan preparation) agrees on the same
    /// app instead of re-deriving it and risking disagreement.
    pub(super) app: Option<String>,
    pub(super) coverage: bool,
    pub(super) unique_test_ids: bool,
    pub(super) unique_html_ids: bool,
    pub(super) prefer_test_id_locators: bool,
}

/// Resolve one [`RuleSelection`] per distinct Playwright project named across
/// `playwright-coverage`, `playwright-unique-test-ids`,
/// `playwright-unique-html-ids`, and `playwright-prefer-test-id-locators`
/// rule applications, and — for each — which single frontend app it
/// exercises.
///
/// `apps` must be the complete, already-resolved frontend app set (see
/// [`crate::config::v2::frontend_apps`]) so every call site agrees on it;
/// this function does not re-derive it.
pub(super) fn rule_selections(
    config: &NoMistakesConfig,
    apps: &[FrontendApp],
) -> Result<Vec<RuleSelection>> {
    let mut by_project = BTreeMap::<Option<String>, RuleSelection>::new();
    let mut app_bindings = BTreeMap::<Option<String>, BTreeSet<String>>::new();
    add_rule_selections(
        config,
        PLAYWRIGHT_COVERAGE,
        |selection| selection.coverage = true,
        &mut by_project,
        &mut app_bindings,
    );
    add_rule_selections(
        config,
        PLAYWRIGHT_UNIQUE_TEST_IDS,
        |selection| selection.unique_test_ids = true,
        &mut by_project,
        &mut app_bindings,
    );
    add_rule_selections(
        config,
        PLAYWRIGHT_UNIQUE_HTML_IDS,
        |selection| selection.unique_html_ids = true,
        &mut by_project,
        &mut app_bindings,
    );
    add_rule_selections(
        config,
        PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
        |selection| selection.prefer_test_id_locators = true,
        &mut by_project,
        &mut app_bindings,
    );

    let app_names: BTreeSet<&str> = apps
        .iter()
        .filter_map(|app| app.project.as_deref())
        .collect();
    for (playwright_project, selection) in by_project.iter_mut() {
        let rule_bound_names = app_bindings.remove(playwright_project).unwrap_or_default();
        selection.app = resolve_selection_app(
            config,
            playwright_project.as_deref(),
            rule_bound_names,
            &app_names,
        )?;
    }
    Ok(by_project.into_values().collect())
}

fn add_rule_selections(
    config: &NoMistakesConfig,
    rule_id: &str,
    apply: impl Fn(&mut RuleSelection) + Copy,
    by_project: &mut BTreeMap<Option<String>, RuleSelection>,
    app_bindings: &mut BTreeMap<Option<String>, BTreeSet<String>>,
) {
    for rule in config.rule_applications(rule_id) {
        let projects: BTreeSet<Option<String>> = if rule.tests.playwright.is_empty() {
            [None].into_iter().collect()
        } else {
            rule.tests
                .playwright
                .iter()
                .map(|project| Some(project.clone()))
                .collect()
        };
        for project in projects {
            let selection = by_project
                .entry(project.clone())
                .or_insert_with(|| RuleSelection {
                    playwright_project: project.clone(),
                    ..RuleSelection::default()
                });
            apply(selection);
            if !rule.projects.is_empty() {
                app_bindings
                    .entry(project)
                    .or_default()
                    .extend(rule.projects.iter().cloned());
            }
        }
    }
}

/// Resolve which single frontend app a Playwright-project selection
/// exercises, in precedence order:
///
/// 1. `tests.playwright.apps.<playwright_project>.project` — fully explicit,
///    always wins outright (no ambiguity check against rule bindings).
/// 2. The frontend-app names collected from every contributing rule
///    application's `projects:` list. Exactly one match binds; zero matches
///    (the list named only non-frontend projects) or more than one match
///    (rules disagree, or one rule names two apps) is an error rather than a
///    silent guess.
/// 3. No rule named a project at all: the sole configured frontend app, or
///    `None` when there are zero apps (not ambiguous — nothing to choose
///    between), or an error when there is more than one.
fn resolve_selection_app(
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
        return Ok(Some(explicit));
    }

    if !rule_bound_names.is_empty() {
        let named: BTreeSet<&String> = rule_bound_names
            .iter()
            .filter(|name| app_names.contains(name.as_str()))
            .collect();
        return match named.len() {
            0 => Err(anyhow!(
                "rule `projects:` for Playwright project `{label}` ({}) does not name any \
                 configured `type: nextjs` project; playwright-coverage and \
                 playwright-unique-test-ids need a frontend app to analyze.",
                rule_bound_names
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
            )),
            1 => Ok(named.into_iter().next().cloned()),
            _ => {
                Err(anyhow!(
                "Playwright project `{label}` is bound to more than one frontend app ({}) across \
                 its rule applications; a Playwright project can exercise at most one app. Set \
                 `tests.playwright.apps.{label}.project` to pick one explicitly.",
                named.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
            ))
            }
        };
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

#[cfg(test)]
mod tests;
