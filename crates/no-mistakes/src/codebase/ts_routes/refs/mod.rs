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
use oxc::syntax::operator::BinaryOperator;
use std::collections::{HashMap, HashSet};
use std::path::Path;
include!("types.rs");
include!("import_bindings.rs");
include!("scope_bindings.rs");
include!("shadowing.rs");
include!("statements.rs");
include!("expressions.rs");
include!("patterns.rs");
include!("helper_bindings.rs");
include!("helper_patterns.rs");
include!("helper_patterns_aliases.rs");
include!("helper_patterns_assign.rs");
include!("helper_patterns_env.rs");
include!("helper_patterns_eval.rs");
include!("helper_patterns_call.rs");
include!("helper_patterns_returns.rs");
include!("helper_patterns_switch.rs");
include!("helper_patterns_template.rs");
include!("helper_patterns_try.rs");
include!("helper_patterns_utils.rs");
include!("helper_refs.rs");
include!("helper_refs_import_wrappers.rs");
include!("helper_refs_statements.rs");
include!("helper_refs_statement_bindings.rs");
include!("helper_refs_statement_control.rs");
include!("helper_refs_statement_for.rs");
include!("helper_refs_statement_declarations.rs");
include!("helper_refs_statement_functions.rs");
include!("helper_refs_statement_switch.rs");
include!("helper_refs_statement_try.rs");
include!("helper_refs_expressions.rs");
include!("helper_refs_expression_calls.rs");
include!("helper_refs_expression_scopes.rs");
include!("helper_refs_expression_objects.rs");
include!("helper_refs_expression_wrappers.rs");
include!("helper_refs_jsx.rs");
include!("helper_refs_context.rs");

#[cfg(test)]
mod tests;
