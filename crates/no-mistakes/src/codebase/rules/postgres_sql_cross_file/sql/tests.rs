use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::EmbeddedSqlFileFacts;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use crate::codebase::ts_source::facts::TsFileFacts;
use sqlparser::ast::{Ident, ObjectName, ObjectNamePart, ObjectNamePartFunction};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn call(sql: &str) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        sql_text: Some(sql.to_string()),
        kind: EmbeddedSqlKind::Dynamic,
        ..EmbeddedSqlCall::default()
    }
}

fn compiled() -> super::super::CompiledOptions {
    compile(
        "postgres-no-mutable-sql-registry",
        &Options {
            import_specifier: "@example/db".into(),
            ..Default::default()
        },
    )
    .unwrap()
}

fn embedded(
    path: PathBuf,
    calls: Vec<EmbeddedSqlCall>,
    spans: Vec<(u32, u32)>,
    unproven: Vec<(u32, u32)>,
) -> EmbeddedSqlFileFacts {
    EmbeddedSqlFileFacts {
        path,
        executor_bindings: Vec::new(),
        calls,
        call_spans: spans,
        exported_helpers: Default::default(),
        exported_constants: Default::default(),
        exported_programs: Default::default(),
        pending_imported: Vec::new(),
        imported_call_lines: Default::default(),
        source_comment_spans: Vec::new(),
        fragments: Vec::new(),
        fragment_variants: Vec::new(),
        fragment_sites: Vec::new(),
        matched_factory_names: Vec::new(),
        matched_type_names: Vec::new(),
        pending_relative: Default::default(),
        unproven_member_query_spans: unproven,
        postgres_rule_ids: Vec::new(),
        record_unproven_spans: true,
    }
}

#[test]
fn relation_placeholders_are_identifier_interpolations() {
    assert_eq!(
        identifier_placeholder(&call("SELECT id FROM sql_placeholder_1.users")),
        Some(true)
    );
    assert_eq!(
        identifier_placeholder(&call("SELECT id FROM users WHERE id = sql_placeholder_1")),
        Some(false)
    );
    assert_eq!(identifier_placeholder(&call("not sql")), None);
    assert_eq!(
        identifier_placeholder(&EmbeddedSqlCall {
            kind: EmbeddedSqlKind::Dynamic,
            ..EmbeddedSqlCall::default()
        }),
        None
    );
    assert_eq!(identifier_placeholder(&EmbeddedSqlCall::default()), None);
    assert!(overlapping((1, 4), (3, 6)));
    assert!(!overlapping((1, 2), (2, 4)));
    assert!(!object_has_placeholder(&ObjectName(vec![
        ObjectNamePart::Function(ObjectNamePartFunction {
            name: Ident::new("fn"),
            args: Vec::new(),
        })
    ])));
}

#[test]
fn sql_calls_skip_missing_projections_and_http_query_collisions() {
    let opts = compiled();
    let root = Path::new("/repo");
    let path = PathBuf::from("/repo/src/query.ts");
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        graph_files: vec![path.clone()],
        ..Default::default()
    };
    facts.ts.insert(
        path.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts::default()),
            ..Default::default()
        }
        .into(),
    );
    assert!(sql_calls(root, &opts, &facts).is_empty());
    facts.ts.insert(
        path.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts::default()),
            embedded_sql: vec![(
                opts.embedded.clone(),
                embedded(
                    path.clone(),
                    vec![EmbeddedSqlCall::default()],
                    vec![(1, 8)],
                    vec![(0, 10)],
                ),
            )],
            ..Default::default()
        }
        .into(),
    );
    assert!(sql_calls(root, &opts, &facts).is_empty());
}
