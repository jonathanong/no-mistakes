#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureImpactReport {
    roots: Vec<String>,
    symbol: String,
    definition: SymbolLocation,
    exports: Vec<SymbolLocation>,
    production_callers: Vec<CallerEntry>,
    test_callers: Vec<CallerEntry>,
    suggested_tests: Vec<TestSuggestion>,
    warnings: Vec<ImpactWarning>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
struct SymbolLocation {
    file: String,
    symbol: String,
    line: u32,
    kind: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CallerEntry {
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
    depth: usize,
    via: Vec<&'static str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct TestSuggestion {
    file: String,
    depth: usize,
    via: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImpactWarning {
    r#type: &'static str,
    message: String,
}
