pub mod extract;
pub mod graph;
pub mod output;

use anyhow::{bail, Context, Result};
use is_terminal::IsTerminal;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

pub use crate::codebase::ts_resolver::TsConfig;
pub use graph::{DepGraph, EdgeKind, NodeId};

pub use crate::cli::Format;

include!("args_relationships.rs");
include!("run_pipeline.rs");
include!("traversal.rs");
include!("symbol_resolution.rs");
include!("output_args.rs");

#[cfg(test)]
mod tests;
