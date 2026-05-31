fn process_export_default_declaration(
    export: &ExportDefaultDeclaration<'_>,
    source: &str,
    out: &mut FileSymbols,
) {
    let line = byte_offset_to_line(source, export.span.start as usize);
    let is_type_only = matches!(
        &export.declaration,
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)
    );
    let name = match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
            default_export_name(f.id.as_ref().map(|id| id.name.as_str()))
        }
        ExportDefaultDeclarationKind::ClassDeclaration(c) => {
            default_export_name(c.id.as_ref().map(|id| id.name.as_str()))
        }
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(i) => i.id.name.as_str().to_string(),
        ExportDefaultDeclarationKind::Identifier(id) => id.name.as_str().to_string(),
        _ => "default".to_string(),
    };
    out.exports.push(Export {
        name,
        local: is_type_only.then(|| "default".to_string()),
        kind: ExportKind::Default,
        line,
        is_type_only,
    });
}

fn process_export_all_declaration(
    export: &ExportAllDeclaration<'_>,
    source: &str,
    out: &mut FileSymbols,
) {
    let source_str = export.source.value.as_str().to_string();
    let line = byte_offset_to_line(source, export.span.start as usize);
    out.exports.push(Export {
        name: export
            .exported
            .as_ref()
            .map(|name| name.name().to_string())
            .unwrap_or_else(|| "*".to_string()),
        local: None,
        kind: ExportKind::ReExport {
            source: source_str,
            imported: "*".to_string(),
        },
        line,
        is_type_only: export.export_kind.is_type(),
    });
}

fn push_export_if_named(
    out: &mut FileSymbols,
    name: Option<&str>,
    kind: ExportKind,
    line: u32,
    is_type_only: bool,
) {
    if let Some(name) = name {
        out.exports.push(Export {
            name: name.to_string(),
            local: None,
            kind,
            line,
            is_type_only,
        });
    }
}

fn default_export_name(name: Option<&str>) -> String {
    name.unwrap_or("default").to_string()
}

fn collect_binding_names(
    pat: &BindingPattern,
    kind: ExportKind,
    line: u32,
    is_type_only: bool,
    out: &mut FileSymbols,
) {
    match pat {
        BindingPattern::BindingIdentifier(id) => {
            out.exports.push(Export {
                name: id.name.as_str().to_string(),
                local: None,
                kind,
                line,
                is_type_only,
            });
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names(&prop.value, kind.clone(), line, is_type_only, out);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names(elem, kind.clone(), line, is_type_only, out);
            }
        }
        BindingPattern::AssignmentPattern(ap) => {
            collect_binding_names(&ap.left, kind, line, is_type_only, out);
        }
    }
}
