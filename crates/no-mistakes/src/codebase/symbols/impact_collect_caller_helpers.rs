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

/// Legacy signature-impact callers answer resolved symbol usage, unlike call
/// policy reports. A new retained unknown/shadowed call must not be attributed
/// to a same-spelled import merely because its text happens to match.
fn legacy_call_matches_local_target(
    call: &crate::codebase::dependencies::extract::FunctionCall,
    local_names: &BTreeSet<String>,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> bool {
    if !matches_local_callee(&call.callee, local_names) {
        return false;
    }
    use crate::codebase::dependencies::extract::CallTargetIdentity;
    match call.target_identity {
        CallTargetIdentity::ModuleExport => true,
        CallTargetIdentity::RepositoryFunction | CallTargetIdentity::Unknown => !facts.imported_bindings.iter().any(|binding| {
            call.callee == binding.local
                || call
                    .callee
                    .strip_prefix(&binding.local)
                    .is_some_and(|suffix| suffix.starts_with('.'))
        }),
        CallTargetIdentity::Global => false,
    }
}
