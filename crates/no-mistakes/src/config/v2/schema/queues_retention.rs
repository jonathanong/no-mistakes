use super::queues_top::{allow_paths_globset, validate_allow_paths};
use globset::GlobSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum JobRetentionKey {
    RemoveOnComplete,
    RemoveOnFail,
}

impl JobRetentionKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RemoveOnComplete => "removeOnComplete",
            Self::RemoveOnFail => "removeOnFail",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnboundedJobRetentionConfig {
    pub require: Vec<JobRetentionKey>,
    pub allow_paths: Vec<String>,
}

impl UnboundedJobRetentionConfig {
    pub fn all() -> Self {
        Self {
            require: vec![
                JobRetentionKey::RemoveOnComplete,
                JobRetentionKey::RemoveOnFail,
            ],
            allow_paths: Vec::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.require.is_empty()
    }

    pub fn allow_globset(&self) -> Option<GlobSet> {
        allow_paths_globset(&self.allow_paths)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RetentionInput {
    Enabled(bool),
    Options(RetentionOptionsInput),
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RetentionOptionsInput {
    #[serde(default = "default_require_keys")]
    require: Vec<JobRetentionKey>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    allow_paths: Vec<String>,
}

fn default_require_keys() -> Vec<JobRetentionKey> {
    vec![
        JobRetentionKey::RemoveOnComplete,
        JobRetentionKey::RemoveOnFail,
    ]
}

pub fn deserialize_unbounded_retention<'de, D>(
    deserializer: D,
) -> Result<Option<UnboundedJobRetentionConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(input) = Option::<RetentionInput>::deserialize(deserializer)? else {
        return Ok(None);
    };
    match input {
        RetentionInput::Enabled(false) => Ok(None),
        RetentionInput::Enabled(true) => Ok(Some(UnboundedJobRetentionConfig::all())),
        RetentionInput::Options(mut options) => {
            validate_allow_paths(
                "queues.unboundedJobRetention.allowPaths",
                &options.allow_paths,
            )
            .map_err(serde::de::Error::custom)?;
            let mut deduped = Vec::new();
            for key in options.require.drain(..) {
                if !deduped.contains(&key) {
                    deduped.push(key);
                }
            }
            Ok(Some(UnboundedJobRetentionConfig {
                require: deduped,
                allow_paths: options.allow_paths,
            }))
        }
    }
}

pub fn serialize_unbounded_retention<S>(
    value: &Option<UnboundedJobRetentionConfig>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let Some(config) = value else {
        return serializer.serialize_none();
    };
    if *config == UnboundedJobRetentionConfig::all() {
        return true.serialize(serializer);
    }
    RetentionOptionsInput {
        require: config.require.clone(),
        allow_paths: config.allow_paths.clone(),
    }
    .serialize(serializer)
}
