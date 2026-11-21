fn order_key<'a>(value: &'a str) -> &'static str {
    let _letter = 'x';
    let _underscore = '_';
    let _newline = '\n';
    let _quote = '\'';
    let _unit = '\u{5f}';
    value.split_once('_').map_or("", |(name, _rest)| name)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
