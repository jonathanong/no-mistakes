use super::ImportBinding;
use oxc_ast::ast::{BindingPattern, Declaration, Expression, ObjectExpression, Program, Statement};
use std::collections::BTreeMap;

pub(in crate::integration_tests::test_config) fn sourced_reexport(
    program: &Program<'_>,
    exported: &str,
) -> Option<ImportBinding> {
    for statement in &program.body {
        let Statement::ExportFromDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type()
                || crate::ast::module_export_name(&specifier.exported) != exported
            {
                continue;
            }
            return Some(ImportBinding {
                source: crate::ast::js_literal(&export.source).into_owned(),
                imported: crate::ast::module_export_name(&specifier.local).into_owned(),
            });
        }
    }
    None
}

pub(in crate::integration_tests::test_config) fn imported_reexport(
    program: &Program<'_>,
    exported: &str,
    imports: &BTreeMap<String, ImportBinding>,
    default_identifier: bool,
) -> Option<ImportBinding> {
    if default_identifier && exported == "default" {
        if let Some(import) = program.body.iter().find_map(|statement| {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                return None;
            };
            let Expression::Identifier(identifier) = export.declaration.as_expression()? else {
                return None;
            };
            imports.get(identifier.name.as_str()).cloned()
        }) {
            return Some(import);
        }
    }
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type()
                || crate::ast::module_export_name(&specifier.exported) != exported
            {
                continue;
            }
            if let Some(import) =
                imports.get(crate::ast::module_export_name(&specifier.local).as_ref())
            {
                return Some(import.clone());
            }
        }
    }
    None
}

pub(in crate::integration_tests::test_config) fn named_export_object<'a>(
    program: &'a Program<'a>,
    exported: &str,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    expression_object: fn(
        &'a Expression<'a>,
        &BTreeMap<String, &'a Expression<'a>>,
    ) -> Option<&'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in &program.body {
        let Statement::ExportDeclaration(export) = statement else {
            continue;
        };
        let Declaration::VariableDeclaration(declaration) = &export.declaration else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                continue;
            };
            if identifier.name == exported {
                return declarator
                    .init
                    .as_ref()
                    .and_then(|expression| expression_object(expression, bindings));
            }
        }
    }
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type()
                || crate::ast::module_export_name(&specifier.exported) != exported
            {
                continue;
            }
            let local = crate::ast::module_export_name(&specifier.local);
            if let Some(object) = bindings
                .get(local.as_ref())
                .and_then(|expression| expression_object(expression, bindings))
            {
                return Some(object);
            }
        }
    }
    None
}
