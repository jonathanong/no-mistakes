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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LockInventoryBaselineUnanalyzed {
    pub file: String,
    pub reason: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LockInventoryUnanalyzedDelta {
    pub file: String,
    pub reason: String,
    pub baseline_count: usize,
    pub current_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LockInventoryBaseline {
    pub version: u32,
    pub sites: Vec<LockInventoryBaselineSite>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unanalyzed: Vec<LockInventoryBaselineUnanalyzed>,
}

impl Default for LockInventoryBaseline {
    fn default() -> Self {
        Self {
            version: 1,
            sites: Vec::new(),
            unanalyzed: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockInventoryReport {
    pub sites: Vec<LockInventorySite>,
    pub counts: BTreeMap<String, usize>,
    pub unanalyzed: Vec<UnanalyzedLockSite>,
    pub unanalyzed_by_reason: BTreeMap<String, usize>,
    pub unanalyzed_by_directory: BTreeMap<String, usize>,
    pub baseline_file: String,
    pub baseline_unanalyzed: bool,
    pub added: Vec<LockInventorySite>,
    pub removed: Vec<LockInventoryBaselineSite>,
    pub unanalyzed_added: Vec<LockInventoryUnanalyzedDelta>,
    pub unanalyzed_removed: Vec<LockInventoryUnanalyzedDelta>,
    pub baseline_updated: bool,
}

impl LockInventoryReport {
    pub fn is_clean(&self) -> bool {
        if self.baseline_unanalyzed {
            self.added.is_empty()
                && self.removed.is_empty()
                && self.unanalyzed_added.is_empty()
                && self.unanalyzed_removed.is_empty()
        } else {
            self.added.is_empty()
                && self.removed.is_empty()
                && self.unanalyzed.is_empty()
                && self.unanalyzed_removed.is_empty()
        }
    }

    pub fn unanalyzed_baseline_entries(&self) -> Vec<LockInventoryBaselineUnanalyzed> {
        let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for gap in &self.unanalyzed {
            *counts
                .entry((gap.file.as_str(), gap.reason.as_str()))
                .or_insert(0) += 1;
        }
        counts
            .into_iter()
            .map(|((file, reason), count)| LockInventoryBaselineUnanalyzed {
                file: file.into(),
                reason: reason.into(),
                count,
            })
            .collect()
    }
}
