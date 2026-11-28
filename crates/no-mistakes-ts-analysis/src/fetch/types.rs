use serde::Serialize;

#[derive(Serialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct FetchOccurrence {
    pub path: String,
    pub raw_path: String,
    pub method: String,
    pub file: String,
    pub line: usize,
    pub side: FetchSide,
    #[serde(rename = "rsc")]
    pub rsc: bool,
    pub cached: bool,
    pub cache_kind: CacheKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_function: Option<String>,
    pub dynamic: bool,
    pub unsupported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,
    pub conditional: bool,
    pub in_promise_all: bool,
    pub error_handled: bool,
    pub source_type: SourceType,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UrlExtraction {
    pub path: String,
    pub raw_path: String,
    pub is_dynamic: bool,
    pub is_unsupported: bool,
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
    Page,
    Layout,
    Loading,
    Error,
    Template,
    Route,
    Module,
}

impl SourceType {
    pub fn from_file_stem(path: &str) -> Self {
        let stem = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        match stem {
            "page" => Self::Page,
            "layout" => Self::Layout,
            "loading" => Self::Loading,
            "error" | "not-found" => Self::Error,
            "template" => Self::Template,
            "route" => Self::Route,
            _ => Self::Module,
        }
    }
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FetchSide {
    Client,
    Server,
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum CacheKind {
    None,
    FetchCache,
    FetchNextRevalidate,
    FetchNextTags,
    ReactCache,
    Cache,
    UnstableCache,
}
