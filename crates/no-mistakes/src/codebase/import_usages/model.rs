use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportUsage {
    pub specifier: String,
    pub package_name: Option<String>,
    pub kind: &'static str,
    pub line: u32,
    pub side_effect_only: bool,
    pub re_export: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImportUsageFile {
    pub path: String,
    pub imports: Vec<ImportUsage>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImportUsagesReport {
    pub roots: Vec<String>,
    pub files: Vec<ImportUsageFile>,
}

pub fn import_usage(import: &ExtractedImport) -> ImportUsage {
    ImportUsage {
        specifier: import.specifier.clone(),
        package_name: package_name_from_specifier(&import.specifier),
        kind: kind_str(import.kind),
        line: import.line,
        side_effect_only: import.side_effect_only,
        re_export: import.re_export,
    }
}

pub(crate) fn kind_str(kind: ImportKind) -> &'static str {
    match kind {
        ImportKind::Static => "static",
        ImportKind::Type => "type",
        ImportKind::Dynamic => "dynamic",
        ImportKind::Require => "require",
        ImportKind::RequireResolve => "require-resolve",
    }
}

pub(crate) fn package_name_from_specifier(specifier: &str) -> Option<String> {
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || specifier.starts_with('#')
        || specifier.starts_with("node:")
        || specifier.contains("://")
        || specifier.starts_with("data:")
    {
        return None;
    }
    let mut parts = specifier.split('/');
    let first = parts.next()?.trim();
    if first.is_empty() {
        return None;
    }
    if first.starts_with('@') {
        let package = parts.next()?.trim();
        if package.is_empty() {
            return None;
        }
        return Some(format!("{first}/{package}"));
    }
    Some(first.to_string())
}
