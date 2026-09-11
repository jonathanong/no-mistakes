use super::EmbeddedSqlOptions;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind,
    ModuleExportName, Program, Statement,
};
use std::collections::HashSet;

const TRANSACTION_IMPORTS: &[&str] = &["withTransaction", "withTransactionOptions"];
const QUERY_PROPERTY: &str = "query";
const SQL_TEMPLATE_STRINGS: &str = "sql-template-strings";
const SQL_STATEMENT: &str = "SQLStatement";

pub(super) fn sql_statement_type_bindings(program: &Program<'_>) -> HashSet<String> {
    let mut bindings = HashSet::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.source.value.as_str() != SQL_TEMPLATE_STRINGS {
            continue;
        }
        let Some(specifiers) = &import.specifiers else {
            continue;
        };
        for specifier in specifiers {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                continue;
            };
            if module_export_name(&named.imported) == SQL_STATEMENT {
                bindings.insert(named.local.name.to_string());
            }
        }
    }
    bindings
}

/// Local identifiers bound as SQL executors by the configured specifier.
pub fn executor_bindings(program: &Program<'_>, options: &EmbeddedSqlOptions) -> HashSet<String> {
    let mut bindings = HashSet::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        collect_import_bindings(import, options, &mut bindings);
    }
    bindings
}

fn collect_import_bindings(
    import: &ImportDeclaration<'_>,
    options: &EmbeddedSqlOptions,
    bindings: &mut HashSet<String>,
) {
    if import.import_kind == ImportOrExportKind::Type
        || import.source.value.as_str() != options.import_specifier
    {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
            continue;
        };
        if named.import_kind == ImportOrExportKind::Type {
            continue;
        }
        let imported = module_export_name(&named.imported);
        if TRANSACTION_IMPORTS.contains(&imported.as_str()) {
            bindings.insert(QUERY_PROPERTY.to_string());
        }
        if options.executor_names.iter().any(|name| name == &imported) {
            bindings.insert(named.local.name.to_string());
        }
    }
}

fn module_export_name(name: &ModuleExportName<'_>) -> String {
    name.name().as_str().to_string()
}

/// True when `call` is a bound executor or a `.query` member call.
pub fn is_database_call(call: &CallExpression<'_>, bindings: &HashSet<String>) -> bool {
    callee_name(call, bindings).is_some()
}

pub(super) fn callee_name(call: &CallExpression<'_>, bindings: &HashSet<String>) -> Option<String> {
    match unwrap_ts_wrappers(&call.callee) {
        Expression::Identifier(ident) if bindings.contains(ident.name.as_str()) => {
            Some(ident.name.to_string())
        }
        Expression::StaticMemberExpression(member) if member.property.name == QUERY_PROPERTY => {
            Some(QUERY_PROPERTY.to_string())
        }
        Expression::ComputedMemberExpression(member) => {
            static_query_key(&member.expression).then(|| QUERY_PROPERTY.to_string())
        }
        _ => None,
    }
}

fn static_query_key(expr: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => literal.value == QUERY_PROPERTY,
        Expression::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            template.quasis[0]
                .value
                .cooked
                .as_ref()
                .map(|cooked| cooked.as_str())
                .unwrap_or(template.quasis[0].value.raw.as_str())
                == QUERY_PROPERTY
        }
        _ => false,
    }
}
