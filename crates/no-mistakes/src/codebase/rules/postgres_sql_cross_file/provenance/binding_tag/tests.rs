use super::super::super::SqlCall;
use super::{binds_only, scans_identifier};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    ImportedBinding, ImportedBindingKind, ValueCall, ValueExpression, ValueFlowFacts,
};
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

fn value_call(offset: u32, end: u32, tag: &str, arguments: Vec<ValueExpression>) -> ValueCall {
    ValueCall {
        offset,
        end,
        source_callee: tag.to_string(),
        context: None,
        callee: ValueExpression::Unknown,
        arguments: Arc::new(arguments),
        last_argument_literal: false,
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    }
}

fn import(specifier: &str, kind: ImportedBindingKind, imported: &str) -> ImportedBinding {
    ImportedBinding {
        specifier: specifier.to_string(),
        local: "sql".to_string(),
        imported: imported.to_string(),
        kind,
        is_type_only: false,
    }
}

fn verdict(calls: Vec<ValueCall>, imports: Vec<ImportedBinding>, sql_text: &str) -> (bool, bool) {
    let file = PathBuf::from("query.ts");
    let ts = TsFileFacts {
        value_flow: ValueFlowFacts {
            calls,
            ..ValueFlowFacts::default()
        },
        imported_bindings: imports,
        ..TsFileFacts::default()
    };
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        Arc::new(CheckFileFacts {
            ts: Arc::new(ts),
            ..CheckFileFacts::default()
        }),
    );
    let embedded = EmbeddedSqlCall {
        sql_text: Some(sql_text.to_string()),
        ..EmbeddedSqlCall::default()
    };
    let call = SqlCall {
        file,
        span: (0, 100),
        call: &embedded,
    };
    (binds_only(&facts, &call), scans_identifier(&facts, &call))
}

fn calls(tag_arguments: Vec<ValueExpression>) -> Vec<ValueCall> {
    vec![
        value_call(0, 100, "query", vec![ValueExpression::Call(10, 90)]),
        value_call(10, 90, "sql", tag_arguments),
    ]
}

const TEXT: &str = "SELECT sql_placeholder_1 FROM keys";

#[test]
fn only_sql_template_strings_tags_are_bind_only() {
    let tagged = || calls(vec![ValueExpression::Unknown, ValueExpression::Unknown]);
    let module = "sql-template-strings";
    for (imports, expected) in [
        (
            vec![import(module, ImportedBindingKind::Default, "default")],
            true,
        ),
        (
            vec![import(module, ImportedBindingKind::Named, "SQL")],
            true,
        ),
        (
            vec![import(module, ImportedBindingKind::Named, "other")],
            false,
        ),
        (
            vec![import(module, ImportedBindingKind::Namespace, "*")],
            false,
        ),
        (
            vec![import("other", ImportedBindingKind::Default, "default")],
            false,
        ),
        (vec![], false),
    ] {
        assert_eq!(verdict(tagged(), imports, TEXT), (expected, !expected));
    }
}

#[test]
fn untagged_or_missing_calls_are_not_bind_only() {
    let default = || {
        vec![import(
            "sql-template-strings",
            ImportedBindingKind::Default,
            "default",
        )]
    };
    let plain = calls(vec![ValueExpression::Awaited(Arc::new(
        ValueExpression::Unknown,
    ))]);
    assert_eq!(verdict(plain, default(), TEXT), (false, true));
    let no_inner = vec![value_call(
        0,
        100,
        "query",
        vec![ValueExpression::Call(10, 90)],
    )];
    assert_eq!(verdict(no_inner, default(), TEXT), (false, true));
    let no_argument = vec![value_call(0, 100, "query", vec![])];
    assert_eq!(verdict(no_argument, default(), TEXT), (false, true));
    assert_eq!(verdict(vec![], default(), TEXT), (false, true));
}
