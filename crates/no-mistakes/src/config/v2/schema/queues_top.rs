use super::queues::{deserialize_factories, serialize_factories};
use super::queues_job_state::{
    deserialize_job_state_as_record, serialize_job_state_as_record, JobStateAsRecordConfig,
};
use super::queues_retention::{
    deserialize_unbounded_retention, serialize_unbounded_retention, UnboundedJobRetentionConfig,
};
use super::QueueFactoryRoles;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
