pub(crate) fn parse_symbols_mode(value: Option<&str>) -> AnyhowResult<SymbolsMode> {
    match value.unwrap_or("list") {
        "list" => Ok(SymbolsMode::List),
        "signature-impact" => Ok(SymbolsMode::SignatureImpact),
        value => bail!("unknown symbols mode: {value}"),
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod options_symbols_tests;
