fn is_namespace_reexport_symbol(file: &Path, symbol: &str) -> bool {
    let Ok(source) = std::fs::read_to_string(file) else {
        return false;
    };
    let is_tsx = file
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("tsx") || ext.eq_ignore_ascii_case("jsx"));
    let Ok(symbols) = extract_symbols(&source, is_tsx) else {
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
    file: &Path,
    symbol: &str,
    target_symbol: &str,
) -> Option<String> {
    let source = std::fs::read_to_string(file).ok()?;
    let is_tsx = file
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("tsx") || ext.eq_ignore_ascii_case("jsx"));
    let symbols = extract_symbols(&source, is_tsx).ok()?;
    if target_symbol
        .strip_prefix(symbol)
        .is_some_and(|tail| tail.starts_with('.'))
    {
        return Some(target_symbol.to_string());
    }
    if !namespace_tail_applies(file, &symbols, symbol, target_symbol) {
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
            && source_exports_symbol(file, source, namespace_tail_root(target_symbol)) =>
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
    file: &Path,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    symbol: &str,
    target_symbol: &str,
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
        source_exports_symbol(file, source, first)
    })
}

fn source_exports_symbol(file: &Path, source: &str, symbol: &str) -> bool {
    let Some(parent) = file.parent() else {
        return false;
    };
    let source_file = resolve_relative_source_file(parent, source);
    let Ok(source) = std::fs::read_to_string(&source_file) else {
        return false;
    };
    let is_tsx = source_file
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("tsx") || ext.eq_ignore_ascii_case("jsx"));
    extract_symbols(&source, is_tsx)
        .ok()
        .is_some_and(|symbols| {
            symbols
                .exports
                .iter()
                .any(|export| export_name(&export.kind, &export.name) == symbol)
        })
}

fn resolve_relative_source_file(parent: &Path, source: &str) -> PathBuf {
    let source_file = parent.join(source);
    if source_file.exists() || source_file.extension().is_some() {
        return source_file;
    }
    for extension in ["mts", "ts", "tsx", "mjs", "js", "jsx", "cts", "cjs"] {
        let candidate = parent.join(format!("{source}.{extension}"));
        if candidate.exists() {
            return candidate;
        }
    }
    source_file
}
