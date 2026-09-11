use super::super::bindings::sql_statement_type_bindings;
use super::resolve;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct BindingState {
    pub(crate) sql: Option<String>,
    pub(crate) kind: EmbeddedSqlKind,
    pub(crate) line: u32,
    pub(crate) sql_builder: bool,
}

pub(crate) struct ScopeVisitor<'a> {
    pub(crate) source: &'a str,
    pub(crate) bindings: &'a HashSet<String>,
    pub(crate) scopes: Vec<HashMap<String, BindingState>>,
    pub(crate) calls: Vec<EmbeddedSqlCall>,
    pub(crate) fragments: Vec<EmbeddedSqlFragment>,
    pub(crate) suppress_nested_builder_fragments: usize,
    pub(crate) control_depth: usize,
    pub(crate) loop_depth: usize,
    pub(crate) function_scopes: Vec<usize>,
    pub(crate) functions: resolve::LocalFunctions,
    pub(crate) sql_statement_types: HashSet<String>,
}
pub(crate) fn collect_calls(
    program: &Program<'_>,
    source: &str,
    bindings: &HashSet<String>,
) -> (Vec<EmbeddedSqlCall>, Vec<EmbeddedSqlFragment>) {
    let mut visitor = ScopeVisitor {
        source,
        bindings,
        scopes: Vec::new(),
        calls: Vec::new(),
        fragments: Vec::new(),
        suppress_nested_builder_fragments: 0,
        control_depth: 0,
        loop_depth: 0,
        function_scopes: Vec::new(),
        functions: resolve::LocalFunctions::collect(program),
        sql_statement_types: sql_statement_type_bindings(program),
    };
    visitor.visit_program(program);
    (visitor.calls, visitor.fragments)
}
