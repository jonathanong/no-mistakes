use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentFacts {
    pub name: String,
    pub file: String,
    pub environment: Environment,
    pub has_state: bool,
    pub has_props: bool,
    pub passes_props: bool,
    pub uses_memo: bool,
    pub uses_context_provider: bool,
    pub uses_suspense: bool,
    pub fetches: Vec<FetchCall>,
    pub dependencies: Vec<String>,
    pub children: Vec<ComponentRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherited_from_children: Option<AggregatedFacts>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedFacts {
    pub has_state: bool,
    pub has_props: bool,
    pub passes_props: bool,
    pub uses_memo: bool,
    pub uses_context_provider: bool,
    pub uses_suspense: bool,
    pub has_fetch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchCall {
    pub file: String,
    pub exported_name: Option<String>,
    pub shape: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRef {
    pub name: String,
    pub file: String,
}

/// The component a `react usages` query was run against.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsagesTarget {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

/// A single JSX render site of the target component.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Callsite {
    pub file: String,
    pub line: usize,
    /// The exported name rendered at this site (the target symbol, or `default`).
    pub component: String,
    /// Named props passed at this callsite, in source order.
    pub props: Vec<String>,
    /// True when the callsite spreads props (`{...rest}`), so `props` may be partial.
    pub has_spread: bool,
}

/// Structural impact map for a component: where it is rendered, which stories and
/// tests import it, and the prop type names it exports. Optional sections are
/// `None` when not requested via `--include`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsagesReport {
    pub target: UsagesTarget,
    pub callsites: Vec<Callsite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prop_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Server,
    Client,
    Shared,
    Unknown,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Server => write!(f, "server"),
            Environment::Client => write!(f, "client"),
            Environment::Shared => write!(f, "shared"),
            Environment::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    pub component: String,
    pub file: String,
    pub rule: String,
    pub detail: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RootConfig {
    pub(crate) frontend_root: Option<String>,
    pub(crate) assert_no_fetch: Option<bool>,
    pub(crate) react_traits: Option<FileConfig>,
}

impl RootConfig {
    pub(crate) fn into_file_config(self) -> FileConfig {
        let mut config = FileConfig {
            frontend_root: self.frontend_root,
            assert_no_fetch: self.assert_no_fetch,
        };

        if let Some(react_traits) = self.react_traits {
            if react_traits.frontend_root.is_some() {
                config.frontend_root = react_traits.frontend_root;
            }
            if react_traits.assert_no_fetch.is_some() {
                config.assert_no_fetch = react_traits.assert_no_fetch;
            }
        }

        config
    }
}

#[derive(Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct FileConfig {
    pub(crate) frontend_root: Option<String>,
    pub(crate) assert_no_fetch: Option<bool>,
}
