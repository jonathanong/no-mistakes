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
    /// Non-literal `import()`/`require()` specifier. Literal-only consumers skip
    /// these; `resolve-check` reports them unresolved instead of omitting them.
    pub computed: bool,
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

include!("extract_kinds.rs");
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
/// bindings. A later assignment retains the alias only for source positions
/// before its invalidation cutoff.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallableAlias {
    pub scope: Option<String>,
    pub scope_id: Option<CallableId>,
    pub local: String,
    pub target: String,
    /// The lexical binding identity of `local`.
    pub binding_scope: usize,
    /// Byte offset of the alias declaration. Calls in the same lexical frame
    /// before this position are in the temporal dead zone and must not resolve.
    pub declared_at: u32,
    /// Byte offset where an assignment invalidates this otherwise immutable
    /// alias. Calls before that source position still have the original
    /// target; later calls must not resolve through it.
    pub invalidated_at: Option<u32>,
}

/// Private binding identity used while extracting callable aliases. Public
/// alias facts are function/module scoped, but invalidation must also retain
/// the lexical frame so an inner shadow cannot invalidate an outer alias.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CallableAliasBinding {
    alias: CallableAlias,
    lexical_scope_depth: usize,
}
