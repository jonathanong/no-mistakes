//! Shared AST traversal helpers for guardrails rules that inspect JSX and/or
//! expressions inside TSX sources.
//!
//! Each helper walks the program once via a `Visitor` trait. Rules implement
//! the hooks they care about and leave the rest as no-ops.

use oxc::ast::ast::{
    Argument, ArrayExpressionElement, ChainExpression, ClassBody, ClassElement, Declaration,
    ExportDefaultDeclarationKind, Expression, ForStatement, ForStatementInit, FunctionBody,
    ImportDeclaration, JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement, JSXExpression,
    JSXOpeningElement, ObjectPropertyKind, Program, Statement, TryStatement,
};

include!("visitor.rs");
include!("statements.rs");
include!("expressions.rs");
include!("helpers.rs");

#[cfg(test)]
mod tests;
