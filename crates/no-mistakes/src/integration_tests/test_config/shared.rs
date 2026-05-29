use oxc_ast::ast::{BindingPattern, Declaration, Expression, Program, Statement};
use std::collections::BTreeMap;

mod objects;

pub(in crate::integration_tests) use super::shared_literals::{
    inferred_string_or_array, optional_string, property_key_name, required_string,
};
pub(in crate::integration_tests) use objects::{
    default_export_object, property_expression, property_expression_deep, property_object,
};

pub(in crate::integration_tests) fn top_level_object_bindings<'a>(
    program: &'a Program<'a>,
) -> BTreeMap<String, &'a Expression<'a>> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => Some(declaration),
            Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
                Some(Declaration::VariableDeclaration(declaration)) => Some(declaration),
                _ => None,
            },
            _ => None,
        };
        let Some(declaration) = declaration else {
            continue;
        };
        for declarator in &declaration.declarations {
            let (Some(name), Some(init)) =
                (binding_identifier_name(&declarator.id), &declarator.init)
            else {
                continue;
            };
            bindings.insert(name.to_string(), init);
        }
    }
    bindings
}

fn binding_identifier_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
