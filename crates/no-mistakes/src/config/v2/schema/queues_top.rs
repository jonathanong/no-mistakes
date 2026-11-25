use super::queues::{deserialize_factories, serialize_factories};
use super::queues_job_state::{
    deserialize_job_state_as_record, serialize_job_state_as_record, JobStateAsRecordConfig,
};
use super::queues_retention::{
    deserialize_unbounded_retention, serialize_unbounded_retention, UnboundedJobRetentionConfig,
};
use super::QueueFactoryRoles;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) fn allow_paths_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    crate::queue::allow_paths_test_hook::note_build();
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = GlobBuilder::new(pattern).literal_separator(false).build() {
            builder.add(glob);
        }
    }
    builder.build().ok()
}

pub(super) fn validate_allow_paths(key: &str, patterns: &[String]) -> Result<(), String> {
    for pattern in patterns {
        GlobBuilder::new(pattern)
            .literal_separator(false)
            .build()
            .map_err(|error| format!("invalid {key} glob `{pattern}`: {error}"))?;
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueuesTopLevelConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_factories",
        serialize_with = "serialize_factories"
    )]
    pub factories: QueueFactoryRoles,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_job_options: Vec<String>,
    /// A present key replaces `required_job_options` for that queue, including `[]`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub required_job_options_by_queue: BTreeMap<String, Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_unbounded_retention",
        serialize_with = "serialize_unbounded_retention"
    )]
    pub unbounded_job_retention: Option<UnboundedJobRetentionConfig>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_job_state_as_record",
        serialize_with = "serialize_job_state_as_record"
    )]
    pub job_state_as_record: Option<JobStateAsRecordConfig>,
}
