#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum EntrypointOption {
    Path(String),
    Symbol {
        file: String,
        symbol: Option<String>,
    },
}

impl EntrypointOption {
    pub(crate) fn into_parts(self) -> (String, Option<String>) {
        match self {
            Self::Path(path) => (path, None),
            Self::Symbol { file, symbol } => (file, symbol.filter(|symbol| !symbol.is_empty())),
        }
    }
}
