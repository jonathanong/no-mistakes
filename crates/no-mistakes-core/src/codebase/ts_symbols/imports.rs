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
            match spec {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    let imported = s.imported.name().to_string();
                    let local = s.local.name.as_str().to_string();
                    let line = byte_offset_to_line(source, s.span.start as usize);
                    out.imports.push(NamedImport {
                        source: src.to_string(),
                        imported,
                        local,
                        line,
                        is_type_only: is_type || s.import_kind.is_type(),
                    });
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    let local = s.local.name.as_str().to_string();
                    let line = byte_offset_to_line(source, s.span.start as usize);
                    out.imports.push(NamedImport {
                        source: src.to_string(),
                        imported: "*".to_string(),
                        local,
                        line,
                        is_type_only: is_type,
                    });
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    let local = s.local.name.as_str().to_string();
                    let line = byte_offset_to_line(source, s.span.start as usize);
                    out.imports.push(NamedImport {
                        source: src.to_string(),
                        imported: "default".to_string(),
                        local,
                        line,
                        is_type_only: is_type,
                    });
                }
            }
        }
    }
}
