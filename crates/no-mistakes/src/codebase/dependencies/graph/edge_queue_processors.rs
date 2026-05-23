fn extract_processor_job_names(
    processors_file: &Path,
    facts: Option<&TsFactMap>,
) -> Option<Vec<String>> {
    use crate::codebase::ts_symbols::extract_symbols;

    if let Some(symbols) = facts
        .and_then(|facts| facts.get(processors_file))
        .and_then(|file_facts| file_facts.symbols.as_ref())
    {
        return Some(
            symbols
                .exports
                .iter()
                .filter(|e| is_processor_export_kind(&e.kind))
                .map(|e| e.name.clone())
                .collect(),
        );
    }

    let proc_source = std::fs::read_to_string(processors_file).unwrap_or_default();
    let is_tsx = processors_file
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "tsx" || e == "jsx")
        .unwrap_or(false);
    let symbols = extract_symbols(&proc_source, is_tsx).ok()?;
    Some(
        symbols
            .exports
            .into_iter()
            .filter(|e| is_processor_export_kind(&e.kind))
            .map(|e| e.name)
            .collect(),
    )
}

fn is_processor_export_kind(kind: &ExportKind) -> bool {
    matches!(
        kind,
        ExportKind::Function | ExportKind::Const | ExportKind::Let | ExportKind::Var
    )
}

