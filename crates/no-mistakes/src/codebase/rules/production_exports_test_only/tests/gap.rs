use super::super::traversal::incomplete_sources;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::{ModuleImport, ModuleSpan, TypeScriptModuleFacts};
use crate::codebase::ts_symbols::{Export, ExportKind, FileSymbols, NamedImport};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn path(name: &str) -> PathBuf {
    PathBuf::from(format!("/tmp/nm-2797-gap/{name}"))
}

fn named(source: &str, imported: &str, line: u32) -> NamedImport {
    NamedImport {
        source: source.to_owned(),
        imported: imported.to_owned(),
        local: imported.to_owned(),
        line,
        is_type_only: false,
    }
}

fn value_export(name: &str, line: u32) -> Export {
    Export {
        name: name.to_owned(),
        local: None,
        kind: ExportKind::Const,
        line,
        is_type_only: false,
    }
}

fn reexport(name: &str, source: &str, imported: &str, line: u32) -> Export {
    Export {
        name: name.to_owned(),
        local: None,
        kind: ExportKind::ReExport {
            source: source.to_owned(),
            imported: imported.to_owned(),
        },
        line,
        is_type_only: false,
    }
}

fn extracted(specifier: &str, line: u32) -> ExtractedImport {
    ExtractedImport {
        specifier: specifier.to_owned(),
        kind: ImportKind::Static,
        line,
        function_scope: None,
        function_scope_id: None,
        side_effect_only: true,
        re_export: false,
        runtime_reachable: false,
        computed: false,
    }
}

fn symbols(imports: Vec<NamedImport>, exports: Vec<Export>) -> Option<Arc<FileSymbols>> {
    Some(Arc::new(FileSymbols { imports, exports }))
}

fn module_import(specifier: &str, start: u32) -> ModuleImport {
    ModuleImport {
        specifier: specifier.to_owned(),
        type_only: false,
        span: ModuleSpan {
            start,
            end: start + 1,
        },
        bindings: Vec::new(),
    }
}

/// Symbol rows the parser would have recorded are omitted on purpose. These
/// edges are the fallbacks a saved fixture cannot build: no outgoing edges,
/// an unknown symbol matched by name, a re-export after a value export, and
/// specifier sites that exist only on the import or binding lists.
#[test]
fn module_gap_messages_cover_symbol_reexport_and_binding_sites() {
    let quiet = path("quiet.mts");
    let named_file = path("named.mts");
    let reexport_file = path("reexport.mts");
    let bare = path("bare.mts");
    let side = path("side.mts");
    let bound = path("bound.mts");
    let bound_missing_source = path("bound-missing-source.mts");
    let resolved = PathBuf::from("/abs/pkg.mts");
    let other = PathBuf::from("/abs/other.mts");
    let missing = PathBuf::from("/abs/x.mts");

    let mut facts = CheckFactMap {
        files: vec![
            quiet.clone(),
            named_file.clone(),
            reexport_file.clone(),
            bare.clone(),
            side.clone(),
            bound.clone(),
            bound_missing_source.clone(),
        ],
        ..Default::default()
    };
    let rows = [
        (
            quiet.clone(),
            TsFileFacts {
                symbols: symbols(Vec::new(), Vec::new()),
                ..Default::default()
            },
        ),
        (
            named_file.clone(),
            TsFileFacts {
                symbols: symbols(vec![named("@scope/pkg", "absent", 3)], Vec::new()),
                ..Default::default()
            },
        ),
        (
            reexport_file.clone(),
            TsFileFacts {
                symbols: symbols(
                    Vec::new(),
                    vec![
                        value_export("local", 1),
                        reexport("gone", "./other.mjs", "gone", 4),
                    ],
                ),
                ..Default::default()
            },
        ),
        (
            bare.clone(),
            TsFileFacts {
                symbols: symbols(
                    vec![named("./x.mjs", "other", 2)],
                    vec![value_export("local", 1)],
                ),
                ..Default::default()
            },
        ),
        (
            side.clone(),
            TsFileFacts {
                symbols: symbols(vec![named("./x.mjs", "other", 2)], Vec::new()),
                imports: vec![extracted("./side.mjs", 6)],
                ..Default::default()
            },
        ),
        (
            bound.clone(),
            TsFileFacts {
                symbols: symbols(Vec::new(), Vec::new()),
                source: Some(Arc::<str>::from("\nimport \"./bound.mjs\";\n")),
                module_bindings: Some(Arc::new(TypeScriptModuleFacts {
                    imports: vec![module_import("./bound.mjs", 1)],
                    ..Default::default()
                })),
                ..Default::default()
            },
        ),
        (
            bound_missing_source.clone(),
            TsFileFacts {
                symbols: symbols(Vec::new(), Vec::new()),
                module_bindings: Some(Arc::new(TypeScriptModuleFacts {
                    imports: vec![module_import("./unlocated.mjs", 0)],
                    ..Default::default()
                })),
                ..Default::default()
            },
        ),
    ];
    for (file, ts) in rows {
        facts.ts.insert(
            file,
            Arc::new(CheckFileFacts {
                ts: Arc::new(ts),
                ..Default::default()
            }),
        );
    }

    let mut forward = crate::fx::FxHashMap::from_iter([
        edge(
            &named_file,
            NodeId::module(format!("{}#absent", resolved.display())),
        ),
        edge(
            &reexport_file,
            NodeId::module(format!("{}#gone", other.display())),
        ),
        edge(
            &bare,
            NodeId::module(format!("{}#missing", missing.display())),
        ),
        edge(&side, NodeId::module("./side.mjs")),
        edge(&bound, NodeId::module("./bound.mjs")),
        edge(&bound_missing_source, NodeId::module("./unlocated.mjs")),
    ]);
    // Import-only/manual graphs can also retain an unresolved re-export module
    // without symbol provenance, so its source specifier must locate the site.
    forward
        .get_mut(&NodeId::file(&reexport_file))
        .unwrap()
        .push((NodeId::module("./other.mjs"), EdgeKind::SymbolImportUnknown));
    let found = incomplete_sources(
        &from_typed_maps(path("."), forward, Default::default()),
        &facts,
    );

    assert!(found.iter().all(|row| row.0 != quiet), "{found:?}");
    assert_gap(&found, &named_file, 3, "unresolved symbol `absent`");
    assert_gap(&found, &named_file, 3, "from `@scope/pkg`");
    assert_gap(&found, &reexport_file, 4, "unresolved symbol `gone`");
    assert_gap(&found, &reexport_file, 4, "from `./other.mjs`");
    assert_gap(
        &found,
        &reexport_file,
        4,
        "unresolved module selector `./other.mjs`",
    );
    assert_gap(&found, &bare, 1, "unresolved symbol `missing`");
    assert_gap(&found, &bare, 1, "from `/abs/x.mts`");
    assert_gap(&found, &side, 6, "unresolved module selector `./side.mjs`");
    assert_gap(
        &found,
        &bound,
        2,
        "unresolved module selector `./bound.mjs`",
    );
    assert_gap(
        &found,
        &bound_missing_source,
        1,
        "unresolved module selector `./unlocated.mjs`",
    );
}

fn edge(file: &Path, target: NodeId) -> (NodeId, Vec<(NodeId, EdgeKind)>) {
    (
        NodeId::file(file),
        vec![(target, EdgeKind::SymbolImportUnknown)],
    )
}

fn assert_gap(found: &[(PathBuf, usize, String)], file: &Path, line: usize, message: &str) {
    assert!(
        found
            .iter()
            .any(|row| row.0 == file && row.1 == line && row.2.contains(message)),
        "missing {message} at {line} in {found:?}"
    );
}

#[test]
fn manual_unknown_edges_without_source_evidence_report_the_missing_provenance() {
    let root = Path::new("/tmp/nm-2797-gap");
    let graph = from_typed_maps(root.to_path_buf(), Default::default(), Default::default());
    let unknown = std::collections::BTreeSet::from([(
        path("consumer.mts"),
        NodeId::file(path("source.mts")),
    )]);
    let (file, line, reason) =
        super::super::selector_advisories::for_export(root, &graph, &unknown, "value").unwrap();
    assert_eq!(file, path("consumer.mts"));
    assert_eq!(line, 1);
    assert!(reason.contains("consumer.mts"));
    assert!(reason.contains("missing prepared selector provenance"));
    assert!(super::super::selector_advisories::for_export(
        root,
        &graph,
        &Default::default(),
        "value"
    )
    .is_none());
}
