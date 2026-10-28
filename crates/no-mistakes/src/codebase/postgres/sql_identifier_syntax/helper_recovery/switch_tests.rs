use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    tests::{fixture, profile},
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
use std::{path::PathBuf, sync::Arc};

fn prepared() -> FileIdMap<CheckFileFacts> {
    let mut files = FileIdMap::default();
    for name in ["switch-abrupt.mts", "switch-abrupt-helper.mts"] {
        let path = fixture().join("src").join(name);
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
            CheckFileFacts {
                sql_identifier_syntax: super::super::collect(program, &ts, &source, &[profile()]),
                ts: Arc::new(ts),
                ..Default::default()
            }
        })
        .unwrap();
        files.insert(path, facts);
    }
    files
}

#[test]
fn finite_switch_requires_source_proven_abrupt_default_and_literal_arms() {
    let files = prepared();
    let path = fixture().join("src/switch-abrupt.mts");
    let helper = fixture().join("src/switch-abrupt-helper.mts");
    let resolve = |specifier: &str, _: &std::path::Path| -> Option<PathBuf> {
        (specifier == "./switch-abrupt-helper.mjs").then(|| helper.clone())
    };
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let mut checked = 0;
    for call in &files[&path].sql_identifier_syntax.recovery.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(") {
            continue;
        }
        let label = source[call.span.1 as usize..].lines().next().unwrap();
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate(&path, &call.arguments[0]);
        if label.contains("COMPLETE") {
            assert!(!engine.residual(), "{raw}");
            let values = values.expect(raw);
            assert_eq!(
                values.len(),
                if raw.contains("selected(mode)") || raw.contains("correlated(mode)") {
                    2
                } else {
                    1
                },
                "{raw}"
            );
            assert!(
                values.iter().all(|value| matches!(value, Value::Sql(_))),
                "{raw}"
            );
        } else {
            assert!(engine.residual(), "{raw}");
        }
        checked += 1;
    }
    assert_eq!(checked, 7);
}
