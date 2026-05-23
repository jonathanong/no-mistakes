
/// For human-friendly output formats (md, human), prefer the resolved
/// project-relative path so an agent can chase the import without re-resolving
/// the specifier. Falls back to the raw specifier when resolution failed (bare
/// npm packages, etc.).
fn display_source(resolved: &Option<PathBuf>, fallback: &str) -> String {
    resolved
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

// ── Shared serializable shape ────────────────────────────────────────────
//
// One owned struct family powers both JSON and YAML. Owning the strings
// (rather than borrowing) costs an extra allocation per field at emit time
// but cuts the duplication that previously embedded near-identical structs
// inside `write_yml`.

#[derive(Serialize)]
struct ReExportInfo {
    source: String,
    imported: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved: Option<String>,
}

#[derive(Serialize)]
struct ExportEntry {
    name: String,
    kind: &'static str,
    line: u32,
    #[serde(skip_serializing_if = "Option::is_none", rename = "reExport")]
    re_export: Option<ReExportInfo>,
}

#[derive(Serialize)]
struct ImportEntry {
    source: String,
    imported: String,
    local: String,
    line: u32,
    #[serde(rename = "typeOnly")]
    type_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved: Option<String>,
}

#[derive(Serialize)]
struct FileOutput {
    path: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exports: Vec<ExportEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    imports: Vec<ImportEntry>,
}

#[derive(Serialize)]
struct Output {
    roots: Vec<String>,
    files: Vec<FileOutput>,
}

fn export_to_entry(e: &ResolvedExport) -> ExportEntry {
    let re_export = match &e.kind {
        ExportKind::ReExport { source, imported } => Some(ReExportInfo {
            source: source.clone(),
            imported: imported.clone(),
            resolved: e.resolved.as_ref().map(|p| p.display().to_string()),
        }),
        _ => None,
    };
    ExportEntry {
        name: e.name.clone(),
        kind: export_kind_str(&e.kind),
        line: e.line,
        re_export,
    }
}

fn import_to_entry(i: &ResolvedImport) -> ImportEntry {
    ImportEntry {
        source: i.source.clone(),
        imported: i.imported.clone(),
        local: i.local.clone(),
        line: i.line,
        type_only: i.is_type_only,
        resolved: i.resolved.as_ref().map(|p| p.display().to_string()),
    }
}

fn build_output(roots: &[String], entries: &[FileEntry]) -> Output {
    Output {
        roots: roots.to_vec(),
        files: entries
            .iter()
            .map(|e| FileOutput {
                path: e.rel_path.display().to_string(),
                exports: e.exports.iter().map(export_to_entry).collect(),
                imports: e.imports.iter().map(import_to_entry).collect(),
            })
            .collect(),
    }
}
