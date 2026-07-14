use crate::codebase::ts_source::byte_offset_to_line;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, FunctionBody,
    ImportDeclarationSpecifier, ObjectPropertyKind, Program, Statement,
};
use oxc_span::SourceType;
use std::collections::HashMap;
include!("types.rs");
include!("statements.rs");
include!("expressions.rs");

#[cfg(test)]
mod tests;
