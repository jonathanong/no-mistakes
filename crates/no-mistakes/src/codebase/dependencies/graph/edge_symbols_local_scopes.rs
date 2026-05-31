fn local_scope_names(
    calls_by_caller: &HashMap<String, Vec<String>>,
    refs_by_caller: &HashMap<String, Vec<String>>,
    scoped_imports: &HashMap<String, Vec<(NodeId, EdgeKind)>>,
) -> HashSet<String> {
    calls_by_caller
        .keys()
        .chain(refs_by_caller.keys())
        .chain(scoped_imports.keys())
        .cloned()
        .collect()
}

fn exported_local_is_callable(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    exported_functions: &[String],
    local: &str,
) -> bool {
    symbols.exports.iter().any(|export| {
        export_local_name(export) == local
            && matches!(export.kind, ExportKind::Function | ExportKind::Class)
            || export.name == local
                && exported_functions.iter().any(|name| name == local)
                && matches!(
                    export.kind,
                    ExportKind::Const | ExportKind::Let | ExportKind::Var | ExportKind::Default
                )
    })
}

fn resolve_local_scope(caller: &str, callee: &str, scopes: &HashSet<String>) -> Option<String> {
    if scopes.contains(callee) {
        return Some(callee.to_string());
    }
    let mut scope = caller;
    loop {
        let candidate = format!("{scope}/{callee}");
        if scopes.contains(&candidate) {
            return Some(candidate);
        }
        let (parent, _) = scope.rsplit_once('/')?;
        scope = parent;
    }
}
