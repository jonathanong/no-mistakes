extern crate self as no_mistakes;

ctor::declarative::ctor! {
    // SAFETY: The constructor only installs an immutable table of function pointers.
    #[ctor(unsafe)]
    fn register_react_analysis() {
        codebase::ts_source::facts::react_analysis::register(
            &react_analysis_registration::REACT_HOOKS,
        );
    }
}

pub mod ast;
#[cfg(feature = "test-instrumentation")]
#[doc(hidden)]
pub mod benchmark_support;
pub mod changes;
#[doc(hidden)]
pub mod check_discovery;
#[doc(hidden)]
pub mod check_parallel;
#[doc(hidden)]
pub mod check_runner;
#[doc(hidden)]
pub mod check_tasks;
pub mod ci;
pub mod cli;
pub mod codebase;
pub mod config;
pub mod data_pw_query;
pub mod diagnostics;
pub(crate) mod edge_index;
mod effect_provenance;
pub mod effects_query;
pub mod fetch;
#[doc(hidden)]
pub mod fetches;
pub mod flow_query;
#[doc(hidden)]
pub mod fx;
pub mod impacted_checks;
pub mod import_shape;
pub mod imports;
pub mod integration_tests;
pub use no_mistakes_invocation as invocation;
pub mod mermaid_validation;
pub mod perf_trace;
pub mod playwright;
pub mod queue;
mod queue_analysis_hooks;
pub mod react_traits;
pub mod registry_extension_query;
pub mod routes;
pub mod rsc_callers_query;
pub mod server_routes;
pub mod swift_api;
pub mod terraform_api;
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub mod test_support;
mod testing_analysis_registration;
#[doc(hidden)]
pub mod tests;
pub use ci::{run as ci_run, CiArgs};
pub use impacted_checks::{run as impacted_checks_run, ImpactedChecksArgs};
pub use tests::audit::{
    audit_test_selection, TestAuditExecutionEvidence, TestAuditObservation,
    TestAuditObservationsArtifact, TestAuditPlanArtifact, TestAuditProvenance, TestAuditReport,
    TestAuditSelectionEvidence, TestAuditSymbol,
};
pub use tests::{
    run as tests_run, Confidence, GroupedExecutionTarget, ImpactEdgeDetail, ImpactReason,
    ResourceCallSite, SelectedTest, TestPlan, TestPlanGroupResult, TestsArgs, Warning,
};

pub mod postgres_catalog;
pub mod postgres_lock_inventory;

mod package_analysis_registration;
mod postgres_analysis_registration;
mod react_analysis_registration;
mod ts_analysis_hooks;

ctor::declarative::ctor! {
    // SAFETY: Registration stores a reference to an immutable static hooks table through OnceLock.
    #[ctor(unsafe)]
    fn register_ts_analysis() {
        ts_analysis_hooks::ensure_registered();
    }
}

ctor::declarative::ctor! {
    // SAFETY: Registration stores a reference to a static table through OnceLock.
    #[ctor(unsafe)]
    fn register_queue_analysis() {
        queue_analysis_hooks::register(&queue::analysis_hooks::HOOKS);
    }
}

ctor::declarative::ctor! {
    // SAFETY: Registration stores a reference to a static table through OnceLock.
    #[ctor(unsafe)]
    fn register_playwright_analysis() {
        codebase::playwright_hooks::register(&playwright::prepared_facts::HOOKS);
    }
}

static RULE_ANALYSIS_HOOKS: codebase::rule_analysis_hooks::Hooks =
    codebase::rule_analysis_hooks::Hooks {
        prepare_requested_rule_config:
            codebase::rules::analysis_policy::prepare_requested_rule_config,
    };

ctor::declarative::ctor! {
    // SAFETY: Registration stores a reference to a static table through OnceLock.
    #[ctor(unsafe)]
    fn register_rule_analysis() {
        codebase::rule_analysis_hooks::register(&RULE_ANALYSIS_HOOKS);
    }
}
