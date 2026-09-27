use serde::{Deserialize, Serialize};

/// Opt-in knobs for `tests impact`. Empty lists preserve existing behavior.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ImpactConfig {
    /// Stub/mock test globs always surfaced for transitive import dependencies,
    /// even when a configured suite excludes those files.
    pub always_include_tests: Vec<String>,
    /// Registry file globs that emit a hint to verify a changed import's entry.
    pub registries: Vec<String>,
}
