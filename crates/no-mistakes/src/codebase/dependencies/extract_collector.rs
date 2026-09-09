#[derive(Default)]
struct ImportCollector {
    /// 0-based byte offsets of each line start. Empty when line numbers are
    /// unused so the collector does not retain a full source copy.
    line_starts: Vec<u32>,
    imports: Vec<ExtractedImport>,
    function_calls: Vec<FunctionCall>,
    unknown_calls: Vec<UnknownCall>,
    symbol_references: Vec<FunctionCall>,
    function_stack: Vec<String>,
    function_id_stack: Vec<CallableId>,
    callable_scope_ids: HashSet<(CallableId, String)>,
    /// Static class members have a display scope derived from the class name,
    /// which is not unique across sibling lexical scopes. Preserve their
    /// parser-owned identity next to their owning class binding.
    class_member_callable_ids: HashSet<(CallableId, String, CallableId)>,
    /// Every class method's identity, used only to invalidate the exact
    /// reassigned lexical binding without promoting instance methods to static
    /// graph edges.
    aggregate_callable_member_ids: HashSet<(CallableId, String, CallableId)>,
    static_getter_member_ids: HashSet<(CallableId, String)>,
    static_setter_member_ids: HashSet<(CallableId, String)>,
    syntactic_caller_stack: Vec<String>,
    local_stack: Vec<HashSet<String>>,
    /// Stable identities parallel to `local_stack`. Scope depth alone is not
    /// an identity: sibling/nested blocks can reuse a depth while shadowing.
    lexical_scope_ids: Vec<usize>,
    /// Lexical parent for each stable scope identity. Alias resolution needs
    /// this rather than display scope strings: sibling functions can share a
    /// display name while still binding different outer aliases.
    lexical_scope_parents: HashMap<usize, Option<usize>>,
    next_lexical_scope_id: usize,
    type_local_stack: Vec<HashSet<String>>,
    type_parameter_stack: Vec<HashSet<String>>,
    /// Local-stack indices that own `var` declarations. Function bodies and
    /// class static blocks each establish an independent var environment.
    var_scope_stack: Vec<usize>,
    exported_functions: HashSet<String>,
    exported_resource_roots: HashSet<String>,
    exported_resource_scopes: HashSet<String>,
    collect_resource_roots: bool,
    exported_type_scopes: HashSet<String>,
    callable_scopes: HashSet<String>,
    class_scopes: HashSet<String>,
    export_depth: usize,
    has_unknown_top_level_call: bool,
    anonymous_scope_count: usize,
    known_function_scopes: HashSet<String>,
    imported_bindings: HashSet<String>,
    predeclared_imported_bindings: HashSet<String>,
    call_import_bindings: Vec<ImportedBinding>,
    call_export_bindings: Vec<ExportedBinding>,
    callable_aliases: Vec<CallableAliasBinding>,
    callable_binding_ids: HashSet<(usize, String)>,
    callable_bindings: HashMap<(usize, String), CallableId>,
    reassigned_callable_binding_ids: HashSet<(usize, String)>,
    star_reexport_specifiers: Vec<String>,
    suppress_imports: bool,
    collect_suppressed_runtime_imports: bool,
    /// `function_stack` depth captured at the start of an exported binding
    /// initializer / default-export expression. A runtime (`import()`/`require()`)
    /// import exactly one function level below this depth — the callback directly
    /// forming the exported value, e.g. `dynamic(() => import('./Foo'))` — is
    /// flagged reachable. Deeper, uninvoked nested imports fall back to normal
    /// call-scope reachability so they are not falsely kept.
    runtime_reachable_base_depth: Option<usize>,
    later_exported_type_names: HashSet<String>,
}
