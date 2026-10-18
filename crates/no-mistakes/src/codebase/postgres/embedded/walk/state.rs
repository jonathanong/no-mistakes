use super::super::bindings::sql_statement_type_bindings;
use super::super::helpers;
use super::super::options::EmbeddedSqlOptions;
use super::super::relative::{PendingRelativeCall, PendingRelativeSpan};
use super::super::scoped_bindings::ScopedExecutors;
use super::executors::MemberQueryProof;
use super::imported;
use super::resolve;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::{BTreeMap, HashMap, HashSet};
#[derive(Clone)]
pub(crate) struct BindingState {
    pub(super) builder_identity: Option<u32>,
    pub(super) builder_effect_revision: u64,
    pub(super) condition_key: Option<(u64, u32)>,
    pub(super) variants: Option<Vec<super::variants::Recovered>>,
    pub(crate) sql: Option<String>,
    pub(crate) kind: EmbeddedSqlKind,
    pub(crate) line: u32,
    // None: ineligible; Some(false): awaiting or pending proof; Some(true): proven.
    pub(super) executor_alias: Option<bool>,
    pub(crate) initialized: bool,
    pub(crate) sql_builder: bool,
    pub(crate) sql_source_positions: Vec<super::super::EmbeddedSqlSourcePosition>,
}

pub(crate) struct ScopeVisitor<'a> {
    pub(crate) source: &'a str,
    pub(crate) configured_module: String,
    pub(crate) query_members: bool,
    pub(crate) query_binding: bool,
    pub(crate) bindings: &'a HashSet<String>,
    pub(in crate::codebase::postgres::embedded) scoped: &'a ScopedExecutors,
    pub(crate) provisional: &'a [PendingRelativeSpan],
    pub(crate) scopes: Vec<HashMap<String, BindingState>>,
    pub(crate) relative_proofs: Vec<HashMap<String, u32>>,
    pub(crate) namespace_proofs: Vec<HashSet<String>>,
    pub(crate) imported_namespace_names: HashSet<String>,
    pub(crate) member_query_proof: MemberQueryProof,
    pub(super) builder_aliases: crate::fx::FxHashMap<String, Vec<String>>,
    pub(super) argument_effects: Vec<super::executor_arguments::ArgumentEffect>,
    pub(super) argument_frames: usize,
    pub(super) builder_reference_exemptions: Vec<u32>,
    /// Inner links of a fluent append chain already applied by its outer call.
    pub(super) consumed_links: crate::fx::FxHashSet<u32>,
    /// Calls that are a whole expression statement: their value is discarded.
    pub(super) statement_calls: crate::fx::FxHashSet<u32>,
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
    pub(crate) exported_programs:
        crate::fx::FxHashMap<String, std::sync::Arc<helpers::program::Program>>,
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
    options: &EmbeddedSqlOptions,
) -> CollectedCalls {
    let track_order = !provisional.is_empty();
    let imported_namespace_names =
        super::namespace_imports::configured_namespaces(program, options);
    let mut visitor = ScopeVisitor {
        source,
        configured_module: options.import_specifier.clone(),
        query_members: !options.import_specifier.is_empty()
            || options.executor_names.iter().any(|name| name == "query"),
        query_binding: options.executor_names.is_empty()
            || options.executor_names.iter().any(|name| name == "query"),
        bindings,
        scoped,
        provisional,
        scopes: Vec::new(),
        relative_proofs: Vec::new(),
        namespace_proofs: Vec::new(),
        imported_namespace_names,
        member_query_proof: MemberQueryProof::default(),
        builder_aliases: crate::fx::FxHashMap::default(),
        argument_effects: Vec::new(),
        argument_frames: 0,
        builder_reference_exemptions: Vec::new(),
        consumed_links: Default::default(),
        statement_calls: Default::default(),
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
        functions: resolve::LocalFunctions::collect(program, source, &options.trusted_sql_tags),
        sql_statement_types: sql_statement_type_bindings(program),
    };
    visitor.visit_program(program);
    visitor.drop_invalidated_member_calls();
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
        exported_programs: visitor.functions.take_programs(),
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
