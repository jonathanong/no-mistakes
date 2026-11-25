use super::{allow_paths_globset, validate_allow_paths};
use globset::GlobSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueIncludeTestsConfig {
    All,
    Paths(Vec<String>),
}

impl QueueIncludeTestsConfig {
    pub fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    pub fn globset(&self) -> Option<GlobSet> {
        match self {
            Self::All => None,
            Self::Paths(paths) => allow_paths_globset(paths),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum IncludeTestsInput {
    Enabled(bool),
    Paths(Vec<String>),
}

pub fn deserialize_include_tests<'de, D>(
    deserializer: D,
) -> Result<Option<QueueIncludeTestsConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(input) = Option::<IncludeTestsInput>::deserialize(deserializer)? else {
        return Ok(None);
    };
    match input {
        IncludeTestsInput::Enabled(false) => Ok(None),
        IncludeTestsInput::Enabled(true) => Ok(Some(QueueIncludeTestsConfig::All)),
        IncludeTestsInput::Paths(paths) => {
            validate_allow_paths("queues.includeTests", &paths)
                .map_err(serde::de::Error::custom)?;
            let mut deduped = Vec::new();
            for pattern in paths {
                if !deduped.contains(&pattern) {
                    deduped.push(pattern);
                }
            }
            if deduped.is_empty() {
                Ok(None)
            } else {
                Ok(Some(QueueIncludeTestsConfig::Paths(deduped)))
            }
        }
    }
}

pub fn serialize_include_tests<S>(
    value: &Option<QueueIncludeTestsConfig>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(QueueIncludeTestsConfig::All) => true.serialize(serializer),
        Some(QueueIncludeTestsConfig::Paths(paths)) => paths.serialize(serializer),
    }
}
