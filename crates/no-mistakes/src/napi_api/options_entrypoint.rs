#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum EntrypointOption {
    Path(String),
    Symbol(EntrypointSymbolOption),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EntrypointSymbolOption {
    pub(crate) file: String,
    pub(crate) symbol: Option<String>,
}

impl EntrypointOption {
    pub(crate) fn into_parts(self) -> (String, Option<String>) {
        match self {
            Self::Path(path) => (path, None),
            Self::Symbol(option) => (
                option.file,
                option.symbol.filter(|symbol| !symbol.is_empty()),
            ),
        }
    }

    pub(crate) fn is_structured(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }
}
