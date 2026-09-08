use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AccessorProperty, Argument, AssignmentExpression, AssignmentTarget,
    AssignmentTargetMaybeDefault, AssignmentTargetProperty, BindingPattern, BlockStatement,
    CallExpression, CatchClause, Class, ClassElement, Declaration, ExportAllDeclaration,
    ExportDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind,
    ExportFromDeclaration, ExportNamedDeclaration, ExportSpecifier, Expression, ForStatementLeft,
    FormalParameters, IdentifierReference, ImportDeclaration, ImportDeclarationSpecifier,
    ImportExpression, JSXOpeningElement, MethodDefinition, MethodDefinitionKind, ModuleExportName,
    NewExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind, Program,
    PropertyDefinition, Statement, StaticBlock, StaticMemberExpression, TSEnumDeclaration,
    TSImportType, TSInterfaceDeclaration, TSQualifiedName, TSTypeAliasDeclaration, TSTypeName,
    TSTypeParameter, TSTypeParameterDeclaration, TSTypeReference, TaggedTemplateExpression,
    VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Opaque source-local identity for a callable owner.
///
/// The byte offset is deliberately never rendered.  It is stable for the two
/// parser-owned collectors that need to agree on ownership, while display
/// names remain the source-level scope strings carried beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CallableId(pub u32);

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
    /// CommonJS `require.resolve("...")` call.
    RequireResolve,
}

/// An extracted import specifier with syntax metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedImport {
    pub specifier: String,
    pub kind: ImportKind,
    pub line: u32,
    pub function_scope: Option<String>,
    pub function_scope_id: Option<CallableId>,
    pub side_effect_only: bool,
    pub re_export: bool,
    /// `true` for a runtime (`import()`/`require()`) import collected from inside
    /// an exported binding initializer, where the enclosing callback is never
    /// statically called (e.g. `next/dynamic(() => import('./Foo'))`). Such
    /// imports are reachable through the exported binding even though no static
    /// call reaches their anonymous scope.
    pub runtime_reachable: bool,
}

/// A statically visible function call in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    /// The lexical callable scope containing the invocation, when known.
    pub caller: Option<String>,
    pub caller_id: Option<CallableId>,
    /// The nearest source-level function owner used for source-occurrence
    /// reports. Unlike [`Self::caller`], this preserves the unqualified
    /// syntactic name and does not invent owners for anonymous callbacks or
    /// property/class methods.
    pub syntactic_caller: Option<String>,
    /// The source spelling of the callee. This deliberately preserves aliases;
    /// resolution belongs to the prepared graph layer.
    pub callee: String,
    /// One-based source line of the invocation.
    pub line: u32,
    /// Zero-based source byte where the invocation starts. Synthetic and
    /// reference-only facts use zero because they do not represent an AST call.
    pub offset: u32,
    /// Whether this is the synthetic callback edge used to preserve anonymous
    /// callback reachability, rather than a JavaScript call expression.
    pub is_callback: bool,
    /// The JavaScript invocation form. Synthetic callback edges retain their
    /// own kind so callers can exclude them from source-level policy checks.
    pub invocation: InvocationKind,
    /// Binding classification captured during the same AST pass. The callee
    /// spelling remains available for exact and terminal-name policies.
    pub target_identity: CallTargetIdentity,
    /// Identity of the lexical frame which owns the callee's first segment.
    /// This distinguishes a block-local shadow from an alias owned by the
    /// surrounding callable scope.
    pub callee_binding_scope: Option<usize>,
    pub static_arg: Option<String>,
    pub static_cwd: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvocationKind {
    Call,
    Construct,
    Callback,
    /// Synthetic aggregate membership used by import reachability. This is
    /// not a JavaScript invocation and must never become a call edge.
    Membership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallTargetIdentity {
    Global,
    ModuleExport,
    RepositoryFunction,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCall {
    pub caller: Option<String>,
    pub caller_id: Option<CallableId>,
    pub line: u32,
    /// Zero-based source byte where the unresolved invocation starts.
    pub offset: u32,
    pub invocation: InvocationKind,
}

/// A runtime import binding.  This is deliberately separate from
/// [`ExtractedImport`]: dependency edges only need the module specifier, while
/// call resolution must retain the local binding and exported name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedBinding {
    pub specifier: String,
    pub local: String,
    pub imported: String,
    pub kind: ImportedBindingKind,
    pub is_type_only: bool,
}

/// The syntactic shape of an import binding. Only a namespace binding can
/// statically resolve `binding.member()`; named/default imports are values, not
/// module objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedBindingKind {
    Named,
    Default,
    Namespace,
}

/// A runtime export binding. `specifier` is present for a named re-export and
/// absent when the export names a local binding in the same module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedBinding {
    pub specifier: Option<String>,
    pub local: String,
    pub exported: String,
}

/// An immutable local value alias whose initializer is a statically named
/// callable. The scope is the lexical function owner, or `None` for module
/// bindings. Mutable declarations and bindings observed on an assignment LHS
/// are deliberately omitted so call resolution never guesses their value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallableAlias {
    pub scope: Option<String>,
    pub scope_id: Option<CallableId>,
    pub local: String,
    pub target: String,
    /// The lexical binding identity of `local`.
    pub binding_scope: usize,
}

/// Private binding identity used while extracting callable aliases. Public
/// alias facts are function/module scoped, but invalidation must also retain
/// the lexical frame so an inner shadow cannot invalidate an outer alias.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CallableAliasBinding {
    alias: CallableAlias,
    lexical_scope_depth: usize,
}

include!("extract_import_facts.rs");

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
        let sentinel = if self.is_tsx {
            "imports.tsx"
        } else {
            "imports.ts"
        };
        let ret = crate::ast::parse(Path::new(sentinel), &allocator, source, source_type);

        Ok(
            extract_import_facts_from_program_with_source_and_resource_roots(
                &ret.program,
                source,
                false,
            )
            .imports,
        )
    }
}

include!("extract_entrypoints.rs");
include!("extract_entrypoints_predeclare.rs");
include!("extract_export_names.rs");
include!("extract_collector.rs");
include!("extract_visit.rs");
include!("extract_visit_modules.rs");
include!("extract_visit_exports.rs");
include!("extract_collector_scopes.rs");
include!("extract_collector_methods.rs");
include!("extract_visit_members.rs");
include!("extract_visit_aggregates.rs");
include!("extract_class_heritage_helpers.rs");
include!("extract_class_eager_helpers.rs");
include!("extract_class_callable_helpers.rs");
include!("extract_visit_object_references.rs");
include!("extract_collector_aliases.rs");
include!("extract_visit_helpers.rs");
include!("extract_visit_variables.rs");
include!("extract_control_flow_scopes.rs");
include!("extract_default_helpers.rs");
include!("extract_object_scope_helpers.rs");
include!("extract_resource_scopes.rs");
include!("extract_type_scope_helpers.rs");
include!("extract_visit_hoist.rs");
include!("extract_visit_types.rs");
include!("extract_binding_names.rs");
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
mod import_metadata_tests;
#[cfg(test)]
mod tests;
