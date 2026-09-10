use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, Program, TemplateLiteral};
use oxc_span::SourceType;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod bindings;
mod options;
mod placeholders;
mod tags;
mod walk;

pub use bindings::{executor_bindings, is_database_call};
pub use options::EmbeddedSqlOptions;

/// One executor call site and its recovered SQL text. For `Dynamic` calls,
/// `sql_text` can be only a verified leading statement rather than complete SQL.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmbeddedSqlCall {
    pub line: u32,
    pub callee: String,
    pub sql_text: Option<String>,
    pub kind: EmbeddedSqlKind,
    pub declaration_line: Option<u32>,
}

/// How executed SQL was recovered from TypeScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmbeddedSqlKind {
    #[default]
    Inline,
    ImmutableLocal,
    Composed,
    Dynamic,
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

pub(super) fn first_call_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        other => other.as_expression(),
    }
}

/// SQL text of a literal, tagged template, or template expression.
pub fn sql_text(expr: &Expression<'_>) -> Option<String> {
    unpublished_sql_text(expr).map(placeholders::publish_placeholders)
}

pub(super) fn unpublished_sql_text(expr: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) => Some(template_sql_text(template, false)),
        Expression::TaggedTemplateExpression(tagged) => {
            let use_raw = tags::is_string_raw_tag_spelling(&tagged.tag);
            Some(template_sql_text(&tagged.quasi, use_raw))
        }
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

/// `use_raw` selects `String.raw`'s own runtime semantics (the literal
/// source characters, escape sequences un-processed) over the cooked form
/// every other tag — the trusted `sql` tag included — returns; see
/// [`tags::is_string_raw_tag_spelling`].
fn template_sql_text(template: &TemplateLiteral<'_>, use_raw: bool) -> String {
    let mut out = String::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        if index > 0 {
            out.push_str(&placeholders::internal_placeholder(index));
        }
        out.push_str(quasi_text(quasi, use_raw));
    }
    out
}

fn quasi_text<'a>(quasi: &'a oxc_ast::ast::TemplateElement<'a>, use_raw: bool) -> &'a str {
    if use_raw {
        return quasi.value.raw.as_str();
    }
    quasi
        .value
        .cooked
        .as_ref()
        .map(|cooked| cooked.as_str())
        .unwrap_or(quasi.value.raw.as_str())
}

#[cfg(test)]
mod append_mutation_tests;
#[cfg(test)]
mod chain_composition_tests;
#[cfg(test)]
mod chain_reassignment_destructuring_tests;
#[cfg(test)]
mod chain_tests;
#[cfg(test)]
mod compose_classification_tests;
#[cfg(test)]
mod imported_sql_tag_tests;
#[cfg(test)]
mod resolution_gaps_tests;
#[cfg(test)]
mod tests;
