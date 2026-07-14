use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ExportNamedDeclaration, Expression, ObjectPropertyKind, Program, PropertyKey,
    Statement, TryStatement,
};
use oxc_span::SourceType;
use std::path::{Path, PathBuf};
include!("ts_process_spawn/types.rs");
include!("ts_process_spawn/statements.rs");
include!("ts_process_spawn/expressions.rs");
include!("ts_process_spawn/web_server.rs");
include!("ts_process_spawn/literals.rs");
include!("ts_process_spawn/resolve.rs");

#[cfg(test)]
mod tests;
