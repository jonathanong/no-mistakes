use crate::codebase::ts_resolver;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ImportDeclarationSpecifier, ModuleExportName, Program, Statement};
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
include!("reachability.rs");
include!("facts.rs");
include!("create_queue.rs");
include!("queue_name.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
