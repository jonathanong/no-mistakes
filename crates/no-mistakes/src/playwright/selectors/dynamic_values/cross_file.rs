use super::collect::{collect_object_string_values, collect_string_leaves};
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier,
    ObjectPropertyKind, Program, Statement,
};
use std::path::Path;

pub(crate) fn resolve_imported_values(
    local_name: &str,
    program: &Program<'_>,
    importing_file: &Path,
) -> Vec<String> {
    let Some((source_str, exported_name, is_default)) = find_import_info(local_name, program)
    else {
        return vec![];
    };

    let Some(resolved_path) = crate::fetch::resolve::resolve_import(importing_file, &source_str)
    else {
        return vec![];
    };

    let Ok(source) = std::fs::read_to_string(&resolved_path) else {
        return vec![];
    };

    crate::playwright::ast::with_program(&resolved_path, &source, |target_program, _| {
        collect_exported_values(target_program, &exported_name, is_default)
    })
    .unwrap_or_default()
}

fn find_import_info(local_name: &str, program: &Program<'_>) -> Option<(String, String, bool)> {
    for stmt in &program.body {
        let Statement::ImportDeclaration(import) = stmt else {
            continue;
        };
        let source_str = import.source.value.to_string();
        for specifier in import.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named) => {
                    if named.local.name == local_name {
                        let exported = named.imported.name().to_string();
                        return Some((source_str, exported, false));
                    }
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                    if default.local.name == local_name {
                        return Some((source_str, "default".to_string(), true));
                    }
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
        }
    }
    None
}

fn collect_exported_values(
    program: &Program<'_>,
    exported_name: &str,
    is_default: bool,
) -> Vec<String> {
    let mut values = Vec::new();

    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(decl) = &export.declaration {
                    collect_from_named_declaration(decl, exported_name, &mut values);
                }
            }
            Statement::ExportDefaultDeclaration(export) if is_default => {
                collect_from_default_export(&export.declaration, &mut values);
            }
            _ => {}
        }
    }

    values
}

fn collect_from_default_export(kind: &ExportDefaultDeclarationKind<'_>, values: &mut Vec<String>) {
    match kind {
        ExportDefaultDeclarationKind::ObjectExpression(obj) => {
            for prop in &obj.properties {
                if let ObjectPropertyKind::ObjectProperty(p) = prop {
                    if !p.computed {
                        if let Expression::StringLiteral(lit) = &p.value {
                            values.push(lit.value.to_string());
                        }
                    }
                }
            }
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            if let Some(body) = &func.body {
                collect_returns_from_function_body(&body.statements, values);
            }
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                for s in &arrow.body.statements {
                    if let Statement::ExpressionStatement(expr_stmt) = s {
                        values.extend(collect_string_leaves(&expr_stmt.expression));
                    }
                }
            } else {
                collect_returns_from_function_body(&arrow.body.statements, values);
            }
        }
        _ => {}
    }
}

fn collect_from_named_declaration(
    decl: &Declaration<'_>,
    exported_name: &str,
    values: &mut Vec<String>,
) {
    match decl {
        Declaration::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                let Some(name) = binding_ident_name(&declarator.id) else {
                    continue;
                };
                if name != exported_name {
                    continue;
                }
                let Some(init) = declarator.init.as_ref() else {
                    continue;
                };
                let leaves = collect_string_leaves(init);
                if !leaves.is_empty() {
                    values.extend(leaves);
                    continue;
                }
                values.extend(collect_object_string_values(init));
            }
        }
        Declaration::FunctionDeclaration(func)
            if func.id.as_ref().is_some_and(|id| id.name == exported_name) =>
        {
            if let Some(body) = &func.body {
                collect_returns_from_function_body(&body.statements, values);
            }
        }
        _ => {}
    }
}

fn collect_returns_from_function_body(statements: &[Statement<'_>], values: &mut Vec<String>) {
    for stmt in statements {
        match stmt {
            Statement::ReturnStatement(ret) => {
                if let Some(expr) = &ret.argument {
                    values.extend(collect_string_leaves(expr));
                }
            }
            Statement::IfStatement(if_stmt) => {
                collect_returns_from_stmt(&if_stmt.consequent, values);
                if let Some(alt) = &if_stmt.alternate {
                    collect_returns_from_stmt(alt, values);
                }
            }
            Statement::BlockStatement(block) => {
                collect_returns_from_function_body(&block.body, values);
            }
            _ => {}
        }
    }
}

fn collect_returns_from_stmt(stmt: &Statement<'_>, values: &mut Vec<String>) {
    match stmt {
        Statement::ReturnStatement(ret) => {
            if let Some(expr) = &ret.argument {
                values.extend(collect_string_leaves(expr));
            }
        }
        Statement::BlockStatement(block) => {
            collect_returns_from_function_body(&block.body, values);
        }
        _ => {}
    }
}

fn binding_ident_name(pattern: &oxc_ast::ast::BindingPattern<'_>) -> Option<String> {
    match pattern {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}
