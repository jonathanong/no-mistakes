//! Binding-aware call policy projected from the prepared canonical graph.

mod argument_demand;
mod arguments;
mod config;
mod findings;
mod roots;
mod selection;
mod source_eligibility;
mod source_only;

pub const RULE_ID: &str = "forbidden-calls";

pub(crate) use selection::{check_with_graph, graph_plan};

#[doc(hidden)]
pub fn source_only_eligible(config: &crate::config::v2::NoMistakesConfig) -> bool {
    source_eligibility::eligible(config)
}

#[cfg(test)]
mod tests;

/// Declares opt-in argument demand without parsing application options twice.
pub fn argument_facts_required(config: &crate::config::v2::NoMistakesConfig) -> bool {
    argument_projection(config).is_some()
}

#[doc(hidden)]
pub fn argument_projection(
    config: &crate::config::v2::NoMistakesConfig,
) -> Option<crate::codebase::ts_source::facts::ArgumentProjection> {
    argument_demand::projection(config)
}

/// True when a forbidden-calls application names a `vitest:` collection root.
pub fn vitest_roots_configured(config: &crate::config::v2::NoMistakesConfig) -> bool {
    catalog_root_configured(config, "vitest")
}

/// True when a forbidden-calls application names a `playwright:` collection root.
pub fn playwright_roots_configured(config: &crate::config::v2::NoMistakesConfig) -> bool {
    catalog_root_configured(config, "playwright")
}

fn catalog_root_configured(config: &crate::config::v2::NoMistakesConfig, key: &str) -> bool {
    config.rule_applications(RULE_ID).iter().any(|application| {
        application
            .options
            .get("roots")
            .and_then(serde_yaml::Value::as_sequence)
            .is_some_and(|roots| roots.iter().any(|root| root.get(key).is_some()))
    })
}

/// Prepare runner catalogs only for forbidden-calls root kinds that exist.
#[doc(hidden)]
pub fn prepare_root_catalogs(
    root: &std::path::Path,
    config: &crate::config::v2::NoMistakesConfig,
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> (
    Option<super::PreparedVitestProjectCatalog>,
    Option<super::PreparedPlaywrightProjectCatalog>,
) {
    (
        vitest_roots_configured(config).then(|| {
            super::prepare_vitest_project_catalog(root, config, snapshot, tsconfig_catalog)
        }),
        playwright_roots_configured(config).then(|| {
            super::prepare_playwright_project_catalog(root, config, snapshot, tsconfig_catalog)
        }),
    )
}

#[cfg(test)]
#[path = "forbidden_calls/tests/argument_projection.rs"]
mod argument_projection_tests;
#[cfg(test)]
mod argument_rule_tests;
#[cfg(test)]
mod catalog_root_tests;
