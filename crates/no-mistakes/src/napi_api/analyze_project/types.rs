use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct AnalyzeProjectOptions {
    pub(super) root: Option<String>,
    pub(super) tsconfig: Option<String>,
    pub(super) config: Option<String>,
    #[serde(default)]
    pub(super) filters: Vec<String>,
    pub(super) reports: Vec<AnalyzeReportRequest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AnalyzeReportRequest {
    pub(super) id: Option<String>,
    #[serde(rename = "type")]
    pub(super) report_type: String,
    #[serde(flatten)]
    pub(super) options: Map<String, Value>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AnalyzeReportResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) id: Option<String>,
    #[serde(rename = "type")]
    pub(super) report_type: String,
    pub(super) result: Value,
}

#[derive(serde::Serialize)]
pub(super) struct AnalyzeProjectResult {
    pub(super) reports: Vec<AnalyzeReportResult>,
}
