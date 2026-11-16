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
use crate::codebase::ts_resolver::normalize_path;
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

fn alias(local: &str, target: &str) -> CallableAlias {
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

#[test]
fn reach_memo_repeats_a_miss() {
    let graph = graph(&[]);
    let reach = alias::Reach::new(&graph, &[]);
    let node = NodeId::file("/repo/src/idle.mts");
    assert!(!reach.reaches(&node));
    assert!(!reach.reaches(&node));
}

#[test]
fn missing_caller_facts_are_not_a_registry() {
    let graph = graph(&[]);
    let facts = Facts {
        files: HashMap::new(),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    let file = Path::new("/repo/src/missing.mts");
    assert!(home::homes(&locate, &site(file), "workers").is_empty());
    assert!(home::function_exports(&locate, file).is_empty());
}

#[test]
fn binding_search_stops_at_the_hop_cap_and_on_a_cycle() {
    let edges = graph(&[]);
    let facts = Facts {
        files: HashMap::new(),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let located = locate(&edges, &facts, &resolution, &visible);
    let file = PathBuf::from("/repo/src/a.mts");
    let mut seen = vec![(file.clone(), "workers".to_string()); 8];
    assert!(home::binding_local(&located, &file, "workers", &mut seen).is_none());

    let left = PathBuf::from("/repo/src/a.mts");
    let right = PathBuf::from("/repo/src/b.mts");
    let edges = graph(&[(&left, &right), (&right, &left)]);
    let mut files = HashMap::new();
    files.insert(
        left.clone(),
        TsFileFacts {
            exported_bindings: vec![export("workers", "workers", Some("./b.mts"))],
            ..TsFileFacts::default()
        },
    );
    files.insert(
        right.clone(),
        TsFileFacts {
            exported_bindings: vec![export("workers", "workers", Some("./a.mts"))],
            ..TsFileFacts::default()
        },
    );
    let facts = Facts { files };
    let located = locate(&edges, &facts, &resolution, &visible);
    assert!(home::binding_local(&located, &left, "workers", &mut Vec::new()).is_none());
}

#[test]
fn a_namespace_reexport_is_not_an_object_registry() {
    let file = PathBuf::from("/repo/src/barrel.mts");
    let graph = graph(&[]);
    let facts = Facts {
        files: HashMap::from([(
            file.clone(),
            TsFileFacts {
                exported_bindings: vec![export("*", "workers", Some("./scan.mts"))],
                ..TsFileFacts::default()
            },
        )]),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    assert!(home::binding_local(&locate, &file, "workers", &mut Vec::new()).is_none());
}

#[test]
fn star_search_skips_a_missing_specifier_and_a_module_that_lacks_the_name() {
    let barrel = PathBuf::from("/repo/src/barrel.mts");
    let barren = PathBuf::from("/repo/src/barren.mts");
    let scan = PathBuf::from("/repo/src/scan.mts");
    let graph = graph(&[(&barrel, &barren), (&barrel, &scan)]);
    let facts = Facts {
        files: HashMap::from([
            (
                barrel.clone(),
                TsFileFacts {
                    star_reexport_specifiers: vec![
                        "./missing.mts".to_string(),
                        "./barren.mts".to_string(),
                        "./scan.mts".to_string(),
                    ],
                    ..TsFileFacts::default()
                },
            ),
            (barren.clone(), TsFileFacts::default()),
            (
                scan.clone(),
                TsFileFacts {
                    exported_bindings: vec![export("workers", "workers", None)],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    assert_eq!(
        home::binding_local(&locate, &barrel, "workers", &mut Vec::new()),
        Some((scan, "workers".to_string()))
    );
}

#[test]
fn function_export_walk_skips_non_callables_and_stops_on_a_star_cycle() {
    let scan = PathBuf::from("/repo/src/scan.mts");
    let looped = PathBuf::from("/repo/src/loop.mts");
    let graph = graph(&[(&scan, &looped), (&looped, &scan)]);
    let facts = Facts {
        files: HashMap::from([
            (
                scan.clone(),
                TsFileFacts {
                    exported_functions: vec!["marker".to_string(), "sync".to_string()],
                    callable_scope_ids: vec![(CallableId(7), "sync".to_string())],
                    star_reexport_specifiers: vec!["./loop.mts".to_string()],
                    ..TsFileFacts::default()
                },
            ),
            (
                looped.clone(),
                TsFileFacts {
                    star_reexport_specifiers: vec!["./scan.mts".to_string()],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    assert_eq!(
        home::function_exports(&locate, &scan),
        vec![(scan, "sync".to_string())]
    );
}

#[test]
fn a_classified_path_without_facts_uses_the_matching_direct_import() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from = root.join("src/lib.rs");
    let cargo = normalize_path(&root.join("Cargo.toml"));
    let other = root.join("src/main.rs");
    let resolution = resolution();
    let visible = PathSet::default();

    let linked = graph(&[(&from, &cargo)]);
    let facts = Facts {
        files: HashMap::new(),
    };
    let linked_locate = locate(&linked, &facts, &resolution, &visible);
    assert_eq!(
        resolve::resolve(&linked_locate, &from, "../Cargo.toml"),
        Some(cargo.clone())
    );

    let unlinked = graph(&[(&from, &other)]);
    let unlinked_locate = locate(&unlinked, &facts, &resolution, &visible);
    assert_eq!(
        resolve::resolve(&unlinked_locate, &from, "../Cargo.toml"),
        None
    );
    assert_eq!(
        resolve::resolve(&unlinked_locate, &from, "./no-such-registry-target.mts"),
        None
    );
}

#[test]
fn a_namespace_const_key_selects_an_exported_function_or_its_alias() {
    let caller = PathBuf::from("/repo/src/test.mts");
    let scan = PathBuf::from("/repo/src/scan.mts");
    let graph = graph(&[(&caller, &scan)]);
    let resolution = resolution();
    let visible = PathSet::default();
    let site = site(&caller);
    let caller_facts = TsFileFacts {
        imported_bindings: vec![namespace_import("./scan.mts")],
        ..TsFileFacts::default()
    };

    let function = Facts {
        files: HashMap::from([
            (caller.clone(), caller_facts.clone()),
            (
                scan.clone(),
                TsFileFacts {
                    exported_bindings: vec![export("sync", "sync", None)],
                    callable_scope_ids: vec![(CallableId(7), "sync".to_string())],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let function_locate = locate(&graph, &function, &resolution, &visible);
    let node = NodeId::callable(&scan, "sync", CallableId(7));
    assert_eq!(
        alias::function_node(
            &function_locate,
            &site,
            "workers",
            "sync",
            &callers(&scan, 7, "sync")
        ),
        Some(node)
    );

    let aliased = Facts {
        files: HashMap::from([
            (caller, caller_facts),
            (
                scan.clone(),
                TsFileFacts {
                    exported_bindings: vec![export("sync", "sync", None)],
                    callable_aliases: vec![alias("sync", "processSync")],
                    callable_scope_ids: vec![(CallableId(9), "processSync".to_string())],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let alias_locate = locate(&graph, &aliased, &resolution, &visible);
    let node = NodeId::callable(&scan, "processSync", CallableId(9));
    assert_eq!(
        alias::function_node(
            &alias_locate,
            &site,
            "workers",
            "sync",
            &callers(&scan, 9, "processSync")
        ),
        Some(node)
    );
}
