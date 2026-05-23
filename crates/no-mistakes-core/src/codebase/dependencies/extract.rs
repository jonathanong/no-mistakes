use anyhow::Result;
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, CallExpression, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, ExportNamedDeclaration, ExportSpecifier, Expression,
    ImportDeclaration, ImportDeclarationSpecifier, ImportExpression, ModuleExportName, Program,
    TSImportType, VariableDeclarator,
};
use oxc::ast_visit::{walk, Visit};
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::collections::HashSet;
use std::path::Path;

/// The syntactic import form that produced an extracted module specifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportKind {
    /// Static value import/re-export, including side-effect imports.
    Static,
    /// Type-only import/re-export or TypeScript `import("...")` type reference.
    Type,
    /// Runtime dynamic `import("...")`.
    Dynamic,
    /// CommonJS `require("...")` call.
    Require,
}

/// An extracted import specifier with syntax metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedImport {
    pub specifier: String,
    pub kind: ImportKind,
    pub function_scope: Option<String>,
}

/// A statically visible function call in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub caller: Option<String>,
    pub callee: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportFacts {
    pub imports: Vec<ExtractedImport>,
    pub function_calls: Vec<FunctionCall>,
    pub exported_functions: Vec<String>,
    pub unknown_callers: Vec<Option<String>>,
    pub has_unknown_top_level_call: bool,
}

/// Holds parser configuration for TypeScript or TSX extraction.
pub struct ImportExtractor {
    is_tsx: bool,
}

impl ImportExtractor {
    pub fn for_typescript() -> Result<Self> {
        Ok(Self { is_tsx: false })
    }

    pub fn for_tsx() -> Result<Self> {
        Ok(Self { is_tsx: true })
    }

    /// Extract import/export specifier strings from `source`, tagging each
    /// with the syntax form that created the dependency.
    pub fn extract(&self, source: &str) -> Result<Vec<ExtractedImport>> {
        let allocator = Allocator::default();
        let source_type = if self.is_tsx {
            SourceType::tsx()
        } else {
            SourceType::ts()
        };
        let ret = Parser::new(&allocator, source, source_type).parse();

        Ok(extract_import_facts_from_program(&ret.program).imports)
    }
}

pub fn extract_imports_from_program<'a>(program: &Program<'a>) -> Vec<ExtractedImport> {
    extract_import_facts_from_program(program).imports
}

pub fn extract_import_facts_from_program<'a>(program: &Program<'a>) -> ImportFacts {
    let mut collector = ImportCollector::default();
    collector.visit_program(program);
    let mut exported_functions: Vec<_> = collector.exported_functions.into_iter().collect();
    exported_functions.sort();
    ImportFacts {
        imports: collector.imports,
        function_calls: collector.function_calls,
        exported_functions,
        unknown_callers: collector.unknown_callers,
        has_unknown_top_level_call: collector.has_unknown_top_level_call,
    }
}

include!("extract_visit.rs");

fn binding_identifier_name<'a>(pattern: &'a oxc::ast::ast::BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        oxc::ast::ast::BindingPattern::BindingIdentifier(identifier) => {
            Some(identifier.name.as_str())
        }
        _ => None,
    }
}

fn simple_callee_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        Expression::ParenthesizedExpression(parenthesized) => {
            simple_callee_name(&parenthesized.expression)
        }
        _ => None,
    }
}

fn import_declaration_kind(import: &ImportDeclaration<'_>) -> ImportKind {
    if import.import_kind.is_type()
        || all_named_specifiers_are_type(import.specifiers.as_deref().map(|v| &**v))
    {
        ImportKind::Type
    } else {
        ImportKind::Static
    }
}

fn export_named_declaration_kind(export: &ExportNamedDeclaration<'_>) -> ImportKind {
    if export.export_kind.is_type() || all_export_specifiers_are_type(&export.specifiers) {
        ImportKind::Type
    } else {
        ImportKind::Static
    }
}

fn all_named_specifiers_are_type(specifiers: Option<&[ImportDeclarationSpecifier<'_>]>) -> bool {
    let Some(specifiers) = specifiers else {
        return false;
    };
    !specifiers.is_empty()
        && specifiers.iter().all(|spec| {
            matches!(
                spec,
                ImportDeclarationSpecifier::ImportSpecifier(s) if s.import_kind.is_type()
            )
        })
}

fn all_export_specifiers_are_type(specifiers: &[ExportSpecifier<'_>]) -> bool {
    !specifiers.is_empty() && specifiers.iter().all(|s| s.export_kind.is_type())
}

fn module_export_name_name<'a>(name: &'a ModuleExportName<'a>) -> Option<&'a str> {
    if let ModuleExportName::IdentifierReference(identifier) = name {
        Some(identifier.name.as_str())
    } else {
        None
    }
}

fn is_require_callee(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(ident) if ident.name == "require")
}

fn string_literal_expr<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

fn string_literal_argument<'a>(arg: &'a Argument<'a>) -> Option<&'a str> {
    match arg {
        Argument::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

/// Returns `true` for `.tsx` / `.jsx` files (which need the TSX grammar).
pub fn is_tsx_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("tsx" | "jsx")
    )
}

/// Returns `true` for any TypeScript/JavaScript source file we should index.
pub fn is_indexable(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts" | "mts" | "tsx" | "cts" | "js" | "mjs" | "jsx" | "cjs")
    )
}

#[cfg(test)]
mod tests;
