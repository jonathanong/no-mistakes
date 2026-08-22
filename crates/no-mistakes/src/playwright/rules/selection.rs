use super::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::config::v2::{FrontendApp, NoMistakesConfig};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

mod app;
use app::resolve_selection_app;

#[derive(Clone, Debug)]
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
    pub(super) cover_routes: bool,
    pub(super) cover_selectors: bool,
}

impl Default for RuleSelection {
    fn default() -> Self {
        Self {
            playwright_project: None,
            app: None,
            coverage: false,
            unique_test_ids: false,
            unique_html_ids: false,
            prefer_test_id_locators: false,
            cover_routes: true,
            cover_selectors: true,
        }
    }
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
        let app = resolve_selection_app(
            config,
            playwright_project.as_deref(),
            rule_bound_names,
            &app_names,
        );
        selection.app = app?;
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
        let projects: BTreeSet<Option<String>> = if !rule.tests.playwright.is_empty() {
            rule.tests
                .playwright
                .iter()
                .map(|project| Some(project.clone()))
                .collect()
        } else if !config.tests.playwright.apps.is_empty() {
            config
                .tests
                .playwright
                .apps
                .keys()
                .cloned()
                .map(Some)
                .collect()
        } else {
            [None].into_iter().collect()
        };
        for project in projects {
            let selection = by_project
                .entry(project.clone())
                .or_insert_with(|| RuleSelection {
                    playwright_project: project.clone(),
                    cover_routes: config.tests.playwright.coverage.routes,
                    cover_selectors: config.tests.playwright.coverage.selectors,
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

#[cfg(test)]
mod tests;
