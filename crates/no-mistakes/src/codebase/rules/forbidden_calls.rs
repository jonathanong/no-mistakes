//! Binding-aware call policy projected from the prepared canonical graph.

mod arguments;
pub(in crate::codebase::rules) mod config;
mod findings;
mod selection;

pub const RULE_ID: &str = "forbidden-calls";

pub(crate) use super::call_policy::argument_facts_covered;
#[doc(hidden)]
pub use super::call_policy::prepare_root_catalogs;
pub use super::call_policy::{
    configure_argument_facts, playwright_roots_configured, vitest_roots_configured,
    ArgumentFactScope,
};
pub(crate) use selection::{check_with_graph, graph_plan};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

/// Compatibility probe for callers that only need to know whether any
/// forbidden-calls application requests argument facts. Prepared runners use
/// [`configure_argument_facts`] to retain each application's file scope.
/// Retained for public Rust API compatibility; its only internal callers are tests.
pub fn argument_facts_required(config: &crate::config::v2::NoMistakesConfig) -> bool {
    config
        .rule_applications(RULE_ID)
        .iter()
        .any(|application| super::call_policy::requires_arguments(application))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod argument_rule_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod catalog_root_tests;
