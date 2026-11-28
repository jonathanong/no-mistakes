#[derive(Debug, Default)]
pub(crate) struct ConfigResolveOptions {
    pub(crate) project: ProjectOptions,
    pub(crate) view: ConfigResolveView,
}

impl<'de> Deserialize<'de> for ConfigResolveOptions {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut value = Value::deserialize(deserializer)?;
        let view = value.as_object_mut().and_then(|object| object.remove("view"))
            .map(serde_json::from_value).transpose().map_err(serde::de::Error::custom)?
            .unwrap_or_default();
        let project = ProjectOptions::deserialize(value).map_err(serde::de::Error::custom)?;
        Ok(Self { project, view })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ConfigResolveView {
    #[default]
    Resolved,
    Declared,
}
