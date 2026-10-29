use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
};
use crate::codebase::{
    check_facts::CheckFileFacts,
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

fn root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-no-offset").join("low-module-demand/src")
}
fn prepared(names: &[&str]) -> FileIdMap<CheckFileFacts> {
    names
        .iter()
        .map(|name| {
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
                let syntax =
                    super::super::collect(program, &ts, &source, &[super::tests::profile()]);
                CheckFileFacts {
                    ts: Arc::new(ts),
                    sql_identifier_syntax: syntax,
                    ..Default::default()
                }
            })
            .unwrap();
            (path, facts)
        })
        .collect()
}

#[test]
fn target_free_and_type_only_modules_discard_unreachable_models() {
    let files = prepared(&["target-free.mts", "type-only.mts"]);
    for name in ["target-free.mts", "type-only.mts"] {
        let facts = &files[&root().join(name)];
        assert!(facts
            .sql_identifier_syntax
            .profiles
            .iter()
            .all(|(_, calls)| calls.is_empty()));
        let models = &facts.sql_identifier_syntax.recovery;
        assert!(models.functions.is_empty() && models.locals.is_empty() && models.calls.is_empty());
        assert!(models.executors.is_empty() && models.sinks.is_empty());
    }
    let exports = &files[&root().join("type-only.mts")]
        .ts
        .module_bindings
        .as_ref()
        .unwrap()
        .exports;
    assert!(exports.iter().all(|export| export.type_only));
}

#[test]
fn executor_escape_and_unproved_member_keep_unknown_obligations() {
    let files = prepared(&["executor-escape.mts", "unproved-query.mts"]);
    for (name, marker) in [
        ("executor-escape.mts", "read"),
        ("unproved-query.mts", "helper.query("),
    ] {
        let path = root().join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = &files[&path].sql_identifier_syntax;
        let selected = facts
            .profiles
            .iter()
            .flat_map(|(_, calls)| calls)
            .find(|call| source[call.span.0 as usize..call.span.1 as usize].contains(marker))
            .unwrap();
        assert!(matches!(selected.input, super::super::Input::Unknown));
        assert_eq!(
            selected.line,
            crate::codebase::ts_source::byte_offset_to_line(&source, selected.span.0 as usize)
        );
        assert!(!facts.recovery.calls.is_empty());
    }
}

#[test]
fn exported_helper_without_local_executor_remains_resolvable() {
    let files = prepared(&["exported.mts", "caller.mts"]);
    let helper = root().join("exported.mts");
    let caller = root().join("caller.mts");
    let helper_facts = &files[&helper].sql_identifier_syntax;
    assert!(helper_facts
        .profiles
        .iter()
        .all(|(_, calls)| calls.is_empty()));
    assert!(!helper_facts.recovery.functions.is_empty());
    let resolve =
        |specifier: &str, _: &Path| (specifier == "./exported.mjs").then(|| helper.clone());
    let catalog = PreparedCatalog {
        application: None,
        profile: None,
        files: &files,
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&caller).unwrap();
    let call = files[&caller]
        .sql_identifier_syntax
        .recovery
        .calls
        .iter()
        .find(|call| source[call.span.0 as usize..call.span.1 as usize].starts_with("read("))
        .unwrap();
    let mut cache = Cache::default();
    let values = Engine::new(&catalog, &mut cache, Limits::default())
        .evaluate_at(&caller, &call.arguments[0], None)
        .unwrap();
    let [Value::Text(text)] = values.as_slice() else {
        panic!("exported SQL helper was not recovered")
    };
    assert_eq!(text.raw.sql, "SELECT 1 OFFSET 7");
    assert_eq!(
        text.raw.positions[0].source_path.as_deref(),
        Some(helper.as_path())
    );
}
