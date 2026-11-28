use super::for_module;
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

fn imp(specifier: &str, local: &str, imported: &str) -> ImportedBinding {
    ImportedBinding {
        specifier: specifier.into(),
        local: local.into(),
        imported: imported.into(),
        kind: ImportedBindingKind::Named,
        is_type_only: false,
    }
}

fn exp(specifier: Option<&str>, name: &str) -> ExportedBinding {
    ExportedBinding {
        specifier: specifier.map(Into::into),
        local: name.into(),
        exported: name.into(),
    }
}

#[test]
fn eager_deps_excludes_lazy_imports_and_follows_reexports() {
    let names = [
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
    ];
    let paths = names.map(PathBuf::from);
    let ids = Arc::new(ModuleIds::new(paths.iter().cloned()));
    let [m_precise, m_empty, m_mixed, b_named, b_star, b_local, cyc_a, cyc_b, l_named, l_star, l_local, l_lazy, l_unk, missing] =
        paths;
    let precise_facts = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let empty_facts = QueryAnnotationFileFacts::default();
    let mut mixed_facts = QueryAnnotationFileFacts {
        roots: ["a", "b", "c", "cyc", "unk", "def", "ext"]
            .into_iter()
            .map(|n| Step::Effect(Expr::Name(n.into())))
            .collect(),
        ..Default::default()
    };
    mixed_facts
        .globals
        .insert("lazyFn".into(), Expr::Name("lazyOnly".into()));
    let mut leaf_facts = QueryAnnotationFileFacts::default();
    for (k, v) in [("x", "1"), ("y", "2"), ("z", "3")] {
        leaf_facts.globals.insert(k.into(), Expr::Text(v.into()));
    }
    let mut unknown_facts = QueryAnnotationFileFacts::default();
    unknown_facts.globals.insert("u".into(), Expr::Unknown);

    let ts_exp = |spec: Option<&str>, name: &str| TsFileFacts {
        exported_bindings: vec![exp(spec, name)],
        ..Default::default()
    };
    let mut files = FxHashMap::from_iter([
        (
            m_precise.clone(),
            make_file(&precise_facts, TsFileFacts::default(), &ids),
        ),
        (
            m_empty.clone(),
            make_file(&empty_facts, TsFileFacts::default(), &ids),
        ),
        (
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
        ),
        (
            b_named.clone(),
            make_file(&empty_facts, ts_exp(Some("./leaf_named.mts"), "x"), &ids),
        ),
        (
            b_star.clone(),
            make_file(
                &empty_facts,
                TsFileFacts {
                    star_reexport_specifiers: vec!["./leaf_star.mts".into()],
                    ..Default::default()
                },
                &ids,
            ),
        ),
        (
            b_local.clone(),
            make_file(
                &empty_facts,
                TsFileFacts {
                    imported_bindings: vec![imp("./leaf_local.mts", "z", "z")],
                    exported_bindings: vec![exp(None, "z")],
                    ..Default::default()
                },
                &ids,
            ),
        ),
        (
            cyc_a.clone(),
            make_file(&empty_facts, ts_exp(Some("./cycle_b.mts"), "x"), &ids),
        ),
        (
            cyc_b.clone(),
            make_file(&empty_facts, ts_exp(Some("./cycle_a.mts"), "x"), &ids),
        ),
        (
            l_unk.clone(),
            make_file(&unknown_facts, ts_exp(None, "u"), &ids),
        ),
    ]);
    for (p, name) in [
        (&l_named, "x"),
        (&l_star, "y"),
        (&l_local, "z"),
        (&l_lazy, "x"),
    ] {
        files.insert(p.clone(), make_file(&leaf_facts, ts_exp(None, name), &ids));
    }
    let targets: FxHashMap<&str, PathBuf> = FxHashMap::from_iter([
        ("./barrel_named.mts", b_named),
        ("./barrel_star.mts", b_star),
        ("./barrel_local.mts", b_local),
        ("./cycle_a.mts", cyc_a),
        ("./cycle_b.mts", cyc_b),
        ("./leaf_named.mts", l_named.clone()),
        ("./leaf_star.mts", l_star.clone()),
        ("./leaf_local.mts", l_local.clone()),
        ("./leaf_lazy.mts", l_lazy),
        ("./leaf_unknown.mts", l_unk),
        ("./missing.mts", missing),
    ]);
    let resolve = |specifier: &str, _: &std::path::Path| targets.get(specifier).cloned();
    assert_eq!(
        for_module(
            &files,
            &ids,
            ids.id(&m_precise),
            &[ids.id(&l_named)],
            &resolve
        ),
        vec![ids.id(&l_named)]
    );
    assert!(for_module(&files, &ids, ids.id(&m_empty), &[], &resolve).is_empty());
    assert_eq!(
        for_module(&files, &ids, ids.id(&m_mixed), &[], &resolve),
        vec![ids.id(&l_local), ids.id(&l_named), ids.id(&l_star)]
    );
}
