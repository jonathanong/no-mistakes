use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlComment {
    pub object_type: String,
    pub name: PostgresSqlName,
    /// None means no signature was written; Some([]) is an explicit ().
    pub arguments: Option<Vec<PostgresSqlFunctionArgument>>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlAlterIndex {
    pub name: PostgresSqlName,
    pub if_exists: bool,
    pub operation: PostgresSqlAlterIndexOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlAlterIndexOperation {
    AttachPartition { partition: PostgresSqlName },
    Rename { name: PostgresSqlName },
}
