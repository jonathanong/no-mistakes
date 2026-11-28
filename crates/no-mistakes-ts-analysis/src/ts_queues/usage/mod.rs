use crate::text::byte_offset_to_line;
mod model;
pub use model::{EnqueueCall, QueueUsage, WorkerDeclaration};
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

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
