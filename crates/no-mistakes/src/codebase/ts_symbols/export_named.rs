fn process_export_named_declaration(
    export: &ExportNamedDeclaration<'_>,
    source: &str,
    local_type_names: &HashSet<String>,
    out: &mut FileSymbols,
) {
    let line = byte_offset_to_line(source, export.span.start as usize);
    let export_is_type = export.export_kind.is_type();

    // Specifier exports without source: `export { a, b }` (local re-bindings)
    for spec in &export.specifiers {
        let name = spec.exported.name().to_string();
        let local = spec.local.name().to_string();
        out.exports.push(Export {
            name,
            local: (local != spec.exported.name().as_str()).then_some(local),
            kind: ExportKind::Const,
            line,
            is_type_only: export_is_type
                || spec.export_kind.is_type()
                || local_type_names.contains(spec.local.name().as_str()),
        });
    }
}

fn process_export_declaration(
    export: &ExportDeclaration<'_>,
    source: &str,
    out: &mut FileSymbols,
) {
    let line = byte_offset_to_line(source, export.span.start as usize);
    match &export.declaration {
            Declaration::FunctionDeclaration(func) => {
                push_export_if_named(
                    out,
                    func.id.as_ref().map(|id| id.name.as_str()),
                    ExportKind::Function,
                    line,
                    false,
                );
            }
            Declaration::ClassDeclaration(cls) => {
                push_export_if_named(
                    out,
                    cls.id.as_ref().map(|id| id.name.as_str()),
                    ExportKind::Class,
                    line,
                    false,
                );
            }
            Declaration::VariableDeclaration(var) => {
                let kind = match var.kind {
                    VariableDeclarationKind::Const
                    | VariableDeclarationKind::Using
                    | VariableDeclarationKind::AwaitUsing => ExportKind::Const,
                    VariableDeclarationKind::Let => ExportKind::Let,
                    VariableDeclarationKind::Var => ExportKind::Var,
                };
                for decl in &var.declarations {
                    collect_binding_names(&decl.id, kind.clone(), line, false, out);
                }
            }
            Declaration::TSTypeAliasDeclaration(ta) => {
                out.exports.push(Export {
                    name: ta.id.name.as_str().to_string(),
                    local: None,
                    kind: ExportKind::TypeAlias,
                    line,
                    is_type_only: true,
                });
            }
            Declaration::TSInterfaceDeclaration(iface) => {
                out.exports.push(Export {
                    name: iface.id.name.as_str().to_string(),
                    local: None,
                    kind: ExportKind::Interface,
                    line,
                    is_type_only: true,
                });
            }
            Declaration::TSEnumDeclaration(en) => {
                out.exports.push(Export {
                    name: en.id.name.as_str().to_string(),
                    local: None,
                    kind: ExportKind::Enum,
                    line,
                    is_type_only: false,
                });
            }
            _ => {}
    }
}

fn process_export_from_declaration(
    export: &ExportFromDeclaration<'_>,
    source: &str,
    out: &mut FileSymbols,
) {
    let line = byte_offset_to_line(source, export.span.start as usize);
    let source_str = export.source.value.as_str().to_string();
    for spec in &export.specifiers {
        let imported = spec.local.name().to_string();
        let name = spec.exported.name().to_string();
        out.exports.push(Export {
            name,
            local: None,
            kind: ExportKind::ReExport {
                source: source_str.clone(),
                imported,
            },
            line,
            is_type_only: export.export_kind.is_type() || spec.export_kind.is_type(),
        });
    }
}
