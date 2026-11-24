use super::queues::{deserialize_factories, serialize_factories};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unbounded_job_retention: Option<UnboundedJobRetentionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_state_as_record: Option<JobStateAsRecordConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct UnboundedJobRetentionConfig {
    pub keys: Vec<String>,
    pub allow_paths: Vec<String>,
}

impl Default for UnboundedJobRetentionConfig {
    fn default() -> Self {
        Self {
            keys: vec!["removeOnComplete".into(), "removeOnFail".into()],
            allow_paths: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct JobStateAsRecordConfig {
    pub allow_paths: Vec<String>,
}

impl QueueFactoryRoles {
    #[doc(hidden)]
    pub fn with_analysis_policies(self, config: &super::NoMistakesConfig) -> Self {
        let mut roles = self.with_project_wait_scopes(config);
        roles.record_job_state = config.queues.job_state_as_record.is_some();
        roles
    }
}
