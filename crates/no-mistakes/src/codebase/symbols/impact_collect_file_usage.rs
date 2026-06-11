fn file_entry_uses_any_symbol(root: &Path, file: &str, target_symbols: &BTreeSet<String>) -> bool {
    target_symbols
        .iter()
        .any(|target_symbol| file_entry_uses_symbol(root, file, target_symbol))
}

fn file_entry_uses_symbol(root: &Path, file: &str, target_symbol: &str) -> bool {
    let path = root.join(file);
    let mut facts_by_file = crate::codebase::ts_source::facts::collect_ts_facts(
        std::slice::from_ref(&path),
        crate::codebase::ts_source::facts::TsFactPlan::imports_and_symbols(),
    );
    let callees: BTreeSet<String> = facts_by_file
        .remove(&path)
        .map(|facts| {
            facts
                .function_calls
                .iter()
                .chain(facts.symbol_references.iter())
                .map(|call| call.callee.clone())
                .collect()
        })
        .unwrap_or_default();
    if callees.iter().any(|callee| {
        callee == target_symbol
            || callee
                .rsplit_once('.')
                .is_some_and(|(_, member)| member == target_symbol)
    }) {
        return true;
    }
    let Ok(source) = std::fs::read_to_string(path) else {
        return false;
    };
    source.contains(&format!(".{target_symbol}"))
        || source.contains(&format!("{target_symbol}("))
        || symbol_aliases_in_source(&source, target_symbol)
            .iter()
            .any(|alias| callees.contains(alias) || source.contains(&format!("{alias}(")))
}

fn symbol_aliases_in_source(source: &str, target_symbol: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    let destructured = format!("{target_symbol}:");
    let mut rest = source;
    while let Some(index) = rest.find(&destructured) {
        let after = &rest[index + destructured.len()..];
        let trimmed = after.trim_start();
        let end = trimmed
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
            .map(|(index, ch)| index + ch.len_utf8())
            .last();
        if let Some(end) = end {
            aliases.insert(trimmed[..end].to_string());
        }
        rest = after;
    }
    let member = format!(".{target_symbol}");
    for line in source.lines().filter(|line| line.contains(&member)) {
        let Some(before_equals) = line.split_once('=').map(|(before, _)| before.trim()) else {
            continue;
        };
        let Some(name) = before_equals
            .strip_prefix("const ")
            .or_else(|| before_equals.strip_prefix("let "))
            .or_else(|| before_equals.strip_prefix("var "))
            .map(str::trim)
        else {
            continue;
        };
        let end = name
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
            .map(|(index, ch)| index + ch.len_utf8())
            .last();
        if let Some(end) = end {
            aliases.insert(name[..end].to_string());
        }
    }
    aliases
}
