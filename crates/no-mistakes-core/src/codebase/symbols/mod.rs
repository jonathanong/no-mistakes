//! `symbols` binary: dump the named exports and imports of one or more TS/JS files.
//!
//! Wraps `crate::codebase::ts_symbols::extract_symbols` and renders the result as JSON
//! (default for non-TTY), YAML, Markdown, paths, or a human tree.
//!
//! Re-export `source` and import `source` specifiers are resolved through
//! `crate::codebase::ts_resolver` to project-relative paths when possible, so an agent
//! can follow the chain without a second tool invocation.

pub mod output;

use anyhow::{Context, Result};
use clap::Parser;
use is_terminal::IsTerminal;
use rayon::prelude::*;
use std::io;
use std::path::{Path, PathBuf};

pub use crate::codebase::dependencies::Format;
use crate::codebase::ts_resolver::{find_tsconfig, load_tsconfig, resolve_import, TsConfig};
use crate::codebase::ts_symbols::{extract_symbols, Export, ExportKind, FileSymbols, NamedImport};

include!("types.rs");
include!("resolve.rs");
include!("pipeline.rs");
include!("entry.rs");
include!("filters.rs");

#[cfg(test)]
mod tests;
