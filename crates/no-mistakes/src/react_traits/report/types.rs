use serde::{Deserialize, Serialize};

pub use crate::codebase::ts_source::facts::react_analysis::component::{
    AggregatedFacts, ComponentFacts, ComponentRef, Environment, FetchCall,
};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    pub component: String,
    pub file: String,
    pub rule: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReactSuppressionTarget {
    pub file: String,
    pub line: usize,
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
