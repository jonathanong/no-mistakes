mod collected;
use super::super::helpers;
use super::super::relative::{PendingRelativeCall, PendingRelativeSpan};
use super::super::scoped_bindings::ScopedExecutors;
use super::executors::MemberQueryProof;
use super::resolve;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use std::collections::{BTreeMap, HashMap, HashSet};
mod collect;
use collected::CollectedCalls;
mod fragment;
pub(crate) use collect::collect_calls;
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
    pub(crate) head_calls_only: bool,
    pub(crate) source: &'a str,
    pub(crate) configured_module: String,
    pub(crate) query_members: bool,
    pub(crate) query_binding: bool,
    pub(crate) bindings: &'a HashSet<String>,
    pub(in crate::codebase::postgres::embedded) scoped: &'a ScopedExecutors,
    pub(crate) provisional: &'a [PendingRelativeSpan],
    pub(crate) scopes: Vec<HashMap<String, BindingState>>,
    pub(super) undeclared_scopes:
        Vec<HashMap<String, super::super::scoped_bindings::UndeclaredReceiverTypes>>,
    pub(super) undeclared_executor_types: Vec<super::super::UndeclaredExecutorType>,
    pub(super) executor_methods: Vec<String>,
    pub(crate) relative_proofs: Vec<HashMap<String, u32>>,
    pub(crate) namespace_proofs: Vec<HashSet<String>>,
    pub(crate) imported_namespace_names: HashMap<String, String>,
    pub(crate) imported_value_origins: HashMap<String, (String, String)>,
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
    pub(crate) unproven_member_query_spans: Vec<(u32, u32)>,
    pub(super) literal_objects: super::const_literals::LiteralObjects,
    pub(super) declaring_const: bool,
    pub(super) clause_unions: crate::fx::FxHashSet<(usize, String)>,
    pub(crate) exported_constants:
        crate::fx::FxHashMap<String, super::super::helpers::ExportedConstant>,
}
