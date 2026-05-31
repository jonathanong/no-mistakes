use anyhow::Result;
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, BindingPattern, BlockStatement, CallExpression, CatchClause, Class, ClassElement,
    ExportAllDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind,
    ExportNamedDeclaration, ExportSpecifier, Expression, FormalParameters, IdentifierReference,
    ImportDeclaration, ImportDeclarationSpecifier, ImportExpression, JSXOpeningElement,
    MethodDefinition, ModuleExportName, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    Program, Statement, StaticMemberExpression, TSEnumDeclaration, TSImportType,
    TSInterfaceDeclaration, TSQualifiedName, TSTypeAliasDeclaration, TSTypeName, TSTypeParameter,
    TSTypeParameterDeclaration, TSTypeReference, VariableDeclaration, VariableDeclarationKind,
    VariableDeclarator,
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
    pub side_effect_only: bool,
}

/// A statically visible function call in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub caller: Option<String>,
    pub callee: String,
    pub static_arg: Option<String>,
    pub static_cwd: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportFacts {
    pub imports: Vec<ExtractedImport>,
    pub function_calls: Vec<FunctionCall>,
    pub symbol_references: Vec<FunctionCall>,
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
    collector
        .exported_functions
        .extend(later_named_value_exports(program));
    collector
        .later_exported_type_names
        .extend(later_named_type_exports(program));
    collector.visit_program(program);
    let callable_scopes = collector.callable_scopes;
    let exported_type_scopes = collector.exported_type_scopes;
    let mut exported_functions: Vec<_> = collector
        .exported_functions
        .into_iter()
        .filter(|scope| callable_scopes.contains(scope) || exported_type_scopes.contains(scope))
        .collect();
    exported_functions.sort();
    ImportFacts {
        imports: collector.imports,
        function_calls: collector.function_calls,
        symbol_references: collector.symbol_references,
        exported_functions,
        unknown_callers: collector.unknown_callers,
        has_unknown_top_level_call: collector.has_unknown_top_level_call,
    }
}

fn later_named_value_exports<'a>(program: &Program<'a>) -> Vec<String> {
    let mut exports = Vec::new();
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() || export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() {
                continue;
            }
            if let Some(name) = module_export_name_name(&specifier.local) {
                exports.push(name.to_string());
            }
        }
    }
    exports
}

fn later_named_type_exports<'a>(program: &Program<'a>) -> Vec<String> {
    let mut exports = Vec::new();
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() || !export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if let Some(name) = module_export_name_name(&specifier.local) {
                exports.push(name.to_string());
            }
        }
    }
    exports
}

include!("extract_visit.rs");
include!("extract_collector_methods.rs");
include!("extract_visit_aggregates.rs");
include!("extract_visit_object_references.rs");
include!("extract_visit_helpers.rs");
include!("extract_default_helpers.rs");
include!("extract_object_scope_helpers.rs");
include!("extract_type_scope_helpers.rs");
include!("extract_visit_hoist.rs");
include!("extract_visit_types.rs");
include!("extract_binding_helpers.rs");
include!("extract_syntax_helpers.rs");

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
mod coverage_tests;
#[cfg(test)]
mod extra_tests;
#[cfg(test)]
mod tests;
