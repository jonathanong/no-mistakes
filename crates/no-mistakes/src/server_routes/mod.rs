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
pub(crate) use extract::{has_server_route_shape, is_client_http_module};
pub(crate) use graph::route_defs_from_files;
pub use graph::{
    analyze_project, analyze_project_with_prepared, prepare_analysis,
    prepare_analysis_with_shared_facts, PreparedServerAnalysis, RelatedDirection,
};
pub use model::ProjectReport;
pub use related::related;
pub use types::{Diagnostic, Edge, EdgeKind, Framework, ServerRoute, Severity, Summary};

#[cfg(test)]
mod tests;
