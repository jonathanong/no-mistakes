use super::{CheckFileFacts, PlaywrightSettingsKey};
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::selectors::{AppSelector, AppSelectorValue};
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

type AppSelectorOccurrencesCache =
    DashMap<(PlaywrightSettingsKey, bool), Result<Arc<Vec<AppSelector>>, String>>;
type AppTextTargetsCache = DashMap<PlaywrightSettingsKey, Result<Arc<Vec<AppTextTarget>>, String>>;

pub(crate) fn playwright_aggregate_facts(
    facts: &HashMap<PathBuf, CheckFileFacts>,
) -> (AppSelectorOccurrencesCache, AppTextTargetsCache) {
    let static_exports = facts
        .iter()
        .filter_map(|(path, facts)| {
            facts
                .playwright_static_exports
                .as_ref()
                .map(|exports| (path.clone(), exports.clone()))
        })
        .collect::<HashMap<_, _>>();
    let mut selectors_by_settings =
        HashMap::<(PlaywrightSettingsKey, bool), Vec<AppSelector>>::new();
    let mut text_targets_by_settings = HashMap::<PlaywrightSettingsKey, Vec<AppTextTarget>>::new();
    for facts in facts.values() {
        for (settings, targets) in &facts.playwright_app_text_targets {
            text_targets_by_settings
                .entry(settings.clone())
                .or_default()
                .extend(targets.iter().cloned());
        }
        for (settings, source_selectors) in &facts.playwright_app_selectors {
            let selectors = selectors_by_settings.entry(settings.clone()).or_default();
            for selector in source_selectors {
                let AppSelectorValue::Exact(selector_value) = &selector.value else {
                    selectors.push(selector.clone());
                    continue;
                };
                match crate::playwright::selectors::resolve_deferred_import(
                    selector_value,
                    &static_exports,
                ) {
                    Some(values) => selectors.extend(values.iter().map(|value| AppSelector {
                        file: selector.file.clone(),
                        attribute: selector.attribute.clone(),
                        value: AppSelectorValue::Exact(value.clone()),
                    })),
                    None => selectors.push(selector.clone()),
                }
            }
        }
    }
    let selector_cache = DashMap::new();
    for ((settings, scan_html_ids), mut selectors) in selectors_by_settings {
        selectors.sort();
        selector_cache.insert(
            (settings.clone(), scan_html_ids),
            Ok(Arc::new(selectors.clone())),
        );
        if scan_html_ids {
            selectors.retain(|selector| {
                selector.attribute != crate::playwright::selectors::HTML_ID_ATTRIBUTE
            });
            selector_cache.insert((settings, false), Ok(Arc::new(selectors)));
        }
    }
    let text_cache = DashMap::new();
    for (settings, mut targets) in text_targets_by_settings {
        targets.sort();
        targets.dedup();
        text_cache.insert(settings, Ok(Arc::new(targets)));
    }
    (selector_cache, text_cache)
}
