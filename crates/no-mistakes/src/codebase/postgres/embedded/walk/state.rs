use super::super::bindings::sql_statement_type_bindings;
use super::super::options::TrustedSqlTag;
use super::super::relative::{PendingRelativeCall, PendingRelativeSpan};
use super::super::scoped_bindings::ScopedExecutors;
use super::resolve;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct BindingState {
    pub(crate) sql: Option<String>,
    pub(crate) kind: EmbeddedSqlKind,
    pub(crate) line: u32,
    pub(crate) initialized: bool,
    pub(crate) sql_builder: bool,
    pub(crate) sql_source_positions: Vec<super::super::EmbeddedSqlSourcePosition>,
}

pub(crate) struct ScopeVisitor<'a> {
    pub(crate) source: &'a str,
    pub(crate) query_members: bool,
    pub(crate) bindings: &'a HashSet<String>,
    pub(crate) scoped: &'a ScopedExecutors,
    pub(crate) provisional: &'a [PendingRelativeSpan],
    pub(crate) scopes: Vec<HashMap<String, BindingState>>,
    pub(crate) calls: Vec<EmbeddedSqlCall>,
    pub(crate) call_spans: Vec<(u32, u32)>,
    pub(crate) pending_spans: BTreeMap<u32, (u32, u32)>,
    pub(crate) pending_calls: Vec<PendingRelativeCall>,
    pub(crate) confirmed_order: Vec<u32>,
    pub(crate) next_seq: u32,
    pub(crate) track_order: bool,
    pub(crate) fragments: Vec<EmbeddedSqlFragment>,
    pub(crate) suppress_nested_builder_fragments: usize,
    pub(crate) control_depth: usize,
    pub(crate) loop_depth: usize,
    pub(crate) function_scopes: Vec<usize>,
    pub(crate) functions: resolve::LocalFunctions,
    pub(crate) sql_statement_types: HashSet<String>,
}
pub(crate) struct CollectedCalls {
    pub(crate) calls: Vec<EmbeddedSqlCall>,
    pub(crate) call_spans: Vec<(u32, u32)>,
    pub(crate) pending_spans: BTreeMap<u32, (u32, u32)>,
    pub(crate) fragments: Vec<EmbeddedSqlFragment>,
    pub(crate) pending_calls: Vec<PendingRelativeCall>,
    pub(crate) confirmed_order: Vec<u32>,
}

pub(crate) fn collect_calls<'a>(
    program: &Program<'a>,
    source: &'a str,
    bindings: &'a HashSet<String>,
    scoped: &'a ScopedExecutors,
    provisional: &'a [PendingRelativeSpan],
    query_members: bool,
    trusted_sql_tags: &[TrustedSqlTag],
) -> CollectedCalls {
    let track_order = !provisional.is_empty();
    let mut visitor = ScopeVisitor {
        source,
        query_members,
        bindings,
        scoped,
        provisional,
        scopes: Vec::new(),
        calls: Vec::new(),
        call_spans: Vec::new(),
        pending_spans: BTreeMap::new(),
        pending_calls: Vec::new(),
        confirmed_order: Vec::new(),
        next_seq: 0,
        track_order,
        fragments: Vec::new(),
        suppress_nested_builder_fragments: 0,
        control_depth: 0,
        loop_depth: 0,
        function_scopes: Vec::new(),
        functions: resolve::LocalFunctions::collect(program, source, trusted_sql_tags),
        sql_statement_types: sql_statement_type_bindings(program),
    };
    visitor.visit_program(program);
    CollectedCalls {
        calls: visitor.calls,
        call_spans: visitor.call_spans,
        pending_spans: visitor.pending_spans,
        fragments: visitor.fragments,
        pending_calls: visitor.pending_calls,
        confirmed_order: visitor.confirmed_order,
    }
}

impl ScopeVisitor<'_> {
    pub(super) fn push_fragment(&mut self, line: u32, sql_text: Option<String>) {
        self.fragments.push(EmbeddedSqlFragment {
            line,
            sql_text,
            recovered_placeholder_positions: Vec::new(),
        });
    }
}
