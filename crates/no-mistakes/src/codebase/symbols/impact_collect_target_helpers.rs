fn is_namespace_reexport_symbol(facts: &TsFactMap, file: &Path, symbol: &str) -> bool {
    let Some(symbols) = facts.get(file).and_then(|facts| facts.symbols.as_ref()) else {
        return false;
    };
    symbols.exports.iter().any(|export| {
        export.name == symbol
            && matches!(
                export.kind,
                ExportKind::ReExport {
                    ref imported,
                    ..
                } if imported == "*"
            )
    })
}

fn namespace_reexport_target_symbol(
    facts: &TsFactMap,
    file: &Path,
    symbol: &str,
    target_symbol: &str,
    visible_files: &HashSet<PathBuf>,
) -> Option<String> {
    let symbols = facts.get(file)?.symbols.as_ref()?;
    if target_symbol
        .strip_prefix(symbol)
        .is_some_and(|tail| tail.starts_with('.'))
    {
        return Some(target_symbol.to_string());
    }
    if !namespace_tail_applies(facts, file, symbols, symbol, target_symbol, visible_files) {
        return None;
    }
    let local = symbols.exports.iter().find_map(|export| {
        if matches!(export.kind, ExportKind::ReExport { .. }) || export.name != symbol {
            return None;
        }
        Some(export.local.as_deref().unwrap_or(&export.name))
    });
    if local.is_some_and(|local| {
        symbols
            .imports
            .iter()
            .any(|import| import.local == local && import.imported == "*")
    }) {
        return Some(format!("{symbol}.{target_symbol}"));
    }
    symbols.exports.iter().find_map(|export| match &export.kind {
        ExportKind::ReExport {
            source, imported, ..
        } if imported == "*"
            && export.name == symbol
            && source_exports_symbol(
                facts,
                file,
                source,
                namespace_tail_root(target_symbol),
                visible_files,
            ) =>
        {
            Some(format!("{symbol}.{target_symbol}"))
        }
        _ => None,
    })
}

fn namespace_tail_root(target_symbol: &str) -> &str {
    target_symbol
        .split_once('.')
        .map_or(target_symbol, |(first, _)| first)
}

fn namespace_tail_applies(
    facts: &TsFactMap,
    file: &Path,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    symbol: &str,
    target_symbol: &str,
    visible_files: &HashSet<PathBuf>,
) -> bool {
    let Some((first, _)) = target_symbol.split_once('.') else {
        return true;
    };
    if first == symbol {
        return true;
    }
    if symbols
        .imports
        .iter()
        .any(|import| import.local == first && import.imported == "*")
    {
        return true;
    }
    symbols.exports.iter().any(|export| {
        if export.name != symbol {
            return false;
        }
        let ExportKind::ReExport { source, .. } = &export.kind else {
            return false;
        };
        source_exports_symbol(facts, file, source, first, visible_files)
    })
}

fn source_exports_symbol(
    facts: &TsFactMap,
    file: &Path,
    source: &str,
    symbol: &str,
    visible_files: &HashSet<PathBuf>,
) -> bool {
    let Some(parent) = file.parent() else {
        return false;
    };
    let Some(source_file) = resolve_relative_source_file(parent, source, visible_files) else {
        return false;
    };
    facts
        .get(&source_file)
        .and_then(|facts| facts.symbols.as_ref())
        .is_some_and(|symbols| {
            symbols
                .exports
                .iter()
                .any(|export| export_name(&export.kind, &export.name) == symbol)
        })
}

fn resolve_relative_source_file(
    parent: &Path,
    source: &str,
    visible_files: &HashSet<PathBuf>,
) -> Option<PathBuf> {
    let source_file = crate::codebase::ts_resolver::normalize_path(&parent.join(source));
    if visible_files.contains(&source_file) {
        return Some(source_file);
    }
    if source_file.extension().is_some() {
        return None;
    }
    for extension in ["mts", "ts", "tsx", "mjs", "js", "jsx", "cts", "cjs"] {
        let candidate = crate::codebase::ts_resolver::normalize_path(
            &parent.join(format!("{source}.{extension}")),
        );
        if visible_files.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
}
