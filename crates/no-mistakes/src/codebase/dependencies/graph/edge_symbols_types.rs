struct SymbolGraphFiles<'a> {
    indexable: &'a [PathBuf],
    all: &'a [PathBuf],
    visible: &'a HashSet<PathBuf>,
    graph_files: &'a GraphFiles,
}

fn namespace_import_member_reference_exists(
    symbol_ref: &str,
    symbol_refs: Option<&Vec<String>>,
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
) -> bool {
    namespace_imports.contains_key(symbol_ref)
        && symbol_refs.is_some_and(|refs| {
            let prefix = format!("{symbol_ref}.");
            let bare_index = refs.iter().position(|candidate| candidate == symbol_ref);
            let member_index = refs
                .iter()
                .position(|candidate| candidate.starts_with(&prefix));
            match (bare_index, member_index) {
                (Some(bare_index), Some(member_index)) => member_index < bare_index,
                _ => false,
            }
        })
}
