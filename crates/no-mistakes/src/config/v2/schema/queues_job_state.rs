use super::queues_top::{allow_paths_globset, validate_allow_paths};
use globset::GlobSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JobStateAsRecordConfig {
    pub allow_paths: Vec<String>,
}

impl JobStateAsRecordConfig {
    pub fn allow_globset(&self) -> Option<GlobSet> {
        allow_paths_globset(&self.allow_paths)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum JobStateInput {
    Enabled(bool),
    Options(JobStateOptionsInput),
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JobStateOptionsInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    allow_paths: Vec<String>,
}

pub fn deserialize_job_state_as_record<'de, D>(
    deserializer: D,
) -> Result<Option<JobStateAsRecordConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(input) = Option::<JobStateInput>::deserialize(deserializer)? else {
        return Ok(None);
    };
    match input {
        JobStateInput::Enabled(false) => Ok(None),
        JobStateInput::Enabled(true) => Ok(Some(JobStateAsRecordConfig::default())),
        JobStateInput::Options(options) => {
            validate_allow_paths("queues.jobStateAsRecord.allowPaths", &options.allow_paths)
                .map_err(serde::de::Error::custom)?;
            Ok(Some(JobStateAsRecordConfig {
                allow_paths: options.allow_paths,
            }))
        }
    }
}

pub fn serialize_job_state_as_record<S>(
    value: &Option<JobStateAsRecordConfig>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let Some(config) = value else {
        return serializer.serialize_none();
    };
    if config.allow_paths.is_empty() {
        return true.serialize(serializer);
    }
    JobStateOptionsInput {
        allow_paths: config.allow_paths.clone(),
    }
    .serialize(serializer)
}
