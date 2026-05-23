use crate::codebase::ts_resolver;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc::allocator::Allocator;
use oxc::ast::ast::{Expression, ImportDeclarationSpecifier, ModuleExportName, Program, Statement};
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
include!("reachability.rs");
include!("create_queue.rs");
include!("queue_name.rs");

#[cfg(test)]
mod tests;
