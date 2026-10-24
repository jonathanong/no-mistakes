//! Binding-aware call policy projected from the prepared canonical graph.

mod argument_scope;
mod arguments;
pub(crate) use argument_scope::argument_facts_covered;
pub use argument_scope::{configure_argument_facts, ArgumentFactScope};
mod config;
mod findings;
mod roots;
mod selection;

pub const RULE_ID: &str = "forbidden-calls";

pub(crate) use selection::{check_with_graph, graph_plan};

#[cfg(test)]
mod tests;

/// Compatibility probe for callers that only need to know whether any
/// forbidden-calls application requests argument facts. Prepared runners use
/// [`configure_argument_facts`] to retain each application's file scope.
/// Retained for public Rust API compatibility; its only internal callers are tests.
pub fn argument_facts_required(config: &crate::config::v2::NoMistakesConfig) -> bool {
    config
        .rule_applications(RULE_ID)
        .iter()
        .any(|application| argument_scope::requires_arguments(application))
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
mod argument_rule_tests;
#[cfg(test)]
mod catalog_root_tests;
