use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct TestRef {
    pub file: Arc<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<String>>,
    #[serde(skip_serializing_if = "super::is_arc_empty", default)]
    pub describe_path: Arc<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<RouteCoverageAttribution>,
}

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct RouteCoverageAttribution {
    pub framework: crate::config::v2::schema::RouteCoverageFramework,
    pub project: String,
    pub declaration_file: String,
}
