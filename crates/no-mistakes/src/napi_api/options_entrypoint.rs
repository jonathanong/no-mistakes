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
    pub(crate) fn into_cli_string(self) -> String {
        match self {
            Self::Path(path) => path,
            Self::Symbol { file, symbol } => match symbol {
                Some(symbol) if !symbol.is_empty() => format!("{file}#{symbol}"),
                _ => file,
            },
        }
    }
}
