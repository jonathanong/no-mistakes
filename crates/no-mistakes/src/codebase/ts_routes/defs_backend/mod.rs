use crate::codebase::ts_source::byte_offset_to_line;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, BindingPattern, Expression, ForStatementInit, ForStatementLeft, Program, Statement,
    TemplateLiteral, VariableDeclarationKind,
};
use oxc_span::SourceType;
use std::path::PathBuf;

include!("types.rs");
include!("statements.rs");
include!("shadows.rs");
include!("routes.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
