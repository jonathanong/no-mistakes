use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_symbols::ExportKind;
use std::path::Path;

pub(super) fn module_gap_message(ts: &TsFileFacts, module: &str) -> (usize, String) {
    if let Some((resolved, symbol)) = module
        .rsplit_once('#')
        .filter(|(path, _)| Path::new(path).is_absolute())
    {
        let (specifier, line) = import_site(ts, symbol, true).unwrap_or((resolved.to_owned(), 1));
        return (
            line,
            format!(
                "unresolved symbol `{symbol}` from `{specifier}` prevents complete per-symbol analysis"
            ),
        );
    }
    (
        import_site(ts, module, false).map_or(1, |(_, line)| line),
        format!("unresolved module selector `{module}` prevents complete per-symbol analysis"),
    )
}

fn import_site(ts: &TsFileFacts, key: &str, symbol: bool) -> Option<(String, usize)> {
    // Callers skip files with no symbol table before asking for a selector site.
    let symbols = ts
        .symbols
        .as_ref()
        .expect("selector gaps run after symbols are present");
    if let Some(import) = symbols.imports.iter().find(|import| {
        if symbol {
            import.imported == key
        } else {
            import.source == key
        }
    }) {
        return Some((import.source.clone(), import.line.max(1) as usize));
    }
    if let Some((source, line)) = symbols.exports.iter().find_map(|export| {
        let ExportKind::ReExport { source, imported } = &export.kind else {
            return None;
        };
        let matched = if symbol {
            imported == key
        } else {
            source == key
        };
        matched.then(|| (source.clone(), export.line.max(1) as usize))
    }) {
        return Some((source, line));
    }
    if symbol {
        return None;
    }
    if let Some(import) = ts.imports.iter().find(|import| import.specifier == key) {
        return Some((key.to_owned(), import.line.max(1) as usize));
    }
    let import = ts
        .module_bindings
        .as_ref()?
        .imports
        .iter()
        .find(|import| import.specifier == key)?;
    Some((
        key.to_owned(),
        crate::codebase::ts_source::byte_offset_to_line(
            ts.source.as_deref()?,
            import.span.start as usize,
        )
        .max(1) as usize,
    ))
}
