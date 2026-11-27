use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleFinding {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Structured reasons the rule could not prove its result; empty for ordinary findings.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub causes: Vec<FindingCause>,
}

/// One uncertainty cause behind a finding: `kind` is `unresolved-dispatch`,
/// `traversal-limit`, or `unmodeled-generator`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingCause {
    pub kind: &'static str,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callee: Option<String>,
    pub detail: String,
}

pub fn sort_findings(findings: &mut Vec<RuleFinding>) {
    findings.sort();
    findings.dedup();
}

pub fn sort_by_file(findings: &mut [RuleFinding]) {
    findings.sort_by(|a, b| a.file.cmp(&b.file));
}

pub fn sort_by_file_and_message(findings: &mut [RuleFinding]) {
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
}
