use crate::codebase::ts_source::byte_offset_to_line;
use anyhow::{bail, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportAllDeclaration, ExportDeclaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, ExportFromDeclaration, ExportNamedDeclaration, ImportDeclaration,
    ImportDeclarationSpecifier, Program, Statement, VariableDeclarationKind,
};
use oxc_span::SourceType;
use std::collections::HashSet;
include!("ts_symbols/types.rs");
include!("ts_symbols/imports.rs");
include!("ts_symbols/export_named.rs");
include!("ts_symbols/export_other.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
