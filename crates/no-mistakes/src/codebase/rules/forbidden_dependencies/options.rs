use crate::codebase::dependencies::RelationshipArg;

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct Options {
    pub roots: Vec<String>,
    pub exclude_roots: Vec<String>,
    pub forbidden_modules: Vec<String>,
    pub forbidden_files: Vec<String>,
    pub relationships: Vec<RelationshipArg>,
    pub exceptions: Vec<Exception>,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct Exception {
    pub roots: Vec<String>,
    pub allowed_files: Vec<String>,
}
