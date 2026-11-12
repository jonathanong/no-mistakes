use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
};
use crate::codebase::{
    check_facts::CheckFileFacts,
    postgres::EmbeddedSqlOptions,
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::{path::PathBuf, sync::Arc};

fn assert_recovery(name: &str, expected: &[(&str, Option<&str>)]) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
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
        let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
        let syntax = super::super::collect(program, &ts, &source, &[profile]);
        CheckFileFacts {
            ts: Arc::new(ts),
            sql_identifier_syntax: syntax,
            ..Default::default()
        }
    })
    .unwrap();
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &|_, _| None,
    };
    let calls = &files[&path].sql_identifier_syntax.recovery.calls;
    assert_eq!(
        calls
            .iter()
            .filter(|call| source[call.span.0 as usize..call.span.1 as usize].starts_with("read("))
            .count(),
        expected.len()
    );
    for (function, expected) in expected {
        let text = format!("read({function}())");
        let call = calls
            .iter()
            .find(|call| source[call.span.0 as usize..call.span.1 as usize] == text)
            .unwrap_or_else(|| panic!("missing {text}"));
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate_at(&path, &call.arguments[0], None);
        match expected {
            Some(expected) => {
                let values = values.unwrap_or_else(|| panic!("{text} did not recover SQL"));
                let sql = values
                    .iter()
                    .map(|value| {
                        let Value::Sql(sql) = value else {
                            panic!("{text} did not return SQL")
                        };
                        assert!(
                            sql.valid_binds(),
                            "{text} must keep generated binds in SQL parameter positions"
                        );
                        assert_eq!(sql.raw.origins.len(), sql.raw.sql.len(), "{text}");
                        sql.raw.sql.as_str()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(sql, [*expected], "{text}");
                assert!(!engine.residual(), "{text}");
            }
            None => {
                assert!(values.is_none(), "{text} must remain unknown");
                assert!(engine.residual(), "{text} must retain uncertainty");
            }
        }
    }
}

#[test]
fn one_loop_pass_preserves_sql_effect_order_and_iteration_body_evidence() {
    assert_recovery(
        "3304-helper-loops.mts",
        &[
            (
                "cForOrder",
                Some("SELECT 1 /* init */ /* test */ /* body */ /* update */"),
            ),
            ("cForBindings", Some("SELECT 1 /* bindings */")),
            ("cForEmptyBody", Some("SELECT 1")),
            ("whileOrder", Some("SELECT 1 /* test */ /* body */")),
            ("doWhileOrder", Some("SELECT 1 /* body */ /* test */")),
            ("forInBody", Some("SELECT 1 /* in */")),
            ("forOfBody", Some("SELECT 1 /* of */")),
            ("cForDestructuring", None),
            ("forOfDestructuring", None),
            ("forInAssignment", None),
            ("unsupportedBody", None),
        ],
    );
}

#[test]
fn method_callbacks_exempt_only_inert_bodies_and_keep_sql_receivers_closed() {
    assert_recovery(
        "3304-helper-arguments.mts",
        &[
            ("arrowValue", Some("SELECT 1")),
            ("arrowBody", Some("SELECT 1")),
            ("functionBody", Some("SELECT 1")),
            ("arrowCall", None),
            ("arrowTemplate", None),
            ("arrowTaggedTemplate", None),
            ("functionSqlBody", None),
            ("functionCallBody", None),
            ("arrowAssignment", None),
            ("blockAssignment", None),
            ("builderReceiver", None),
        ],
    );
}

#[test]
fn closed_values_preserve_own_properties_bind_renumbering_and_nullish_semantics() {
    assert_recovery(
        "3304-helper-values.mts",
        &[
            ("ownComputed", Some("SELECT 1")),
            ("duplicateOwn", Some("SELECT 1")),
            ("missingOwn", None),
            ("computedSql", None),
            ("nullMember", None),
            ("dataIndex", Some("SELECT $1")),
            ("numericAddition", Some("SELECT $1")),
            ("textAddition", Some("SELECT 1")),
            ("mixedTextAddition", None),
            ("sqlAddition", None),
            ("scalarAppend", None),
            ("unownedAppend", None),
            (
                "generatedBinds",
                Some("SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10"),
            ),
            ("authoredBindText", Some("SELECT $1 /* authored $1 */, $2")),
            ("unicodeBindPositions", Some("SELECT $1 /* é */,\n $2")),
            ("strictNullish", Some("SELECT 1")),
            ("looseNullish", Some("SELECT 1")),
            ("unequalBooleans", Some("SELECT 1")),
            ("sameText", Some("SELECT 1")),
            ("distinctTypes", Some("SELECT 1")),
            ("nullishFallback", Some("SELECT 1")),
            ("retainedNullish", Some("SELECT 1")),
            ("booleanFallback", Some("SELECT 1")),
        ],
    );
}
