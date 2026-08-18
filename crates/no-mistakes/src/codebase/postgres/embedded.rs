use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier,
    ImportOrExportKind, ModuleExportName, Program, Statement, TemplateLiteral,
};
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

mod walk;

const DEFAULT_IMPORT_SPECIFIER: &str = "@data-stores/psql";
const DEFAULT_EXECUTOR_NAMES: &[&str] = &["query", "read", "write"];
const TRANSACTION_IMPORTS: &[&str] = &["withTransaction", "withTransactionOptions"];
const QUERY_PROPERTY: &str = "query";

/// Configurable executor import matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSqlOptions {
    pub import_specifier: String,
    pub executor_names: Vec<String>,
}

impl Default for EmbeddedSqlOptions {
    fn default() -> Self {
        Self {
            import_specifier: DEFAULT_IMPORT_SPECIFIER.to_string(),
            executor_names: DEFAULT_EXECUTOR_NAMES
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
        }
    }
}

/// One executor call site and the SQL text it would execute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSqlCall {
    pub line: u32,
    pub callee: String,
    pub sql_text: Option<String>,
}

/// Embedded-SQL facts for one TypeScript/JavaScript file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSqlFileFacts {
    pub path: PathBuf,
    pub executor_bindings: Vec<String>,
    pub calls: Vec<EmbeddedSqlCall>,
}

/// Parse `source` and extract executor SQL call sites.
pub fn extract_embedded_sql_from_source(
    path: &Path,
    source: &str,
    options: &EmbeddedSqlOptions,
) -> EmbeddedSqlFileFacts {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = crate::ast::parse(path, &allocator, source, source_type);
    extract_embedded_sql_from_program(path, &parsed.program, source, options)
}

/// Extract executor SQL call sites from an already-parsed program.
pub fn extract_embedded_sql_from_program(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    options: &EmbeddedSqlOptions,
) -> EmbeddedSqlFileFacts {
    let bindings = executor_bindings(program, options);
    let mut executor_bindings: Vec<String> = bindings.iter().cloned().collect();
    executor_bindings.sort();
    let calls = walk::collect_calls(program, source, &bindings);
    EmbeddedSqlFileFacts {
        path: path.to_path_buf(),
        executor_bindings,
        calls,
    }
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
    if import.import_kind == ImportOrExportKind::Type {
        return;
    }
    if import.source.value.as_str() != options.import_specifier {
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

fn callee_name(call: &CallExpression<'_>, bindings: &HashSet<String>) -> Option<String> {
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
            quasi_text(&template.quasis[0]) == QUERY_PROPERTY
        }
        _ => false,
    }
}

fn first_call_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        other => other.as_expression(),
    }
}

fn resolve_call_sql(
    argument: &Expression<'_>,
    bindings: &HashMap<String, String>,
) -> Option<String> {
    executed_query_text(argument, bindings)
}

/// SQL text of a literal, tagged template, or template expression.
pub fn sql_text(expr: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) => Some(template_sql_text(template)),
        Expression::TaggedTemplateExpression(tagged) => Some(template_sql_text(&tagged.quasi)),
        _ => None,
    }
}

/// `sql_text` plus identifier lookup of in-scope SQL bindings.
pub fn executed_query_text(
    expr: &Expression<'_>,
    bindings: &HashMap<String, String>,
) -> Option<String> {
    if let Some(text) = sql_text(expr) {
        return Some(text);
    }
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) => bindings.get(ident.name.as_str()).cloned(),
        _ => None,
    }
}

fn template_sql_text(template: &TemplateLiteral<'_>) -> String {
    let mut out = String::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        if index > 0 {
            out.push_str(&format!("sql_placeholder_{index}"));
        }
        out.push_str(quasi_text(quasi));
    }
    out
}

fn quasi_text<'a>(quasi: &'a oxc_ast::ast::TemplateElement<'a>) -> &'a str {
    quasi
        .value
        .cooked
        .as_ref()
        .map(|cooked| cooked.as_str())
        .unwrap_or(quasi.value.raw.as_str())
}

#[cfg(test)]
mod tests;
