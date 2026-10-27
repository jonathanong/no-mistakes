fn collect_helper_alias_exports_from_statement<'a>(
    stmt: &'a Statement<'a>,
    defs: &mut HashMap<std::borrow::Cow<'a, str>, HelperDef<'a>>,
) {
    let Statement::ExportNamedDeclaration(export) = stmt else {
        return;
    };
    if export.export_kind.is_type() {
        return;
    }
    for specifier in &export.specifiers {
        if specifier.export_kind.is_type() {
            continue;
        }
        let local = crate::ast::module_export_name(&specifier.local);
        let exported = crate::ast::module_export_name(&specifier.exported);
        if local == exported || defs.contains_key(exported.as_ref()) {
            continue;
        }
        if let Some(def) = defs.get(local.as_ref()).copied() {
            defs.insert(exported, def);
        }
    }
}
