use super::discover;
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::postgres::query_annotation::evaluate::File;
use crate::codebase::postgres::query_annotation::module_ids::ModuleIds;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::{Expr, QueryAnnotationFileFacts, Step};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

fn make_file<'a>(
    facts: &'a QueryAnnotationFileFacts,
    ts: TsFileFacts,
    ids: &Arc<ModuleIds>,
) -> File<'a> {
    File {
        facts,
        module: Arc::new(Module::new(Arc::new(ts)).with_module_ids(Arc::clone(ids))),
        executors: FxHashSet::default(),
    }
}

#[test]
fn referenced_deps_tracks_named_star_and_local_reexports() {
    let [r1, r2, missing, barrel_named, barrel_star, barrel_local, leaf_named, leaf_star, leaf_local] =
        [
            "r1.mts",
            "r2.mts",
            "missing.mts",
            "barrel_named.mts",
            "barrel_star.mts",
            "barrel_local.mts",
            "leaf_named.mts",
            "leaf_star.mts",
            "leaf_local.mts",
        ]
        .map(PathBuf::from);
    let ids = Arc::new(ModuleIds::new([
        r1.clone(),
        r2.clone(),
        missing.clone(),
        barrel_named.clone(),
        barrel_star.clone(),
        barrel_local.clone(),
        leaf_named.clone(),
        leaf_star.clone(),
        leaf_local.clone(),
    ]));
    let empty_facts = QueryAnnotationFileFacts::default();
    let root_facts = QueryAnnotationFileFacts {
        roots: vec![
            Step::Effect(Expr::Name("a".into())),
            Step::Effect(Expr::Name("b".into())),
            Step::Effect(Expr::Name("c".into())),
        ],
        ..Default::default()
    };
    let root_ts = || TsFileFacts {
        imported_bindings: vec![
            ImportedBinding {
                specifier: "./barrel_named.mts".into(),
                local: "a".into(),
                imported: "x".into(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            },
            ImportedBinding {
                specifier: "./barrel_star.mts".into(),
                local: "b".into(),
                imported: "y".into(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            },
            ImportedBinding {
                specifier: "./barrel_local.mts".into(),
                local: "c".into(),
                imported: "z".into(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            },
        ],
        ..Default::default()
    };
    let mut barrel_local_facts = QueryAnnotationFileFacts::default();
    barrel_local_facts
        .globals
        .insert("localOnly".into(), Expr::Text("ok".into()));

    let mut files = FxHashMap::default();
    files.insert(r1.clone(), make_file(&root_facts, root_ts(), &ids));
    files.insert(r2.clone(), make_file(&root_facts, root_ts(), &ids));
    files.insert(
        barrel_named.clone(),
        make_file(
            &empty_facts,
            TsFileFacts {
                exported_bindings: vec![ExportedBinding {
                    specifier: Some("./leaf_named.mts".into()),
                    local: "x".into(),
                    exported: "x".into(),
                }],
                ..Default::default()
            },
            &ids,
        ),
    );
    files.insert(
        barrel_star.clone(),
        make_file(
            &empty_facts,
            TsFileFacts {
                star_reexport_specifiers: vec!["./leaf_star.mts".into()],
                ..Default::default()
            },
            &ids,
        ),
    );
    files.insert(
        barrel_local.clone(),
        make_file(
            &barrel_local_facts,
            TsFileFacts {
                imported_bindings: vec![ImportedBinding {
                    specifier: "./leaf_local.mts".into(),
                    local: "z".into(),
                    imported: "z".into(),
                    kind: ImportedBindingKind::Named,
                    is_type_only: false,
                }],
                exported_bindings: vec![
                    ExportedBinding {
                        specifier: None,
                        local: "z".into(),
                        exported: "z".into(),
                    },
                    ExportedBinding {
                        specifier: None,
                        local: "localOnly".into(),
                        exported: "localOnly".into(),
                    },
                    ExportedBinding {
                        specifier: None,
                        local: "unmodeled".into(),
                        exported: "unmodeled".into(),
                    },
                ],
                ..Default::default()
            },
            &ids,
        ),
    );
    files.insert(
        leaf_named.clone(),
        make_file(&empty_facts, TsFileFacts::default(), &ids),
    );
    files.insert(
        leaf_star.clone(),
        make_file(&empty_facts, TsFileFacts::default(), &ids),
    );
    files.insert(
        leaf_local.clone(),
        make_file(&empty_facts, TsFileFacts::default(), &ids),
    );

    let targets: FxHashMap<&str, PathBuf> = FxHashMap::from_iter([
        ("./barrel_named.mts", barrel_named.clone()),
        ("./barrel_star.mts", barrel_star.clone()),
        ("./barrel_local.mts", barrel_local.clone()),
        ("./leaf_named.mts", leaf_named.clone()),
        ("./leaf_star.mts", leaf_star.clone()),
        ("./leaf_local.mts", leaf_local.clone()),
    ]);
    let resolve = |specifier: &str, _: &std::path::Path| targets.get(specifier).cloned();
    let graph = discover(&files, &[r1, r2, missing.clone()], &ids, &resolve);
    assert!(graph.direct_deps[&ids.id(&missing)].is_empty());
    assert_eq!(
        graph.direct_deps[&ids.id(&barrel_named)],
        vec![ids.id(&leaf_named)]
    );
    assert_eq!(
        graph.direct_deps[&ids.id(&barrel_star)],
        vec![ids.id(&leaf_star)]
    );
    assert_eq!(
        graph.direct_deps[&ids.id(&barrel_local)],
        vec![ids.id(&leaf_local)]
    );
}
