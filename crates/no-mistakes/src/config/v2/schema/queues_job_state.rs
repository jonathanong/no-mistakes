use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JobStateAsRecordConfig {
    pub allow_paths: Vec<String>,
}

impl JobStateAsRecordConfig {
    pub fn allow_globset(&self) -> Option<GlobSet> {
        if self.allow_paths.is_empty() {
            return None;
        }
        let mut builder = GlobSetBuilder::new();
        for pattern in &self.allow_paths {
            if let Ok(glob) = GlobBuilder::new(pattern).literal_separator(false).build() {
                builder.add(glob);
            }
        }
        builder.build().ok()
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
            for pattern in &options.allow_paths {
                GlobBuilder::new(pattern)
                    .literal_separator(false)
                    .build()
                    .map_err(|error| {
                        serde::de::Error::custom(format!(
                            "invalid queues.jobStateAsRecord.allowPaths glob `{pattern}`: {error}"
                        ))
                    })?;
            }
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
