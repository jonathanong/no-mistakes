use crate::codebase::ts_source::byte_offset_to_line;
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, BindingPattern, Expression, ForStatementInit, ForStatementLeft,
    ImportDeclarationSpecifier, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild,
    JSXElement, JSXExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
    TemplateLiteral, VariableDeclarationKind,
};
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::collections::HashSet;
use std::path::Path;
include!("types.rs");
include!("import_bindings.rs");
include!("scope_bindings.rs");
include!("shadowing.rs");
include!("statements.rs");
include!("expressions.rs");
include!("patterns.rs");

#[cfg(test)]
mod tests;
