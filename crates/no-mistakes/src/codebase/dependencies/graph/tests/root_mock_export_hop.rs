use super::super::{
    follow, module_id, owned_hop, Chain, DepGraph, ExternalMock, GraphBuildPlan, GraphFiles, Hop,
    MockedModule, TsFactLookup,
};
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::ts_resolver::{normalize_path, ImportClassification, ImportResolution};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::{ModuleSpan, TypeScriptModuleFacts};
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::fx_set;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

struct FixedResolver {
    visible: PathBuf,
    hidden: PathBuf,
    left: PathBuf,
    right: PathBuf,
    broken: PathBuf,
}

impl ImportResolution for FixedResolver {
    fn resolve(&self, specifier: &str, _: &Path) -> Option<PathBuf> {
        match specifier {
            "visible.mts" => Some(self.visible.clone()),
            "hidden.mts" => Some(self.hidden.clone()),
            "./cycle-a.mts" => Some(self.left.clone()),
            "./cycle-b.mts" => Some(self.right.clone()),
            "broken.mts" => Some(self.broken.clone()),
            _ => None,
        }
    }

    fn resolution_candidates(&self, _: &str, _: &Path) -> BTreeSet<PathBuf> {
        BTreeSet::new()
    }

    fn visible_files(&self) -> Option<&dyn crate::codebase::ts_resolver::VisiblePathLookup> {
        None
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &IndexedWorkspaceMap,
        _: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
    ) -> ImportClassification {
        ImportClassification::unresolved()
    }

    fn matches_paths_alias(&self, _: &str, _: &Path) -> bool {
        false
    }
}

struct MappedFacts {
    broken: TsFileFacts,
    left: TsFileFacts,
    right: TsFileFacts,
    broken_path: PathBuf,
    left_path: PathBuf,
    right_path: PathBuf,
}

impl TsFactLookup for MappedFacts {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        if path == self.broken_path {
            Some(&self.broken)
        } else if path == self.left_path {
            Some(&self.left)
        } else if path == self.right_path {
            Some(&self.right)
        } else {
            None
        }
    }
}

fn path(name: &str) -> PathBuf {
    normalize_path(&PathBuf::from("/tmp/nm-grok-1-mock").join(name))
}

fn export_from(specifier: &str, name: &str) -> ExportedBinding {
    ExportedBinding {
        specifier: Some(specifier.to_string()),
        local: name.to_string(),
        exported: name.to_string(),
    }
}

#[test]
fn owned_hop_misses_an_unimported_value_and_two_value_imports() {
    let mut facts = TsFileFacts::default();
    facts.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "helper".into(),
        exported: "helper".into(),
    });
    assert!(matches!(owned_hop(&facts, "helper"), Hop::Miss));
    facts.imported_bindings.push(ImportedBinding {
        specifier: "./a.mts".into(),
        local: "helper".into(),
        imported: "helper".into(),
        kind: ImportedBindingKind::Named,
        is_type_only: false,
    });
    facts.imported_bindings.push(ImportedBinding {
        specifier: "./b.mts".into(),
        local: "helper".into(),
        imported: "helper".into(),
        kind: ImportedBindingKind::Named,
        is_type_only: false,
    });
    assert!(matches!(owned_hop(&facts, "helper"), Hop::Uncertain));
}

#[test]
fn file_mock_follow_covers_external_cycles_and_parse_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/production-exports-only-used-by-tests/fixture/class-extends-production",
        ),
    );
    let graph = DepGraph::build_with_plan(
        &root,
        &crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..crate::codebase::ts_resolver::TsConfig::default()
        },
        GraphBuildPlan::default(),
    )
    .unwrap();
    let visible = path("visible.mts");
    let hidden = path("hidden.mts");
    let left = path("cycle-a.mts");
    let right = path("cycle-b.mts");
    let broken = path("broken.mts");
    let resolver = FixedResolver {
        visible: visible.clone(),
        hidden: hidden.clone(),
        left: left.clone(),
        right: right.clone(),
        broken: broken.clone(),
    };
    assert!(resolver
        .resolution_candidates("hidden.mts", &visible)
        .is_empty());
    assert!(resolver.visible_files().is_none());
    assert!(!resolver.matches_paths_alias("hidden.mts", &visible));
    let files = GraphFiles::from_files(vec![
        visible.clone(),
        left.clone(),
        right.clone(),
        broken.clone(),
    ]);
    let _ = resolver.classify_import(
        "hidden.mts",
        &visible,
        &IndexedWorkspaceMap::default(),
        &files,
    );
    let module = TypeScriptModuleFacts::default();
    let span = ModuleSpan { start: 0, end: 4 };
    let empty_facts = MappedFacts {
        broken: TsFileFacts::default(),
        left: TsFileFacts::default(),
        right: TsFileFacts::default(),
        broken_path: broken.clone(),
        left_path: left.clone(),
        right_path: right.clone(),
    };
    let external = |specifier: Option<&str>| {
        graph.external_replaced_module(ExternalMock {
            specifier,
            factory_span: Some(&span),
            factories: &[],
            root: Path::new("/tmp/nm-mock-root.mts"),
            facts: &empty_facts,
            resolver: &resolver,
            files: &files,
            module: &module,
            ready: true,
        })
    };
    assert!(external(None).is_none());
    assert!(external(Some("visible.mts")).is_none());
    let hidden_mock = external(Some("hidden.mts"));
    assert!(matches!(
        hidden_mock,
        Some((MockedModule::File(ref file), true)) if file == &hidden
    ));
    let broken_facts = TsFileFacts {
        parse_error: Some("broken".into()),
        ..TsFileFacts::default()
    };
    let left_facts = TsFileFacts {
        exported_bindings: vec![export_from("./cycle-b.mts", "again")],
        ..TsFileFacts::default()
    };
    let right_facts = TsFileFacts {
        exported_bindings: vec![export_from("./cycle-a.mts", "again")],
        ..TsFileFacts::default()
    };
    let facts = MappedFacts {
        broken: broken_facts,
        left: left_facts,
        right: right_facts,
        broken_path: broken.clone(),
        left_path: left.clone(),
        right_path: right.clone(),
    };
    assert!(facts.get_ts_facts(Path::new("/tmp/missing.mts")).is_none());
    let chain = Chain {
        modules: &fx_set(),
        facts: &facts,
        resolver: &resolver,
        files: &files,
    };
    assert!(matches!(
        module_id(&chain, "hidden.mts", Path::new("/tmp/from.mts")),
        MockedModule::File(ref file) if file == &hidden
    ));
    assert_eq!(
        follow(
            &chain,
            Path::new("/tmp/from.mts"),
            "broken.mts",
            "localHelper",
            &mut fx_set(),
        ),
        None
    );
    assert_eq!(
        follow(
            &chain,
            Path::new("/tmp/from.mts"),
            "./cycle-a.mts",
            "again",
            &mut fx_set(),
        ),
        Some(false)
    );
}
