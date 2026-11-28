use super::for_module;
use super::tests::{exp, make_file};
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::postgres::query_annotation::module_ids::ModuleIds;
use crate::codebase::postgres::query_annotation::{Expr, QueryAnnotationFileFacts, Step};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn eager_deps_tracks_namespace_imports_and_star_reexports() {
    let names = [
        "consumer.mts",
        "registry.mts",
        "reexport_leaf.mts",
        "star_leaf.mts",
        "star_cyc_a.mts",
        "star_cyc_b.mts",
        "missing.mts",
    ];
    let paths = names.map(PathBuf::from);
    let ids = Arc::new(ModuleIds::new(paths.iter().cloned()));
    let [consumer, registry, reexport_leaf, star_leaf, star_cyc_a, star_cyc_b, missing] = paths;
    let empty_facts = QueryAnnotationFileFacts::default();
    let consumer_facts = QueryAnnotationFileFacts {
        roots: ["reg", "cyc", "miss"]
            .into_iter()
            .map(|n| Step::Effect(Expr::Name(n.into())))
            .collect(),
        ..Default::default()
    };
    let mut leaf_facts = QueryAnnotationFileFacts::default();
    leaf_facts
        .globals
        .insert("prefix".into(), Expr::Text("/* ok */".into()));
    leaf_facts
        .globals
        .insert("reexported".into(), Expr::Text("/* re */".into()));
    let ns_imp = |specifier: &str, local: &str| ImportedBinding {
        specifier: specifier.into(),
        local: local.into(),
        imported: "*".into(),
        kind: ImportedBindingKind::Namespace,
        is_type_only: false,
    };
    let files = FxHashMap::from_iter([
        (
            consumer.clone(),
            make_file(
                &consumer_facts,
                TsFileFacts {
                    imported_bindings: vec![
                        ns_imp("./registry.mts", "reg"),
                        ns_imp("./star_cyc_a.mts", "cyc"),
                        ns_imp("./missing.mts", "miss"),
                    ],
                    ..Default::default()
                },
                &ids,
            ),
        ),
        (
            registry.clone(),
            make_file(
                &leaf_facts,
                TsFileFacts {
                    exported_bindings: vec![
                        exp(None, "prefix"),
                        exp(Some("./reexport_leaf.mts"), "reexported"),
                    ],
                    star_reexport_specifiers: vec!["./star_leaf.mts".into()],
                    ..Default::default()
                },
                &ids,
            ),
        ),
        (
            reexport_leaf.clone(),
            make_file(
                &leaf_facts,
                TsFileFacts {
                    exported_bindings: vec![exp(None, "reexported")],
                    ..Default::default()
                },
                &ids,
            ),
        ),
        (
            star_leaf.clone(),
            make_file(
                &leaf_facts,
                TsFileFacts {
                    exported_bindings: vec![exp(None, "prefix")],
                    ..Default::default()
                },
                &ids,
            ),
        ),
        (
            star_cyc_a.clone(),
            make_file(
                &empty_facts,
                TsFileFacts {
                    star_reexport_specifiers: vec!["./star_cyc_b.mts".into()],
                    ..Default::default()
                },
                &ids,
            ),
        ),
        (
            star_cyc_b.clone(),
            make_file(
                &empty_facts,
                TsFileFacts {
                    star_reexport_specifiers: vec!["./star_cyc_a.mts".into()],
                    ..Default::default()
                },
                &ids,
            ),
        ),
    ]);
    let targets: FxHashMap<&str, PathBuf> = FxHashMap::from_iter([
        ("./registry.mts", registry.clone()),
        ("./reexport_leaf.mts", reexport_leaf.clone()),
        ("./star_leaf.mts", star_leaf.clone()),
        ("./star_cyc_a.mts", star_cyc_a),
        ("./star_cyc_b.mts", star_cyc_b),
        ("./missing.mts", missing),
    ]);
    let resolve = |specifier: &str, _: &std::path::Path| targets.get(specifier).cloned();
    assert_eq!(
        for_module(&files, &ids, ids.id(&consumer), &[], &resolve),
        vec![
            ids.id(&reexport_leaf),
            ids.id(&registry),
            ids.id(&star_leaf)
        ]
    );
}
