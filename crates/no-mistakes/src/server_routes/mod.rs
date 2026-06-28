mod contracts;
pub(crate) mod extract;
mod graph;
pub(crate) mod model;
mod mounts;
mod normalize;
mod related;
mod source;
mod types;

pub use contracts::{analyze_contracts, ServerContractsReport};
pub(crate) use extract::{has_server_route_shape, is_client_http_module};
pub(crate) use graph::route_defs_from_files;
pub use graph::{analyze_project, RelatedDirection};
pub use model::ProjectReport;
pub use related::related;
pub use types::{Diagnostic, Edge, EdgeKind, Framework, ServerRoute, Severity, Summary};

#[cfg(test)]
mod tests;
