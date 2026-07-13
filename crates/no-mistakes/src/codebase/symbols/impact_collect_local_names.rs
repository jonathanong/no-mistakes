fn target_local_names(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    file: &Path,
    target_symbols: &BTreeMap<PathBuf, BTreeSet<String>>,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    visible_files: &HashSet<PathBuf>,
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    if let Some(exported_symbols) = target_symbols.get(file) {
        names.extend(symbols
            .exports
            .iter()
            .filter_map(|export| {
                if !matches!(export.kind, ExportKind::ReExport { .. })
                    && !export.is_type_only
                    && (exported_symbols.contains(&export.name)
                        || (exported_symbols.contains("default")
                            && matches!(export.kind, ExportKind::Default)))
                {
                    return Some(export.local.clone().unwrap_or_else(|| export.name.clone()));
                }
                None
            }));
        if names.is_empty() {
            names.extend(exported_symbols.clone());
        }
    }

    names.extend(imported_target_local_names(
        symbols,
        file,
        target_symbols,
        tsconfig,
        workspace,
        visible_files,
    ));
    names
}

fn imported_target_local_names(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    file: &Path,
    target_symbols: &BTreeMap<PathBuf, BTreeSet<String>>,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    visible_files: &HashSet<PathBuf>,
) -> BTreeSet<String> {
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(tsconfig)
        .with_visible(visible_files);
    symbols
        .imports
        .iter()
        .filter_map(|import| {
            if import.is_type_only {
                return None;
            }
            let exported_symbols = resolver
                .resolve(&import.source, file)
                .or_else(|| {
                    workspace.resolve_specifier_from_file_visible(
                        &import.source,
                        file,
                        visible_files,
                    )
                })
                .and_then(|resolved| target_symbols.get(&resolved))?;
            if exported_symbols.is_empty() {
                return None;
            }
            if import.imported == "*" {
                return Some(
                    exported_symbols
                        .iter()
                        .map(|symbol| format!("{}.{}", import.local, symbol))
                        .collect::<BTreeSet<_>>(),
                );
            }
            let member_names: BTreeSet<_> = exported_symbols
                .iter()
                .filter_map(|symbol| {
                    symbol
                        .strip_prefix(&format!("{}.", import.imported))
                        .map(|suffix| format!("{}.{}", import.local, suffix))
                })
                .collect();
            if !member_names.is_empty() {
                return Some(member_names);
            }
            exported_symbols
                .contains(&import.imported)
                .then(|| BTreeSet::from([import.local.clone()]))
        })
        .flatten()
        .collect()
}

fn exported_symbol_for_local(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    local: &str,
) -> Option<String> {
    symbols.exports.iter().find_map(|export| {
        if matches!(export.kind, ExportKind::ReExport { .. }) || export.is_type_only {
            return None;
        }
        (export.local.as_deref().unwrap_or(&export.name) == local).then(|| {
            if matches!(export.kind, ExportKind::Default) {
                "default".to_string()
            } else {
                export.name.clone()
            }
        })
    })
}
