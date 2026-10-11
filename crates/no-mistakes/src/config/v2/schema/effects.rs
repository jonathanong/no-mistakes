use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One named effect family (e.g. `valkey`) for the `effects` query.
///
/// `categories` maps a category label (e.g. `cache`, `pubsub`) to the function
/// or constructor names that belong to it; `functions` is a flat list applied
/// when no category split is needed (reported as uncategorized).
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct EffectKindConfig {
    pub categories: BTreeMap<String, Vec<String>>,
    pub functions: Vec<String>,
    /// Binding-aware module export selectors, independent of local import names.
    pub targets: Vec<EffectTargetConfig>,
    /// Configured transaction-client sinks for per-item query diagnostics.
    pub transaction_functions: Vec<String>,
    /// Configured batch/pipeline builders which exempt their own call paths.
    pub batch_functions: Vec<String>,
}

/// An effect selected by its exact import specifier and exported member path.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectTargetConfig {
    pub module: String,
    pub export: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}
