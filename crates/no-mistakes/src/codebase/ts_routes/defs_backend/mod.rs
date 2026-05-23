use crate::codebase::ts_source::{byte_offset_to_line, is_skipped_dir};
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, BindingPattern, Expression, ForStatementInit, ForStatementLeft, Program, Statement,
    TemplateLiteral, VariableDeclarationKind,
};
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::path::PathBuf;

include!("types.rs");
include!("statements.rs");
include!("shadows.rs");
include!("routes.rs");

#[cfg(test)]
mod tests;
