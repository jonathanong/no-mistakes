use super::super::bindings::sql_statement_type_bindings;
use super::super::helpers;
use super::super::options::TrustedSqlTag;
use super::super::relative::{PendingRelativeCall, PendingRelativeSpan};
use super::super::scoped_bindings::ScopedExecutors;
use super::imported;
use super::resolve;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct BindingState {
    // None: ineligible; Some(false): const awaiting its initializer; Some(true): proven.
    pub(super) executor_alias: Option<bool>,
    pub(super) builder_identity: Option<u32>,
    pub(super) builder_effect_revision: u64,
    pub(super) condition_key: Option<(u64, u32)>,
    pub(super) variants: Option<Vec<super::variants::Recovered>>,
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
    pub(in crate::codebase::postgres::embedded) scoped: &'a ScopedExecutors,
    pub(crate) provisional: &'a [PendingRelativeSpan],
    pub(crate) scopes: Vec<HashMap<String, BindingState>>,
    pub(super) builder_aliases: crate::fx::FxHashMap<String, Vec<String>>,
    pub(super) argument_effects: Vec<super::executor_arguments::ArgumentEffect>,
    pub(super) argument_frames: usize,
    pub(crate) calls: Vec<EmbeddedSqlCall>,
    pub(crate) call_spans: Vec<(u32, u32)>,
    /// Local names bound by value imports; a call to one is an imported helper.
    pub(crate) imported_names: crate::fx::FxHashSet<String>,
    /// The `const` initializer being walked, when it is a recognized IIFE.
    pub(super) iife: Option<super::iife::Iife>,
    pub(crate) imported_sites: std::cell::RefCell<helpers::Sites>,
    pub(crate) pending_spans: BTreeMap<u32, (u32, u32)>,
    pub(crate) pending_calls: Vec<PendingRelativeCall>,
    pub(crate) confirmed_order: Vec<u32>,
    pub(crate) next_seq: u32,
    pub(crate) track_order: bool,
    pub(crate) fragments: Vec<EmbeddedSqlFragment>,
    pub(crate) fragment_sites: Vec<Option<u32>>,
    pub(crate) fragment_variants: Vec<Vec<super::super::EmbeddedSqlVariant>>,
    pub(crate) suppress_nested_builder_fragments: usize,
    pub(crate) variant_paths: Vec<Vec<(u64, u32)>>,
    pub(super) variant_guard_owner: Option<usize>,
    pub(crate) control_depth: usize,
    pub(crate) loop_depth: usize,
    pub(crate) function_scopes: Vec<usize>,
    pub(crate) functions: resolve::LocalFunctions,
    pub(crate) sql_statement_types: HashSet<String>,
}
pub(crate) struct CollectedCalls {
    pub(crate) calls: Vec<EmbeddedSqlCall>,
    pub(crate) call_spans: Vec<(u32, u32)>,
    pub(crate) exported_helpers: crate::fx::FxHashMap<String, helpers::HelperSummary>,
    pub(crate) pending_imported: Vec<helpers::PendingImportedSql>,
    pub(crate) pending_spans: BTreeMap<u32, (u32, u32)>,
    pub(crate) fragments: Vec<EmbeddedSqlFragment>,
    pub(crate) fragment_sites: Vec<Option<u32>>,
    pub(crate) fragment_variants: Vec<Vec<super::super::EmbeddedSqlVariant>>,
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
        builder_aliases: crate::fx::FxHashMap::default(),
        argument_effects: Vec::new(),
        argument_frames: 0,
        calls: Vec::new(),
        call_spans: Vec::new(),
        imported_names: imported::value_import_names(program),
        iife: None,
        imported_sites: Default::default(),
        pending_spans: BTreeMap::new(),
        pending_calls: Vec::new(),
        confirmed_order: Vec::new(),
        next_seq: 0,
        track_order,
        fragments: Vec::new(),
        fragment_variants: Vec::new(),
        fragment_sites: Vec::new(),
        suppress_nested_builder_fragments: 0,
        variant_paths: vec![Vec::new()],
        variant_guard_owner: None,
        control_depth: 0,
        loop_depth: 0,
        function_scopes: Vec::new(),
        functions: resolve::LocalFunctions::collect(program, source, trusted_sql_tags),
        sql_statement_types: sql_statement_type_bindings(program),
    };
    visitor.visit_program(program);
    let sites = std::mem::take(&mut *visitor.imported_sites.borrow_mut()).into_list();
    let pending_imported = helpers::finalize::finalize(
        helpers::finalize::Walked {
            calls: &mut visitor.calls,
            call_spans: &visitor.call_spans,
            pending_calls: &mut visitor.pending_calls,
            pending_spans: &visitor.pending_spans,
            fragments: &mut visitor.fragments,
            fragment_variants: &mut visitor.fragment_variants,
        },
        &sites,
    );
    CollectedCalls {
        calls: visitor.calls,
        call_spans: visitor.call_spans,
        exported_helpers: visitor.functions.exported_helpers(),
        pending_imported,
        pending_spans: visitor.pending_spans,
        fragments: visitor.fragments,
        fragment_variants: visitor.fragment_variants,
        fragment_sites: visitor.fragment_sites,
        pending_calls: visitor.pending_calls,
        confirmed_order: visitor.confirmed_order,
    }
}

impl ScopeVisitor<'_> {
    pub(super) fn push_fragment(
        &mut self,
        line: u32,
        sql_text: Option<String>,
        expression: Option<&oxc_ast::ast::Expression<'_>>,
        append_site: Option<u32>,
    ) {
        let variants = expression
            .and_then(|expression| self.recover_variants(expression))
            .filter(|values| {
                values
                    .iter()
                    .all(|value| value.value == super::variants::ValueKind::Sql)
            })
            .unwrap_or_default()
            .into_iter()
            .map(|value| value.publish())
            .collect();
        self.fragment_variants.push(variants);
        self.fragment_sites.push(append_site);
        self.fragments.push(EmbeddedSqlFragment {
            line,
            sql_text,
            recovered_placeholder_positions: Vec::new(),
        });
    }
}
