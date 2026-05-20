use crate::codebase::ts_source::byte_offset_to_line;
use anyhow::{bail, Result};
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    BindingPattern, Declaration, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, ExportNamedDeclaration, ImportDeclaration,
    ImportDeclarationSpecifier, Program, Statement, VariableDeclarationKind,
};
use oxc::parser::Parser;
use oxc::span::SourceType;
include!("ts_symbols/types.rs");
include!("ts_symbols/imports.rs");
include!("ts_symbols/export_named.rs");
include!("ts_symbols/export_other.rs");

#[cfg(test)]
mod tests;
