use crate::codebase::ts_symbols::{Export, FileSymbols};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UniqueExportsOptions {
    pub unique_across_types_and_values: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UniqueExportFinding {
    pub rule: String,
    pub file: String,
    pub line: u32,
    pub export_name: String,
    pub export_kind: String,
    pub message: String,
}

/// Internal aggregate-check sidecar; the public finding remains six fields.
#[doc(hidden)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PreparedUniqueExportFinding {
    pub finding: UniqueExportFinding,
    pub suppression_source_location: Option<(String, u32)>,
}

#[derive(Debug, Clone)]
pub(super) struct SourceFile {
    pub(super) path: PathBuf,
    pub(super) rel: String,
    pub(super) source: String,
    pub(super) symbols: std::sync::Arc<FileSymbols>,
    pub(super) disabled: bool,
    pub(super) defer_suppression: bool,
    pub(super) is_nextjs_project: bool,
    pub(super) is_remix_route_module: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum ExportBucket {
    Type,
    Value,
    Any,
}

impl ExportBucket {
    pub(super) fn from_export(export: &Export) -> Self {
        if export.is_type_only {
            Self::Type
        } else {
            Self::Value
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Value => "value",
            Self::Any => "export",
        }
    }

    pub(super) fn key(self, strict: bool) -> Self {
        if strict {
            Self::Any
        } else {
            self
        }
    }

    pub(super) fn message_label(self) -> &'static str {
        match self {
            Self::Type => "type export",
            Self::Value => "value export",
            Self::Any => "export",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ExportOccurrence {
    pub(super) name: String,
    pub(super) bucket: ExportBucket,
    pub(super) file: String,
    pub(super) line: u32,
    pub(super) kind: String,
    pub(super) origin: ExportOrigin,
    /// Deferred aggregate analysis needs this to keep a suppressed occurrence
    /// from becoming the canonical export for a visible duplicate.
    pub(super) suppressed: bool,
    /// The source location whose directive suppressed this occurrence. Origin
    /// directives must remain auditable even when a re-export is the duplicate.
    pub(super) suppression_location: Option<(String, u32)>,
}

#[derive(Debug, Clone)]
pub(super) struct ExportOrigin {
    pub(super) file: String,
    pub(super) line: u32,
    pub(super) name: String,
    pub(super) bucket: ExportBucket,
    pub(super) suppressed: bool,
    pub(super) suppression_location: Option<(String, u32)>,
}

impl ExportOrigin {
    fn identity(&self) -> (&str, u32, &str, ExportBucket) {
        (&self.file, self.line, &self.name, self.bucket)
    }
}

impl PartialEq for ExportOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl Eq for ExportOrigin {}

impl PartialOrd for ExportOrigin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExportOrigin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.identity().cmp(&other.identity())
    }
}
