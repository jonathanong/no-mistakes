#[cfg(test)]
pub(crate) use crate::codebase::ts_source::facts::react_analysis::server::{Binding, MountSite};
pub(crate) use crate::codebase::ts_source::facts::react_analysis::server::{
    FileFacts, ImportBinding, RouteSite,
};
use crate::edge_index::PreparedRelationshipIndex;
use crate::server_routes::types::{EdgeKind, RelationshipNode};
use serde::Serialize;

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct PreparedProjectReport {
    pub(crate) report: ProjectReport,
    pub(crate) relationships: PreparedRelationshipIndex<RelationshipNode, EdgeKind>,
}

impl PreparedProjectReport {
    pub fn report(&self) -> &ProjectReport {
        &self.report
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReport {
    pub summary: crate::server_routes::types::Summary,
    pub routes: Vec<crate::server_routes::types::ServerRoute>,
    pub edges: Vec<crate::server_routes::types::Edge>,
    pub diagnostics: Vec<crate::server_routes::types::Diagnostic>,
}
