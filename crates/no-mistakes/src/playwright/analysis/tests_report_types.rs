use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestsReport {
    pub tests: Vec<TestEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestEntry {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub describe_path: Vec<String>,
    pub test_ids: Vec<String>,
    pub html_ids: Vec<String>,
    pub routes: Vec<String>,
    pub fetch_apis: Vec<String>,
    pub locator_texts: Vec<String>,
}
