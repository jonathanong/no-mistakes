fn is_test_like_file(file: &Path) -> bool {
    file.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains(".test.") || name.contains(".spec."))
}

fn caller_is_target_export(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    file: &Path,
    target_symbols: &BTreeMap<PathBuf, BTreeSet<String>>,
    caller: &str,
) -> bool {
    let Some(file_symbols) = target_symbols.get(file) else {
        return false;
    };
    if file_symbols.contains(caller) {
        return true;
    }
    exported_symbol_for_local(symbols, caller)
        .as_ref()
        .is_some_and(|symbol| file_symbols.contains(symbol))
}

fn matches_local_callee(callee: &str, local_names: &BTreeSet<String>) -> bool {
    local_names.iter().any(|local| {
        callee == local
            || callee
                .strip_prefix(local)
                .is_some_and(|suffix| suffix.starts_with('.'))
    })
}
