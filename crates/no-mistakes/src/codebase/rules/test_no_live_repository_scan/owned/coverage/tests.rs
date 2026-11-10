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

pub(super) fn assemble(fixture: Fixture<'_>) -> Owners<'_> {
    Owners {
        callers: fixture.callers,
        env: EnvWrites::default(),
        imported: ImportedWrites::new(fixture.graph),
        setups: fixture.setups,
        fixture_dirs: Vec::new(),
        nodes: fixture.nodes,
        escaped: fixture.escaped,
        consumed: fixture.consumed,
        graph: fixture.graph,
        test_closure: FxHashSet::default(),
        gate_roots: Vec::new(),
        reached: RefCell::new(fixture.reached),
    }
}
