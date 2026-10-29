use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    values::{Cache, Engine},
};
use crate::codebase::{
    check_facts::CheckFileFacts,
    postgres::embedded::helpers::program::Stmt,
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::{path::PathBuf, sync::Arc};

fn root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-no-offset").join("low-body-demand/src")
}
fn prepared(name: &str) -> (PathBuf, String, CheckFileFacts) {
    let path = root().join(name);
    let source = std::fs::read_to_string(&path).unwrap();
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
        let syntax = super::super::collect(program, &ts, &source, &[super::tests::profile()]);
        CheckFileFacts {
            ts: Arc::new(ts),
            sql_identifier_syntax: syntax,
            ..Default::default()
        }
    })
    .unwrap();
    (path, source, facts)
}
fn function<'a>(
    models: &'a super::FileModels,
    source: &str,
    marker: &str,
) -> &'a super::FunctionModel {
    models
        .functions
        .values()
        .find(|function| {
            source[function.span.0 as usize..function.span.1 as usize].contains(marker)
        })
        .unwrap()
}

#[test]
fn known_anonymous_body_is_trimmed_but_unknown_caller_replays() {
    let (path, source, facts) = prepared("callbacks.mts");
    let models = &facts.sql_identifier_syntax.recovery;
    let known = function(models, &source, "SELECT 1 OFFSET 3");
    assert!(matches!(
        known.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let unknown = function(models, &source, "SELECT 1 OFFSET 4");
    assert!(!matches!(
        unknown.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let bound = function(models, &source, "SELECT 1 OFFSET 5");
    assert!(!matches!(
        bound.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let wrapped = function(models, &source, "SELECT 1 OFFSET 11");
    assert!(!matches!(
        wrapped.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let partial = function(models, &source, "OFFSET 9");
    assert!(!matches!(
        partial.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let partial_call = facts
        .sql_identifier_syntax
        .profiles
        .iter()
        .flat_map(|(_, calls)| calls)
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("read(`SELECT 1")
        })
        .unwrap();
    assert!(matches!(&partial_call.input, super::super::Input::Sql {
        holes, parameterized: None, ..
    } if !holes.is_empty()));
    let named = function(models, &source, "SELECT 1 OFFSET 10");
    assert!(!matches!(
        named.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let sink = models
        .calls
        .iter()
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("read(statement)")
        })
        .unwrap();
    let sink_span = sink.span;
    let profile = super::tests::profile();
    assert!(models.sinks.get(&profile).unwrap().contains(&sink.span));
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    let models = &files[&path].sql_identifier_syntax.recovery;
    let sink = models
        .calls
        .iter()
        .find(|call| call.span == sink_span)
        .unwrap();
    let catalog = PreparedCatalog {
        application: None,
        profile: Some(&profile),
        files: &files,
        resolve: &|_, _| None,
    };
    let mut cache = Cache::default();
    let proof = Engine::new(&catalog, &mut cache, Limits::default())
        .site(&path, sink, models.executors.get(&profile).unwrap())
        .unwrap();
    assert!(proof.complete());
    assert!(proof
        .alternatives
        .iter()
        .any(|sql| sql.sql == "SELECT 1 OFFSET 4"));
    assert!(
        !crate::codebase::postgres::sql_offset_facts(&proof.alternatives[0].sql)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn iterator_params_and_conservative_module_roots_survive() {
    let (_, source, facts) = prepared("callbacks.mts");
    let models = &facts.sql_identifier_syntax.recovery;
    let iterator = function(models, &source, "SELECT 1 OFFSET 8");
    assert!(matches!(
        iterator.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    assert_eq!(iterator.lowered.program.params.len(), 1);
    assert!(models.iterators.contains_key(&iterator.id));
    let (_, source, exported) = prepared("exported.mts");
    let model = function(
        &exported.sql_identifier_syntax.recovery,
        &source,
        "SELECT 1 OFFSET 6",
    );
    assert!(!matches!(
        model.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let (_, source, diagnostic) = prepared("diagnostic.mts");
    assert!(!diagnostic
        .ts
        .module_bindings
        .as_ref()
        .unwrap()
        .diagnostics
        .is_empty());
    let model = function(
        &diagnostic.sql_identifier_syntax.recovery,
        &source,
        "SELECT 1 OFFSET 7",
    );
    assert!(!matches!(
        model.lowered.program.body.as_slice(),
        [Stmt::Opaque]
    ));
}
