use super::is_statically_pure_top_level;
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::postgres::query_annotation::evaluate::File;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::{Expr, Function, QueryAnnotationFileFacts, Step};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashSet;
use std::path::PathBuf;
use std::sync::Arc;

fn make_file<'a>(facts: &'a QueryAnnotationFileFacts, imported: &[&str]) -> File<'a> {
    let ts = TsFileFacts {
        imported_bindings: imported
            .iter()
            .map(|name| ImportedBinding {
                specifier: "./dep.mts".into(),
                local: (*name).into(),
                imported: (*name).into(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            })
            .collect(),
        ..Default::default()
    };
    File {
        facts,
        module: Arc::new(Module::new(Arc::new(ts))),
        executors: FxHashSet::default(),
    }
}

fn scalar_fn(asynchronous: bool, body: Vec<Step>) -> Expr {
    Expr::Function(Arc::new(Function {
        start: 0,
        params: vec!["p".into()],
        self_name: Some("selfFn".into()),
        body,
        supported: true,
        asynchronous,
        arrow: false,
        append_callback: None,
    }))
}

#[test]
fn is_statically_pure_top_level_covers_steps_exprs_and_local_scalar_callees() {
    let path = PathBuf::from("mod.mts");
    let resolve = |_: &str, _: &std::path::Path| None;

    // Pure top-level with Reserve, Var, ComputedMember, Template, Binary, and pure local scalar call.
    let mut facts = QueryAnnotationFileFacts::default();
    facts.globals.insert(
        "calc".into(),
        scalar_fn(
            false,
            vec![
                Step::Reserve(vec!["r".into()]),
                Step::Var("v".into()),
                Step::Hoisted("h".into(), Expr::Number(1)),
                Step::Bind(
                    "r".into(),
                    Expr::Binary(
                        "+".into(),
                        Box::new(Expr::Name("p".into())),
                        Box::new(Expr::Template(vec![Expr::Text("x".into())])),
                    ),
                ),
                Step::Return(Expr::Name("r".into())),
            ],
        ),
    );
    facts.roots = vec![
        Step::Reserve(vec!["a".into()]),
        Step::Var("b".into()),
        Step::Bind(
            "a".into(),
            Expr::ComputedMember(
                Box::new(Expr::Name("b".into())),
                Box::new(Expr::Text("k".into())),
            ),
        ),
        Step::Effect(Expr::Call {
            callee: Box::new(Expr::Name("calc".into())),
            args: vec![Expr::Boolean(true), Expr::Primitive],
            start: 0,
            end: 5,
        }),
    ];
    assert!(is_statically_pure_top_level(
        &make_file(&facts, &["imp"]),
        &path,
        &resolve
    ));

    // Impure cases: bind imported name, non-Name callee, imported callee, async callee,
    // non-local bind in callee, effect step in callee, non-scalar arg, arguments name.
    let bad_roots: Vec<(QueryAnnotationFileFacts, &[&str])> = vec![
        (
            QueryAnnotationFileFacts {
                roots: vec![Step::Bind("imp".into(), Expr::Number(1))],
                ..Default::default()
            },
            &["imp"],
        ),
        (
            QueryAnnotationFileFacts {
                roots: vec![Step::Effect(Expr::Call {
                    callee: Box::new(Expr::Member(Box::new(Expr::Name("o".into())), "m".into())),
                    args: vec![],
                    start: 0,
                    end: 1,
                })],
                ..Default::default()
            },
            &[],
        ),
        (
            QueryAnnotationFileFacts {
                roots: vec![Step::Effect(Expr::Call {
                    callee: Box::new(Expr::Name("imp".into())),
                    args: vec![],
                    start: 0,
                    end: 1,
                })],
                ..Default::default()
            },
            &["imp"],
        ),
        (
            QueryAnnotationFileFacts {
                roots: vec![Step::Effect(Expr::Call {
                    callee: Box::new(Expr::Name("calc".into())),
                    args: vec![Expr::Array(vec![])],
                    start: 0,
                    end: 1,
                })],
                globals: facts.globals.clone(),
                ..Default::default()
            },
            &[],
        ),
        (
            QueryAnnotationFileFacts {
                roots: vec![Step::Effect(Expr::Name("arguments".into()))],
                ..Default::default()
            },
            &[],
        ),
    ];
    for (bad, imported) in &bad_roots {
        assert!(!is_statically_pure_top_level(
            &make_file(bad, imported),
            &path,
            &resolve
        ));
    }

    for bad_fn in [
        scalar_fn(true, vec![Step::Return(Expr::Number(1))]),
        scalar_fn(false, vec![Step::Bind("outer".into(), Expr::Number(1))]),
        scalar_fn(false, vec![Step::Effect(Expr::Number(1))]),
    ] {
        let mut f = QueryAnnotationFileFacts::default();
        f.globals.insert("fn_bad".into(), bad_fn);
        f.roots.push(Step::Effect(Expr::Call {
            callee: Box::new(Expr::Name("fn_bad".into())),
            args: vec![],
            start: 0,
            end: 1,
        }));
        assert!(!is_statically_pure_top_level(
            &make_file(&f, &[]),
            &path,
            &resolve
        ));
    }
}
