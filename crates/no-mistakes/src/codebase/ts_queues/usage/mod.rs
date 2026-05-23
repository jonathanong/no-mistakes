use crate::codebase::ts_source::byte_offset_to_line;
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, FunctionBody,
    ImportDeclarationSpecifier, ObjectPropertyKind, Program, Statement,
};
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::collections::HashMap;
include!("types.rs");
include!("statements.rs");
include!("expressions.rs");

#[cfg(test)]
mod tests;
