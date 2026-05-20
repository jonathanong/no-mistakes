use crate::codebase::ts_routes::refs::normalize_template;
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, Expression, ForStatement, ForStatementInit, FunctionBody, Statement,
};
use oxc::parser::Parser;
use oxc::span::SourceType;

include!("statements.rs");
include!("expressions.rs");
include!("helpers.rs");

#[cfg(test)]
mod tests;
