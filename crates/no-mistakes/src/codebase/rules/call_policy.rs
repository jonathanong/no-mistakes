//! Shared call-graph root selection, argument scope, and call-site helpers.

pub(crate) mod argument_scope;
pub(crate) mod root;
mod root_files;
pub(crate) mod roots;
mod shared;

pub(crate) use argument_scope::{argument_facts_covered, requires_arguments};
pub use argument_scope::{configure_argument_facts, ArgumentFactScope};
pub(crate) use shared::{expand_roots, SharedRoots};

use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::ArgumentValues;
use crate::config::v2::NoMistakesConfig;

pub(crate) const FORBIDDEN_CALLS_RULE_ID: &str = "forbidden-calls";
pub(crate) const TEST_NO_LIVE_REPOSITORY_SCAN_RULE_ID: &str = "test-no-live-repository-scan";

pub(crate) fn site_source_node(site: &ResolvedCallSite) -> NodeId {
    site.caller.as_deref().map_or_else(
        || NodeId::file(&site.file),
        |symbol| {
            site.caller_id.map_or_else(
                || NodeId::symbol(&site.file, symbol),
                |id| NodeId::callable(&site.file, symbol.to_string(), id),
            )
        },
    )
}

pub(crate) fn call_argument_values(facts: &TsFileFacts, offset: u32) -> Option<&ArgumentValues> {
    // Recovered syntax and failed semantic ownership cannot prove a predicate.
    if facts.operational_error.is_some() || facts.parse_error.is_some() || facts.fatal_parse_error {
        return None;
    }
    let module = facts.module_bindings.as_deref()?;
    // The fact owner withholds argument proofs for binding-unsafe diagnostics;
    // an unrelated opaque module load does not invalidate these recorded facts.
    if !module
        .diagnostics
        .iter()
        .all(crate::codebase::ts_source::module_facts::argument_safe_diagnostic)
    {
        return None;
    }
    module.call_arguments.as_deref()?.get(&offset)
}

/// True when a call-policy application names a `vitest:` collection root.
pub fn vitest_roots_configured(config: &NoMistakesConfig) -> bool {
    catalog_root_configured(config, "vitest")
        || (crate::codebase::testing_analysis::hooks().vitest_roots_configured)(config)
}

/// True when a forbidden-calls application names a `playwright:` collection root.
pub fn playwright_roots_configured(config: &NoMistakesConfig) -> bool {
    catalog_root_configured(config, "playwright")
}

fn catalog_root_configured(config: &NoMistakesConfig, key: &str) -> bool {
    config
        .rule_applications(FORBIDDEN_CALLS_RULE_ID)
        .iter()
        .any(|application| {
            application
                .options
                .get("roots")
                .and_then(serde_yaml::Value::as_sequence)
                .is_some_and(|roots| roots.iter().any(|root| root.get(key).is_some()))
        })
}

/// Prepare runner catalogs for declared call roots and integration-suite ownership.
#[doc(hidden)]
pub fn prepare_root_catalogs(
    root: &std::path::Path,
    config: &NoMistakesConfig,
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
        (playwright_roots_configured(config)
            || config
                .tests
                .playwright
                .projects
                .values()
                .any(|policy| !policy.integration_suites.is_empty() && policy.include.is_empty()))
        .then(|| {
            super::prepare_playwright_project_catalog(root, config, snapshot, tsconfig_catalog)
        }),
    )
}
