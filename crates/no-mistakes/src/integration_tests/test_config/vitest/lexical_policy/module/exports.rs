use super::super::{Kind, Value};
use super::*;

pub(super) fn parse(
    reader: &mut Reader<'_>,
    path: &Path,
    source: &str,
    program: &Program<'_>,
    evaluator: &mut Evaluator<'_, '_>,
    exported: BTreeSet<String>,
) -> Module {
    let mut module = stars(reader, path, source, program);
    for name in exported {
        let value = evaluator
            .named(&name)
            .unwrap_or_else(|| unknown(path, source, 0));
        module.exports.insert(name, value);
    }
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                let value = export
                    .declaration
                    .as_expression()
                    .map(|value| evaluator.eval(value))
                    .unwrap_or_else(|| unknown(path, source, export.span.start));
                module.exports.insert("default".into(), value);
            }
            Statement::ExportNamedDeclaration(export) if !export.export_kind.is_type() => {
                for specifier in &export.specifiers {
                    if specifier.export_kind.is_type() {
                        continue;
                    }
                    let local = crate::ast::module_export_name(&specifier.local);
                    let value = evaluator.named(local.as_ref()).unwrap_or_else(|| {
                        module.opaque = true;
                        unknown(path, source, specifier.span.start)
                    });
                    module.exports.insert(
                        crate::ast::module_export_name(&specifier.exported).into_owned(),
                        value,
                    );
                }
            }
            Statement::ExportFromDeclaration(export) if !export.export_kind.is_type() => {
                let external = reader.import(path, crate::ast::js_literal(&export.source).as_ref());
                for specifier in &export.specifiers {
                    if specifier.export_kind.is_type() {
                        continue;
                    }
                    module.opaque |= external.opaque;
                    let local = crate::ast::module_export_name(&specifier.local);
                    let value = external
                        .exports
                        .get(local.as_ref())
                        .cloned()
                        .unwrap_or_else(|| {
                            module.opaque = true;
                            unknown(path, source, specifier.span.start)
                        });
                    module.exports.insert(
                        crate::ast::module_export_name(&specifier.exported).into_owned(),
                        value,
                    );
                }
            }
            Statement::ExportDeclaration(export) => {
                module.opaque |= match &export.declaration {
                    Declaration::VariableDeclaration(declaration) => {
                        declaration.declarations.iter().any(|binding| {
                            !matches!(
                                &binding.id,
                                oxc_ast::ast::BindingPattern::BindingIdentifier(_)
                            )
                        })
                    }
                    Declaration::TSTypeAliasDeclaration(_)
                    | Declaration::TSInterfaceDeclaration(_) => false,
                    _ => true,
                };
            }
            _ => {}
        }
    }
    if super::super::mutations::contains_mutation(program) {
        module.opaque = true;
        for value in module.exports.values_mut() {
            *value = value.unknown();
        }
    }
    module
}

fn stars(reader: &mut Reader<'_>, path: &Path, source: &str, program: &Program<'_>) -> Module {
    let mut module = Module {
        exports: BTreeMap::new(),
        opaque: false,
    };
    let mut namespaces = BTreeMap::new();
    for statement in &program.body {
        let Statement::ExportAllDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() {
            continue;
        }
        let external = reader.import(path, crate::ast::js_literal(&export.source).as_ref());
        module.opaque |= external.opaque;
        if let Some(name) = &export.exported {
            namespaces.insert(
                crate::ast::module_export_name(name).into_owned(),
                external.namespace(path, line(source, export.span.start)),
            );
            continue;
        }
        for (name, value) in external.exports {
            if name == "default" {
                continue;
            }
            module
                .exports
                .entry(name)
                .and_modify(|prior| *prior = prior.unknown())
                .or_insert(value);
        }
    }
    if module.opaque {
        for value in module.exports.values_mut() {
            *value = value.unknown();
        }
    }
    module.exports.extend(namespaces);
    module
}
fn line(source: &str, offset: u32) -> usize {
    source[..offset as usize]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}
fn unknown(path: &Path, source: &str, offset: u32) -> Value {
    Value {
        kind: Kind::Unknown,
        file: path.to_path_buf(),
        line: line(source, offset),
    }
}
