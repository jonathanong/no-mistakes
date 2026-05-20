use super::{export_kind_str, FileEntry, ResolvedExport, ResolvedImport};
use crate::codebase::ts_symbols::ExportKind;
use anyhow::Result;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;

include!("output/types.rs");
include!("output/structured.rs");
include!("output/text.rs");
