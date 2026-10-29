use super::{collect, Pending, Summary};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan, TsFileFacts,
};
use std::{collections::BTreeMap, sync::Arc};

type Projection = (BTreeMap<String, Summary>, Vec<Pending>);
fn prepared(name: &str, change: impl FnOnce(&mut Syntax, &mut TsFileFacts)) -> Projection {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("async-return-metadata/src")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut facts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                imports: true,
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let mut syntax =
            super::super::super::syntax::collect_with_helpers(program, true, true, true);
        let executors = syntax
            .calls
            .iter()
            .filter(|call| source[call.span.0 as usize..].starts_with("read("))
            .map(|call| call.span)
            .collect::<crate::fx::FxHashSet<_>>();
        assert_eq!(executors.len(), 1);
        change(&mut syntax, &mut facts);
        let bindings = Bindings::new(&syntax, &facts, executors.iter().copied(), true);
        collect(
            &syntax,
            &bindings,
            facts.module_bindings.as_deref(),
            &executors,
        )
    })
    .unwrap()
}

#[test]
fn async_metadata_exports_require_unique_local_identity_and_complete_module() {
    let (exports, pending) = prepared("query.mts", |_, _| {});
    assert_eq!(exports.len(), 2);
    assert_eq!(pending.len(), 1);
    for kind in ["missing", "duplicate", "unresolved", "reexport"] {
        let (exports, _) = prepared("query.mts", |_, facts| {
            if kind == "missing" {
                facts.module_bindings = None;
                return;
            }
            let module = Arc::make_mut(facts.module_bindings.as_mut().unwrap());
            let i = module
                .exports
                .iter()
                .position(|e| e.exported == "factory")
                .unwrap();
            match kind {
                "duplicate" => {
                    let export = module.exports[i].clone();
                    module.exports.extend([export.clone(), export]);
                }
                "unresolved" => module.exports[i].local = "absent".into(),
                "reexport" => module.exports[i].specifier = Some("./remote.mjs".into()),
                _ => unreachable!(),
            }
        });
        assert!(!exports.contains_key("factory"), "{kind}");
    }
}

#[test]
fn async_metadata_callers_require_result_owner_and_unique_stable_import() {
    for kind in ["missing-result", "duplicate-import", "missing-import"] {
        let (_, pending) = prepared("query.mts", |_, facts| {
            let module = Arc::make_mut(facts.module_bindings.as_mut().unwrap());
            match kind {
                "missing-result" => module.bindings.retain(|binding| binding.name != "result"),
                "duplicate-import" => {
                    let import = module
                        .imports
                        .iter_mut()
                        .find(|i| i.specifier == "./remote.mjs")
                        .unwrap();
                    import.bindings.push(import.bindings[0].clone());
                }
                "missing-import" => module.imports.retain(|i| i.specifier != "./remote.mjs"),
                _ => unreachable!(),
            }
        });
        assert!(pending.is_empty(), "{kind}");
    }
    for name in ["mutable-import.mts", "dynamic-caller.mts"] {
        assert!(prepared(name, |_, _| {}).1.is_empty(), "{name}");
    }
}

#[test]
fn async_metadata_source_rejects_mismatched_or_incomplete_builder_identity() {
    for kind in ["wrong-declaration", "missing-use"] {
        let (exports, _) = prepared("query.mts", |syntax, facts| {
            let module = Arc::make_mut(facts.module_bindings.as_mut().unwrap());
            if kind == "wrong-declaration" {
                let owner = module
                    .bindings
                    .iter()
                    .find(|b| b.name == "factory")
                    .unwrap();
                syntax
                    .async_returns
                    .get_mut(&(owner.span.start, owner.span.end))
                    .unwrap()
                    .declaration = (0, 1);
            } else {
                let owner = module
                    .bindings
                    .iter_mut()
                    .find(|b| b.name == "statement")
                    .unwrap();
                // A partial reference table must not silently satisfy a full native lifetime census.
                assert_eq!(owner.references.len(), 2);
                owner.references.remove(0);
            }
        });
        assert!(!exports.contains_key("factory"), "{kind}");
        assert!(exports.contains_key("withLocalEffect"));
    }
}
