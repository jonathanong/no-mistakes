use super::*;
use crate::codebase::{
    check_facts::CheckFileFacts,
    postgres::sql_identifier_syntax::{collect, helper_recovery::tests::profile},
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::sync::Arc;

#[test]
fn abrupt_proofs_require_complete_acyclic_models_within_the_work_budget() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("variants/coverage-catalog-edges/abrupt.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let profile = profile();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        CheckFileFacts {
            sql_identifier_syntax: collect(program, &ts, &source, std::slice::from_ref(&profile)),
            ts: Arc::new(ts),
            ..Default::default()
        }
    })
    .unwrap();
    let at = |text: &str| {
        let start = source.rfind(text).unwrap() as u32;
        (start, start + text.len() as u32)
    };
    let pure = at("pure(value)");
    let recursive = at("recursive(value)");
    let effectful = at("effectful(value)");
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    {
        let catalog = PreparedCatalog {
            files: &files,
            application: None,
            profile: Some(&profile),
            resolve: &|_, _| None,
        };
        assert!(catalog.throw_has_no_selected_sql(&path, pure, &mut Vec::new(), &mut 0));
        assert!(!catalog.throw_has_no_selected_sql(&path, pure, &mut Vec::new(), &mut 10_000));
        assert!(!catalog.throw_has_no_selected_sql(&path, recursive, &mut Vec::new(), &mut 0));
        assert!(!catalog.throw_has_no_selected_sql(&path, effectful, &mut Vec::new(), &mut 0));
        let missing = path.with_file_name("missing.mts");
        assert!(!catalog.selected_sql(&missing, None));
        assert!(!catalog.throw_has_no_selected_sql(&missing, pure, &mut Vec::new(), &mut 0));
    }
    let models = &mut files.get_mut(&path).unwrap().sql_identifier_syntax.recovery;
    let callable = models
        .calls
        .iter()
        .find(|call| call.span == pure)
        .unwrap()
        .callee
        .unwrap();
    let key = models.binding_functions[&callable].unwrap();
    models.functions.remove(&key);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    assert!(!catalog.throw_has_no_selected_sql(&path, pure, &mut Vec::new(), &mut 0));
}
