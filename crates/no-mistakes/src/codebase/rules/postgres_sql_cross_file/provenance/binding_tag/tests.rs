use super::super::bindings::Bindings;
use super::is_bind_tag;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    ImportedBinding, ImportedBindingKind, ValueBinding, ValueCall, ValueExpression, ValueFlowFacts,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

fn tag(arguments: Vec<ValueExpression>, callee: ValueExpression) -> ValueCall {
    ValueCall {
        offset: 10,
        end: 90,
        source_callee: "sql".to_string(),
        context: None,
        callee,
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

fn named() -> ValueExpression {
    ValueExpression::Named {
        name: "sql".to_string(),
        scope: Some(1),
        offset: 10,
        context: None,
    }
}

fn local() -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: None,
        name: "sql".to_string(),
        scope: 1,
        declared_at: 0,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: Default::default(),
        value: ValueExpression::Unknown,
    }
}

fn verdict(call: &ValueCall, imports: Vec<ImportedBinding>, bindings: Vec<ValueBinding>) -> bool {
    let file = PathBuf::from("query.ts");
    let ts = TsFileFacts {
        value_flow: ValueFlowFacts {
            bindings,
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
    is_bind_tag(&Bindings::new(&facts), &file, call)
}

const MODULE: &str = "sql-template-strings";

#[test]
fn only_sql_template_strings_tags_bind() {
    let call = tag(vec![ValueExpression::Unknown], named());
    for (imports, expected) in [
        (
            vec![import(MODULE, ImportedBindingKind::Default, "default")],
            true,
        ),
        (
            vec![import(MODULE, ImportedBindingKind::Named, "SQL")],
            true,
        ),
        (
            vec![import(MODULE, ImportedBindingKind::Named, "other")],
            false,
        ),
        (
            vec![import(MODULE, ImportedBindingKind::Namespace, "*")],
            false,
        ),
        (
            vec![import("other", ImportedBindingKind::Default, "default")],
            false,
        ),
        (vec![], false),
    ] {
        assert_eq!(verdict(&call, imports, vec![]), expected);
    }
}

#[test]
fn plain_calls_and_shadowed_tags_do_not_bind() {
    let default = || vec![import(MODULE, ImportedBindingKind::Default, "default")];
    let plain = tag(
        vec![ValueExpression::Awaited(Arc::new(ValueExpression::Unknown))],
        named(),
    );
    assert!(!verdict(&plain, default(), vec![]));
    let empty = tag(vec![], named());
    assert!(!verdict(&empty, default(), vec![]));
    let template = tag(vec![ValueExpression::Unknown], named());
    assert!(!verdict(&template, default(), vec![local()]));
}
