use super::consumers::Consumed;
use super::env::EnvWrites;
use super::escapes::Escapes;
use super::imports::ImportedWrites;
use super::setups::ConfiguredSetups;
use super::Owners;
use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{
    DepGraph, NodeId, ResolvedCallSite, ResolvedCallTarget, TsFactLookup,
};
use crate::fx::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

pub(super) struct NoFacts;

impl TsFactLookup for NoFacts {
    fn get_ts_facts(&self, _: &Path) -> Option<&crate::codebase::ts_source::facts::TsFileFacts> {
        None
    }
}

pub(super) struct Fixture<'a> {
    pub(super) graph: &'a DepGraph,
    pub(super) escaped: Escapes,
    pub(super) consumed: Consumed,
    pub(super) setups: ConfiguredSetups,
    pub(super) callers: FxHashMap<NodeId, Vec<&'a ResolvedCallSite>>,
    pub(super) nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>>,
    pub(super) reached: FxHashMap<PathBuf, bool>,
}

pub(super) fn empty_graph() -> DepGraph {
    from_typed_maps(
        PathBuf::from("/repo"),
        FxHashMap::default(),
        FxHashMap::default(),
    )
}

pub(super) fn known_setups() -> ConfiguredSetups {
    ConfiguredSetups::new(Path::new("/repo"), None, &[])
}

pub(super) fn blank(graph: &DepGraph) -> Fixture<'_> {
    Fixture {
        graph,
        escaped: Escapes::default(),
        consumed: Consumed::default(),
        setups: known_setups(),
        callers: FxHashMap::default(),
        nodes: FxHashMap::default(),
        reached: FxHashMap::default(),
    }
}

pub(super) fn call_site(file: &Path, caller: &str, id: u32) -> ResolvedCallSite {
    ResolvedCallSite {
        file: file.to_path_buf(),
        caller: Some(caller.to_string()),
        caller_id: Some(CallableId(id)),
        line: 4,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: caller.to_string(),
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

#[test]
fn a_second_entry_lookup_is_a_cache_hit() {
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let target = NodeId::callable("/repo/src/fn.mts", "fn", CallableId(1));
    assert!(owners.entries(target.clone()).is_none());
    assert!(owners.entries(target).is_none());
    let memo = owners.entry_memo.borrow();
    assert_eq!(memo.walks, 1);
    assert_eq!(memo.hits, 1);
}

#[test]
fn scan_stats_print_when_resolve_debug_is_set() {
    let previous = std::env::var_os("NO_MISTAKES_SCAN_RESOLVE_DEBUG");
    std::env::set_var("NO_MISTAKES_SCAN_RESOLVE_DEBUG", "1");
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    owners.report_scan_stats();
    restore_debug(previous);
}

fn restore_debug(previous: Option<std::ffi::OsString>) {
    match previous {
        Some(value) => std::env::set_var("NO_MISTAKES_SCAN_RESOLVE_DEBUG", value),
        None => std::env::remove_var("NO_MISTAKES_SCAN_RESOLVE_DEBUG"),
    }
}

pub(super) fn assemble(fixture: Fixture<'_>) -> Owners<'_> {
    Owners {
        env_entries: std::cell::RefCell::new(crate::fx::FxHashMap::default()),
        callers: fixture.callers,
        env: EnvWrites::default(),
        imported: ImportedWrites::new(fixture.graph),
        setups: fixture.setups,
        fixture_dirs: Vec::new(),
        nodes: fixture.nodes,
        escaped: fixture.escaped,
        environment_consumed: fixture.consumed.clone(),
        consumed: fixture.consumed,
        graph: fixture.graph,
        test_closure: FxHashSet::default(),
        gate_roots: Vec::new(),
        reached: RefCell::new(fixture.reached),
        entry_memo: RefCell::new(super::entry_memo::EntryMemo::default()),
    }
}
