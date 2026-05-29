use super::super::{import_bindings, objects, ImportBinding};
use oxc_ast::ast::{Declaration, ObjectExpression, Program, Statement};

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn sourced_reexport(
    program: &Program<'_>,
    exported: &str,
) -> Option<ImportBinding> {
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() {
            continue;
        }
        let Some(source) = &export.source else {
            continue;
        };
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() || specifier.exported.name() != exported {
                continue;
            }
            return Some(ImportBinding {
                source: source.value.to_string(),
                imported: specifier.local.name().to_string(),
            });
        }
    }
    None
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn imported_reexport(
    program: &Program<'_>,
    exported: &str,
) -> Option<ImportBinding> {
    let imports = import_bindings(program);
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() || export.source.is_some() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() || specifier.exported.name() != exported {
                continue;
            }
            if let Some(import) = imports.get(specifier.local.name().as_str()) {
                return Some(import.clone());
            }
        }
    }
    None
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn named_export_object<'a>(
    program: &'a Program<'a>,
    exported: &str,
    bindings: &super::super::ExprMap<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration else {
            continue;
        };
        for declarator in &declaration.declarations {
            let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                continue;
            };
            if identifier.name == exported {
                return declarator
                    .init
                    .as_ref()
                    .and_then(|expression| objects::expression_object(expression, bindings));
            }
        }
    }
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() || specifier.exported.name() != exported {
                continue;
            }
            let local = specifier.local.name();
            if let Some(object) = bindings
                .get(local.as_str())
                .and_then(|expression| objects::expression_object(expression, bindings))
            {
                return Some(object);
            }
        }
    }
    None
}
