use oxc_ast::ast::{BindingPattern, Declaration, Statement, VariableDeclarationKind};
use std::collections::HashSet;

pub(super) fn bound(pattern: &BindingPattern<'_>, names: &mut HashSet<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.insert(id.name.to_string());
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                bound(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                bound(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                bound(element, names);
            }
            if let Some(rest) = &array.rest {
                bound(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => bound(&assignment.left, names),
    }
}

pub(super) fn declaration(value: &Declaration<'_>, names: &mut HashSet<String>) {
    match value {
        Declaration::VariableDeclaration(value) if value.kind != VariableDeclarationKind::Var => {
            for value in &value.declarations {
                bound(&value.id, names);
            }
        }
        Declaration::FunctionDeclaration(value) => {
            if let Some(id) = &value.id {
                names.insert(id.name.to_string());
            }
        }
        Declaration::ClassDeclaration(value) => {
            if let Some(id) = &value.id {
                names.insert(id.name.to_string());
            }
        }
        Declaration::TSNamespaceDeclaration(value) => {
            names.insert(value.id.name.to_string());
        }
        Declaration::TSImportEqualsDeclaration(value) => {
            names.insert(value.id.name.to_string());
        }
        Declaration::TSEnumDeclaration(value) => {
            names.insert(value.id.name.to_string());
        }
        _ => {}
    }
}

fn lexical_with_imports(statements: &[Statement<'_>], imports: bool) -> HashSet<String> {
    let mut names = HashSet::new();
    for statement in statements {
        match statement {
            Statement::ImportDeclaration(value) if imports => {
                for value in value.specifiers.iter().flatten() {
                    use oxc_ast::ast::ImportDeclarationSpecifier::*;
                    let id = match value {
                        ImportSpecifier(value) => &value.local,
                        ImportDefaultSpecifier(value) => &value.local,
                        ImportNamespaceSpecifier(value) => &value.local,
                    };
                    names.insert(id.name.to_string());
                }
            }
            Statement::ExportDeclaration(value) => declaration(&value.declaration, &mut names),
            Statement::ExportDefaultDeclaration(value) => {
                use oxc_ast::ast::ExportDefaultDeclarationKind::*;
                let id = match &value.declaration {
                    FunctionDeclaration(value) => value.id.as_ref(),
                    ClassDeclaration(value) => value.id.as_ref(),
                    _ => None,
                };
                if let Some(id) = id {
                    names.insert(id.name.to_string());
                }
            }
            _ => {
                if let Some(value) = statement.as_declaration() {
                    declaration(value, &mut names);
                }
            }
        }
    }
    names
}

pub(super) fn lexical(statements: &[Statement<'_>]) -> HashSet<String> {
    lexical_with_imports(statements, true)
}

pub(super) fn declarations(statements: &[Statement<'_>]) -> HashSet<String> {
    let mut names = lexical_with_imports(statements, false);
    names.extend(super::hoisted::collect(statements));
    names
}

pub(super) fn function_scope(statements: &[Statement<'_>]) -> HashSet<String> {
    let mut names = lexical(statements);
    names.extend(super::hoisted::collect(statements));
    names
}

pub(super) fn module_functions(statements: &[Statement<'_>]) -> HashSet<String> {
    statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::FunctionDeclaration(value) => value.id.as_ref(),
            Statement::ExportDeclaration(value) => match &value.declaration {
                Declaration::FunctionDeclaration(value) => value.id.as_ref(),
                _ => None,
            },
            Statement::ExportDefaultDeclaration(value) => match &value.declaration {
                oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(value) => {
                    value.id.as_ref()
                }
                _ => None,
            },
            _ => None,
        })
        .map(|id| id.name.to_string())
        .collect()
}
