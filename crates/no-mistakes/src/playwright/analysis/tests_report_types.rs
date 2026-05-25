use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestsReport {
    pub(crate) tests: Vec<TestEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestEntry {
    pub(crate) file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) describe_path: Vec<String>,
    pub(crate) test_ids: Vec<String>,
    pub(crate) html_ids: Vec<String>,
    pub(crate) routes: Vec<String>,
    pub(crate) fetch_apis: Vec<String>,
    pub(crate) locator_texts: Vec<String>,
}
