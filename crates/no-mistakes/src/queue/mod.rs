pub(crate) mod extract;
mod extract_helpers;
mod extract_model;
mod extract_record;
mod extract_visitor;
mod graph;
mod graph_build;
mod graph_entities;
mod graph_model;
mod graph_related;
mod graph_resolution;
mod resolver;
mod source;
mod types;

pub use graph::{
    analyze_project, analyze_project_indexed, analyze_project_with_facts,
    analyze_project_with_prepared_facts,
    analyze_project_with_prepared_facts_and_catalog_and_session,
    analyze_project_with_prepared_facts_indexed,
    analyze_project_with_prepared_facts_indexed_and_catalog_and_session, RelatedDirection,
};
pub use graph_model::{CheckFinding, PreparedProjectReport, ProjectReport};
pub use graph_related::related;
pub use source::{discover_source_files, relative_string};
pub use types::{Diagnostic, Edge, EdgeKind, QueueJobNode, QueueProducer, QueueWorker, Severity};

#[cfg(test)]
mod tests;
