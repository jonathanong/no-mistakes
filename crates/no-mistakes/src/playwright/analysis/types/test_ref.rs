use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestRef {
    pub(crate) file: Arc<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<Arc<String>>,
    #[serde(skip_serializing_if = "super::is_arc_empty", default)]
    pub(crate) describe_path: Arc<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) attribution: Option<RouteCoverageAttribution>,
}

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RouteCoverageAttribution {
    pub(crate) framework: crate::config::v2::schema::RouteCoverageFramework,
    pub(crate) project: String,
    pub(crate) declaration_file: String,
}
