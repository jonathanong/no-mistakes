/** Explicit files only; no package resolution or whole-project traversal. */
export interface TypeScriptModulesOptions {
  root?: string;
  files: string[];
}
/** Half-open UTF-8 byte offsets into the original source, not JS UTF-16 indexes. */
export interface TypeScriptModuleSpan {
  start: number;
  end: number;
}
export interface TypeScriptBindingReference {
  span: TypeScriptModuleSpan;
  runtime: boolean;
  typeOnly: boolean;
}
export interface TypeScriptModuleBinding {
  /** Module-local stable identifier; do not compare IDs across modules or runs. */
  id: number;
  name: string;
  scopeId: number;
  span: TypeScriptModuleSpan;
  imported: boolean;
  /** A syntactically declared runtime value, without resolving imported modules. */
  runtime: boolean;
  typeOnly: boolean;
  shadows: number | null;
  references: TypeScriptBindingReference[];
}
export interface TypeScriptModuleScope {
  id: number;
  parentId: number | null;
}
export type TypeScriptImportBindingKind = "default" | "named" | "namespace";
export interface TypeScriptModuleImportBinding {
  imported: string;
  local: string;
  kind: TypeScriptImportBindingKind;
  typeOnly: boolean;
  bindingId: number;
  span: TypeScriptModuleSpan;
}
export interface TypeScriptModuleImport {
  specifier: string;
  typeOnly: boolean;
  span: TypeScriptModuleSpan;
  /** Empty for side-effect imports. */
  bindings: TypeScriptModuleImportBinding[];
}
export interface TypeScriptModuleExport {
  specifier: string | null;
  /** "*" for star or namespace re-exports; empty for anonymous default exports. */
  local: string;
  exported: string;
  typeOnly: boolean;
  span: TypeScriptModuleSpan;
}
export type TypeScriptModuleLoadKind = "dynamicImport" | "require";
export interface TypeScriptModuleLoad {
  kind: TypeScriptModuleLoadKind;
  specifier: string;
  span: TypeScriptModuleSpan;
}
export type TypeScriptModuleDiagnosticKind =
  | "sourceError"
  | "parseError"
  | "semanticError"
  | "unsupported";
export interface TypeScriptModuleDiagnostic {
  kind: TypeScriptModuleDiagnosticKind;
  message: string;
  span: TypeScriptModuleSpan | null;
}
export interface TypeScriptModuleFacts {
  fileName: string;
  /** False for source/read/parser errors and explicitly unsupported dynamic forms. */
  complete: boolean;
  imports: TypeScriptModuleImport[];
  exports: TypeScriptModuleExport[];
  bindings: TypeScriptModuleBinding[];
  scopes: TypeScriptModuleScope[];
  loads: TypeScriptModuleLoad[];
  diagnostics: TypeScriptModuleDiagnostic[];
}
export interface TypeScriptModulesReport {
  modules: TypeScriptModuleFacts[];
}
