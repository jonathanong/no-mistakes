pub(crate) mod analysis_hooks;
mod cycles;
mod enqueue_wrapper;
pub(crate) mod extract;
mod extract_debounce;
mod extract_factory;
mod extract_helpers;
mod extract_job_id;
mod extract_model;
mod extract_prefilter;
mod extract_priority;
mod extract_processor;
mod extract_record;
mod extract_visitor;
mod extract_worker_rate_limit;
mod factory_resolve;
mod globs;
mod graph;
mod graph_build;
mod graph_entities;
mod graph_model;
mod graph_related;
mod graph_resolution;
mod job_options;
mod job_state;
mod lang;
mod number_scope;
mod object_bindings;
mod outcomes;
mod priority_mix;
mod rate_limit;
mod resolver;
mod source;
mod test_scope;
mod traversal;
mod types;
mod worker_rate_limit;

pub use graph::{
    analyze_project, analyze_project_indexed, analyze_project_with_config,
    analyze_project_with_facts, analyze_project_with_prepared_facts,
    analyze_project_with_prepared_facts_and_catalog_and_session,
    analyze_project_with_prepared_facts_indexed,
    analyze_project_with_prepared_facts_indexed_and_catalog_and_session, RelatedDirection,
};
pub use graph_model::{
    format_aggregate_warning, format_diagnostic, is_check_warning, CheckFinding, CheckReport,
    CheckSeverity, PreparedProjectReport, ProjectReport,
};
pub use graph_related::related;
pub use source::{discover_source_files, relative_string};
pub use types::{
    Diagnostic, DiagnosticKind, Edge, EdgeKind, QueueJobNode, QueueProducer, QueueWorker, Severity,
};
pub use worker_rate_limit::{
    is_worker_rate_limit_unresolved_diagnostic, WORKER_RATE_LIMIT_UNRESOLVED_MESSAGE,
};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
pub(crate) mod tests;
