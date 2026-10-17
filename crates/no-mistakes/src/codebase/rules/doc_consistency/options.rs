use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SubstringSpec {
    pub(crate) file: String,
    pub(crate) substring: String,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct BannedPattern {
    pub(crate) files: Vec<String>,
    pub(crate) pattern: String,
    pub(crate) message: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) required_files: Vec<String>,
    pub(crate) required_heading: Option<String>,
    pub(crate) required_substrings: Vec<SubstringSpec>,
    pub(crate) banned_substrings: Vec<String>,
    pub(crate) banned_patterns: Vec<BannedPattern>,
}
