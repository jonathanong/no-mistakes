mod contracts;
pub(crate) mod extract;
mod graph;
pub(crate) mod model;
mod mounts;
mod normalize;
mod related;
mod source;
mod types;

pub use contracts::{analyze_contracts, analyze_contracts_with_prepared, ServerContractsReport};
pub(crate) use extract::{has_server_route_shape_from_program, is_client_http_module};
pub use graph::{
    analyze_project, analyze_project_indexed, analyze_project_with_prepared,
    analyze_project_with_prepared_indexed, prepare_analysis, prepare_analysis_with_shared_facts,
    PreparedServerAnalysis, RelatedDirection,
};
pub(crate) use graph::{
    configure_fact_context, route_defs_from_files, route_defs_from_prepared_facts,
};
pub use model::{PreparedProjectReport, ProjectReport};
pub use related::related;
pub use types::{Diagnostic, Edge, EdgeKind, Framework, ServerRoute, Severity, Summary};

#[cfg(test)]
mod tests;
