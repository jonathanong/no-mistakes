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
                star_reexport_specifiers: vec![
                    "./leaf_star.mts".into(),
                    "./external.mts".into(),
                    "./unresolved.mts".into(),
                ],
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
        ("./external.mts", missing.clone()),
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

#[test]
fn eager_deps_excludes_lazy_imports_and_follows_reexports() {
    use super::super::eager_deps::for_module;
    let [m_precise, m_empty, m_mixed, barrel_named, barrel_star, barrel_local, cycle_a, cycle_b, leaf_named, leaf_star, leaf_local, leaf_lazy, leaf_unknown, missing] =
        [
            "m_precise.mts",
            "m_empty.mts",
            "m_mixed.mts",
            "barrel_named.mts",
            "barrel_star.mts",
            "barrel_local.mts",
            "cycle_a.mts",
            "cycle_b.mts",
            "leaf_named.mts",
            "leaf_star.mts",
            "leaf_local.mts",
            "leaf_lazy.mts",
            "leaf_unknown.mts",
            "missing.mts",
        ]
        .map(PathBuf::from);
    let ids = Arc::new(ModuleIds::new([
        m_precise.clone(),
        m_empty.clone(),
        m_mixed.clone(),
        barrel_named.clone(),
        barrel_star.clone(),
        barrel_local.clone(),
        cycle_a.clone(),
        cycle_b.clone(),
        leaf_named.clone(),
        leaf_star.clone(),
        leaf_local.clone(),
        leaf_lazy.clone(),
        leaf_unknown.clone(),
        missing.clone(),
    ]));
    let precise_facts = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let empty_facts = QueryAnnotationFileFacts::default();
    let mut mixed_facts = QueryAnnotationFileFacts {
        roots: vec![
            Step::Effect(Expr::Name("a".into())),
            Step::Effect(Expr::Name("b".into())),
            Step::Effect(Expr::Name("c".into())),
            Step::Effect(Expr::Name("cyc".into())),
            Step::Effect(Expr::Name("unk".into())),
            Step::Effect(Expr::Name("def".into())),
            Step::Effect(Expr::Name("ext".into())),
        ],
        ..Default::default()
    };
    mixed_facts
        .globals
        .insert("lazyFn".into(), Expr::Name("lazyOnly".into()));
    let mut leaf_facts = QueryAnnotationFileFacts::default();
    leaf_facts
        .globals
        .insert("x".into(), Expr::Text("1".into()));
    leaf_facts
        .globals
        .insert("y".into(), Expr::Text("2".into()));
    leaf_facts
        .globals
        .insert("z".into(), Expr::Text("3".into()));
    let mut unknown_facts = QueryAnnotationFileFacts::default();
    unknown_facts.globals.insert("u".into(), Expr::Unknown);

    let imp = |specifier: &str, local: &str, imported: &str| ImportedBinding {
        specifier: specifier.into(),
        local: local.into(),
        imported: imported.into(),
        kind: ImportedBindingKind::Named,
        is_type_only: false,
    };
    let exp = |specifier: Option<&str>, name: &str| ExportedBinding {
        specifier: specifier.map(Into::into),
        local: name.into(),
        exported: name.into(),
    };

    let mut files = FxHashMap::default();
    files.insert(
        m_precise.clone(),
        make_file(&precise_facts, TsFileFacts::default(), &ids),
    );
    files.insert(
        m_empty.clone(),
        make_file(&empty_facts, TsFileFacts::default(), &ids),
    );
    files.insert(
        m_mixed.clone(),
        make_file(
            &mixed_facts,
            TsFileFacts {
                imported_bindings: vec![
                    imp("./barrel_named.mts", "a", "x"),
                    imp("./barrel_star.mts", "b", "y"),
                    imp("./barrel_local.mts", "c", "z"),
                    imp("./cycle_a.mts", "cyc", "x"),
                    imp("./leaf_unknown.mts", "unk", "u"),
                    imp("./barrel_star.mts", "def", "default"),
                    imp("./missing.mts", "ext", "x"),
                    imp("./leaf_lazy.mts", "lazyOnly", "x"),
                ],
                ..Default::default()
            },
            &ids,
        ),
    );
    files.insert(
        barrel_named.clone(),
        make_file(
            &empty_facts,
            TsFileFacts {
                exported_bindings: vec![exp(Some("./leaf_named.mts"), "x")],
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
            &empty_facts,
            TsFileFacts {
                imported_bindings: vec![imp("./leaf_local.mts", "z", "z")],
                exported_bindings: vec![exp(None, "z")],
                ..Default::default()
            },
            &ids,
        ),
    );
    files.insert(
        cycle_a.clone(),
        make_file(
            &empty_facts,
            TsFileFacts {
                exported_bindings: vec![exp(Some("./cycle_b.mts"), "x")],
                ..Default::default()
            },
            &ids,
        ),
    );
    files.insert(
        cycle_b.clone(),
        make_file(
            &empty_facts,
            TsFileFacts {
                exported_bindings: vec![exp(Some("./cycle_a.mts"), "x")],
                ..Default::default()
            },
            &ids,
        ),
    );
    for (p, name) in [
        (&leaf_named, "x"),
        (&leaf_star, "y"),
        (&leaf_local, "z"),
        (&leaf_lazy, "x"),
    ] {
        files.insert(
            p.clone(),
            make_file(
                &leaf_facts,
                TsFileFacts {
                    exported_bindings: vec![exp(None, name)],
                    ..Default::default()
                },
                &ids,
            ),
        );
    }
    files.insert(
        leaf_unknown.clone(),
        make_file(
            &unknown_facts,
            TsFileFacts {
                exported_bindings: vec![exp(None, "u")],
                ..Default::default()
            },
            &ids,
        ),
    );
    let targets: FxHashMap<&str, PathBuf> = FxHashMap::from_iter([
        ("./barrel_named.mts", barrel_named),
        ("./barrel_star.mts", barrel_star),
        ("./barrel_local.mts", barrel_local),
        ("./cycle_a.mts", cycle_a),
        ("./cycle_b.mts", cycle_b),
        ("./leaf_named.mts", leaf_named.clone()),
        ("./leaf_star.mts", leaf_star.clone()),
        ("./leaf_local.mts", leaf_local.clone()),
        ("./leaf_lazy.mts", leaf_lazy),
        ("./leaf_unknown.mts", leaf_unknown),
        ("./missing.mts", missing),
    ]);
    let resolve = |specifier: &str, _: &std::path::Path| targets.get(specifier).cloned();
    assert_eq!(
        for_module(
            &files,
            &ids,
            ids.id(&m_precise),
            &[ids.id(&leaf_named)],
            &resolve
        ),
        vec![ids.id(&leaf_named)]
    );
    assert!(for_module(&files, &ids, ids.id(&m_empty), &[], &resolve).is_empty());
    assert_eq!(
        for_module(&files, &ids, ids.id(&m_mixed), &[], &resolve),
        vec![ids.id(&leaf_local), ids.id(&leaf_named), ids.id(&leaf_star)]
    );
}
