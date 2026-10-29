use serde::Serialize;

/// Half-open UTF-8 byte offsets into the original module source.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ModuleSpan {
    pub start: u32,
    pub end: u32,
}
impl From<oxc_span::Span> for ModuleSpan {
    fn from(span: oxc_span::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingReference {
    pub span: ModuleSpan,
    pub runtime: bool,
    pub type_only: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleBinding {
    pub id: usize,
    pub name: String,
    pub scope_id: usize,
    pub span: ModuleSpan,
    pub imported: bool,
    pub runtime: bool,
    pub type_only: bool,
    pub shadows: Option<usize>,
    pub references: Vec<BindingReference>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleScope {
    pub id: usize,
    pub parent_id: Option<usize>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleImportBinding {
    pub imported: String,
    pub local: String,
    pub kind: String,
    pub type_only: bool,
    pub binding_id: usize,
    pub span: ModuleSpan,
    /// Members read through a `import type * as ns` binding, or `None` when
    /// any use of the namespace is not a plain `ns.Member` type reference.
    #[serde(skip)]
    pub(crate) namespace_members: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleImport {
    pub specifier: String,
    pub type_only: bool,
    pub span: ModuleSpan,
    pub bindings: Vec<ModuleImportBinding>,
    /// Leading export name of an `import("x").Name` type reference.
    #[serde(skip)]
    pub(crate) qualifier: Option<String>,
    /// Export names a bare-looking `typeof import("x")` provably selects. An
    /// empty list is a type assertion that reads no export. `None` is a whole
    /// module type.
    #[serde(skip)]
    pub(crate) picked: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleExport {
    pub specifier: Option<String>,
    pub local: String,
    pub exported: String,
    pub type_only: bool,
    pub span: ModuleSpan,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLoad {
    pub kind: String,
    pub specifier: String,
    pub span: ModuleSpan,
}
#[derive(Debug, Clone)]
pub(crate) struct RequireBinding {
    pub local: String,
    pub load_start: u32,
    pub scope_id: usize,
    /// A statically named destructured export; `None` means a module value.
    pub imported: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ModuleDiagnostic {
    pub kind: String,
    pub message: String,
    pub span: Option<ModuleSpan>,
}
#[derive(Debug, Clone)]
pub(crate) struct ModuleMockCall {
    pub(crate) automock: bool,
    pub(crate) hoisted: bool,
    pub(crate) specifier: Option<String>,
    pub(crate) factory_span: Option<ModuleSpan>,
    pub(crate) full: bool,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct TypeScriptModuleFacts {
    #[serde(skip)]
    pub(crate) module_mock_calls: Vec<ModuleMockCall>,
    #[serde(skip)]
    pub(crate) module_mock_actual_spans: Vec<ModuleSpan>,
    /// Potential Node-origin calls, consumed only as unresolved loader findings.
    #[serde(skip)]
    pub(crate) node_loader_calls:
        crate::fx::FxHashMap<u32, Vec<super::node_loaders::NodeLoaderCall>>,
    /// Locals initialized directly from a recognized static CommonJS load.
    #[serde(skip)]
    pub(crate) require_bindings: Vec<RequireBinding>,
    #[serde(skip)]
    pub(crate) mock_import_spans: Vec<ModuleSpan>,
    /// Byte ranges of `unsupported` diagnostics that cannot hide an importer
    /// (ambient wildcard modules, whole-file CommonJS export assignments).
    /// They stay in `diagnostics` so module completeness is unchanged.
    #[serde(skip)]
    pub(crate) importer_safe: Vec<(u32, u32)>,
    /// Literal loads through a `createRequire(import.meta.url)` binding. Only
    /// the importer rule reads them; public `loads` is unchanged.
    #[serde(skip)]
    pub(crate) created_requires: Vec<ModuleLoad>,
    /// Start offset and message of created-loader uses the importer rule
    /// cannot follow (escaping references, non-literal specifiers).
    #[serde(skip)]
    pub(crate) importer_unsafe: Vec<(u32, &'static str)>,
    /// Start offset and export names of dynamic imports with proven selectors.
    #[serde(skip)]
    pub(crate) dynamic_import_selectors: Vec<(u32, Vec<String>)>,
    #[serde(skip)]
    pub(crate) call_arguments:
        Option<std::sync::Arc<crate::fx::FxHashMap<u32, super::arguments::ArgumentValues>>>,
    #[serde(skip)]
    pub(crate) path_provenance: Option<std::sync::Arc<super::arguments::PathProvenance>>,
    pub imports: Vec<ModuleImport>,
    pub exports: Vec<ModuleExport>,
    pub bindings: Vec<ModuleBinding>,
    pub scopes: Vec<ModuleScope>,
    pub loads: Vec<ModuleLoad>,
    pub diagnostics: Vec<ModuleDiagnostic>,
}
