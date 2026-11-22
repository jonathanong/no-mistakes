use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LockInventorySite {
    pub key: String,
    pub file: String,
    pub line: usize,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnanalyzedLockSite {
    pub file: String,
    pub line: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LockInventoryBaselineSite {
    pub key: String,
    pub file: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LockInventoryBaseline {
    pub version: u32,
    pub sites: Vec<LockInventoryBaselineSite>,
}

impl Default for LockInventoryBaseline {
    fn default() -> Self {
        Self {
            version: 1,
            sites: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockInventoryReport {
    pub sites: Vec<LockInventorySite>,
    pub counts: BTreeMap<String, usize>,
    pub unanalyzed: Vec<UnanalyzedLockSite>,
    pub baseline_file: String,
    pub added: Vec<LockInventorySite>,
    pub removed: Vec<LockInventoryBaselineSite>,
    pub baseline_updated: bool,
}
