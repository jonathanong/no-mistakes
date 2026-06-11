fn collect_helper_alias_exports_from_statement<'a>(
    stmt: &'a Statement<'a>,
    defs: &mut HashMap<&'a str, HelperDef<'a>>,
) {
    let Statement::ExportNamedDeclaration(export) = stmt else {
        return;
    };
    if export.source.is_some() || export.export_kind.is_type() {
        return;
    }
    for specifier in &export.specifiers {
        if specifier.export_kind.is_type() {
            continue;
        }
        let local = specifier.local.name().as_str();
        let exported = specifier.exported.name().as_str();
        if local == exported || defs.contains_key(exported) {
            continue;
        }
        if let Some(def) = defs.get(local).copied() {
            defs.insert(
                exported,
                HelperDef {
                    name: exported,
                    ..def
                },
            );
        }
    }
}
