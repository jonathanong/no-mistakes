use crate::playwright::analysis::types::Edge;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SelectorHelperReference {
    pub(crate) test_file: Arc<String>,
    pub(crate) line: u32,
    pub(crate) call: String,
}

#[derive(Default)]
pub(crate) struct TestFileAnalysis {
    pub(crate) edges: Vec<Edge>,
    pub(crate) helper_references: Vec<SelectorHelperReferenceWithValue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SelectorHelperReferenceWithValue {
    pub(crate) attribute: String,
    pub(crate) value: String,
    pub(crate) reference: SelectorHelperReference,
}
