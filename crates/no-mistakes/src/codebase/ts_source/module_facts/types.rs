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
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleImport {
    pub specifier: String,
    pub type_only: bool,
    pub span: ModuleSpan,
    pub bindings: Vec<ModuleImportBinding>,
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
#[derive(Debug, Clone, Serialize)]
pub struct ModuleDiagnostic {
    pub kind: String,
    pub message: String,
    pub span: Option<ModuleSpan>,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct TypeScriptModuleFacts {
    pub imports: Vec<ModuleImport>,
    pub exports: Vec<ModuleExport>,
    pub bindings: Vec<ModuleBinding>,
    pub scopes: Vec<ModuleScope>,
    pub loads: Vec<ModuleLoad>,
    pub diagnostics: Vec<ModuleDiagnostic>,
}
