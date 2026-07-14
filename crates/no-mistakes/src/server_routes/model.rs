use crate::edge_index::{EdgeIndex, NodeAliases};
use crate::server_routes::types::{EdgeKind, Framework, RelationshipNode};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Binding {
    pub framework: Framework,
    pub prefixes: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct RouteSite {
    pub file: PathBuf,
    pub line: usize,
    pub binding: String,
    pub method: String,
    pub raw_path: String,
    pub path: String,
    pub query_params: Vec<String>,
    pub framework: Framework,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct MountSite {
    pub parent: String,
    pub child: String,
    pub prefix: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ImportBinding {
    pub local: String,
    pub imported: String,
    pub source: String,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FileFacts {
    pub bindings: HashMap<String, Binding>,
    pub exports: HashMap<String, String>,
    pub imports: Vec<ImportBinding>,
    pub routes: Vec<RouteSite>,
    pub mounts: Vec<MountSite>,
    pub diagnostics: Vec<(usize, String)>,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct PreparedProjectReport {
    pub(crate) root: PathBuf,
    pub(crate) report: ProjectReport,
    pub(crate) index: EdgeIndex<RelationshipNode, EdgeKind>,
    pub(crate) nodes_by_name: HashMap<String, Vec<RelationshipNode>>,
    pub(crate) aliases: NodeAliases<RelationshipNode>,
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
