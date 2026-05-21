fn process_statement(stmt: &Statement, source: &str, out: &mut FileSymbols) {
    match stmt {
        Statement::ImportDeclaration(import) => process_import_declaration(import, source, out),
        Statement::ExportNamedDeclaration(export) => {
            process_export_named_declaration(export, source, out)
        }
        Statement::ExportDefaultDeclaration(export) => {
            process_export_default_declaration(export, source, out)
        }
        Statement::ExportAllDeclaration(export) => {
            process_export_all_declaration(export, source, out)
        }
        _ => {}
    }
}

fn process_import_declaration(import: &ImportDeclaration<'_>, source: &str, out: &mut FileSymbols) {
    let src = import.source.value.as_str();
    let is_type = import.import_kind.is_type();
    if let Some(specifiers) = &import.specifiers {
        for spec in specifiers {
            let (imported, local, span, is_specifier_type) = match spec {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    (s.imported.name().to_string(), s.local.name.as_str().to_string(), s.span, s.import_kind.is_type())
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    ("*".to_string(), s.local.name.as_str().to_string(), s.span, false)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    ("default".to_string(), s.local.name.as_str().to_string(), s.span, false)
                }
            };
            out.imports.push(NamedImport {
                source: src.to_string(),
                imported,
                local,
                line: byte_offset_to_line(source, span.start as usize),
                is_type_only: is_type || is_specifier_type,
            });
        }
    }
}
