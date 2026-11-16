use super::alias::{self, CallerIndex};
use super::home::{self, Locate};
use super::resolve;
use crate::codebase::dependencies::extract::{
    CallableAlias, CallableId, ExportedBinding, ImportedBinding, ImportedBindingKind,
    InvocationKind,
};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::dependencies::graph::{
    DepGraph, NodeId, ResolvedCallSite, ResolvedCallTarget, TsFactLookup,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, PathSet};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

struct Facts {
    files: HashMap<PathBuf, TsFileFacts>,
}

impl TsFactLookup for Facts {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.files.get(path)
    }
}

fn graph(edges: &[(&Path, &Path)]) -> DepGraph {
    let mut forward: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut reverse: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for (from, to) in edges {
        forward
            .entry((*from).to_path_buf())
            .or_default()
            .push((*to).to_path_buf());
        reverse
            .entry((*to).to_path_buf())
            .or_default()
            .push((*from).to_path_buf());
    }
    from_raw_maps(PathBuf::from("/repo"), forward, reverse)
}

fn locate<'a>(
    graph: &'a DepGraph,
    facts: &'a Facts,
    resolution: &'a super::super::Resolution<'a>,
    visible: &'a PathSet,
) -> Locate<'a> {
    Locate {
        graph,
        facts,
        resolution,
        visible,
        resolver: resolve::RegistryResolver::prepare(resolution, visible),
    }
}

fn resolution() -> super::super::Resolution<'static> {
    super::super::Resolution {
        catalog: None,
        workspace: None,
        session: None,
    }
}

fn site(file: &Path) -> ResolvedCallSite {
    ResolvedCallSite {
        file: file.to_path_buf(),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "workers[name]".to_string(),
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        returned_member_from: Vec::new(),
        typed_from: Vec::new(),
        target_node: None,
    }
}

fn namespace_import(specifier: &str) -> ImportedBinding {
    ImportedBinding {
        specifier: specifier.to_string(),
        local: "workers".to_string(),
        imported: "*".to_string(),
        kind: ImportedBindingKind::Namespace,
        is_type_only: false,
    }
}

fn export(local: &str, exported: &str, specifier: Option<&str>) -> ExportedBinding {
    ExportedBinding {
        specifier: specifier.map(str::to_string),
        local: local.to_string(),
        exported: exported.to_string(),
    }
}

fn callable_alias(local: &str, target: &str) -> CallableAlias {
    CallableAlias {
        scope: None,
        scope_id: None,
        local: local.to_string(),
        target: target.to_string(),
        binding_scope: 0,
        declared_at: 0,
        invalidated_at: None,
    }
}

fn callers(file: &Path, id: u32, name: &str) -> CallerIndex {
    let mut ids = fx_map();
    ids.insert(CallableId(id), NodeId::callable(file, name, CallableId(id)));
    let mut index = fx_map();
    index.insert(file.to_path_buf(), ids);
    index
}

#[path = "registry_coverage_alias_tests.rs"]
mod alias_tests;
#[path = "registry_coverage_home_tests.rs"]
mod home_tests;
