#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum, serde::Deserialize, serde::Serialize,
)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum TraverseProjection {
    #[default]
    Graph,
    Paths,
}

impl TraverseProjection {
    pub(crate) fn is_paths(self) -> bool {
        matches!(self, Self::Paths)
    }
}
