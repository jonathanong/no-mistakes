use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind, ValueBinding, ValueCall, ValueExpression,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::fx_set;
use std::sync::Arc;

pub(super) fn producer() -> TsFileFacts {
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from("import sql from \"@example/sql\";\n")),
        ..TsFileFacts::default()
    };
    ts.imported_bindings.push(imported(
        "@example/sql",
        "sql",
        "default",
        ImportedBindingKind::Default,
    ));
    ts.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "widgetColumns".to_string(),
        exported: "widgetColumns".to_string(),
    });
    let start = 10;
    let end = 14;
    ts.value_flow.bindings.push(binding(
        "widgetColumns",
        start,
        ValueExpression::Call(start, end),
    ));
    ts.value_flow.calls.push(call(
        start,
        end,
        named("sql"),
        vec![ValueExpression::Unknown],
    ));
    ts.value_flow
        .tagged_templates
        .insert((start, end), Default::default());
    ts
}

pub(super) fn consumer(source: &str, start: u32, end: u32) -> TsFileFacts {
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.imported_bindings.push(imported(
        "./columns",
        "widgetColumns",
        "widgetColumns",
        ImportedBindingKind::Named,
    ));
    ts.value_flow.calls.push(call(
        start,
        end,
        ValueExpression::Member(Arc::new(named("widgetColumns")), "append".to_string()),
        vec![ValueExpression::Unknown],
    ));
    ts
}

pub(super) fn exhausted() -> TsFileFacts {
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from("export const widgetNote = \"kept\";\n")),
        ..TsFileFacts::default()
    };
    ts.value_flow.incomplete_reason = Some("analysis budget".to_string());
    ts
}

fn imported(
    specifier: &str,
    local: &str,
    imported: &str,
    kind: ImportedBindingKind,
) -> ImportedBinding {
    ImportedBinding {
        specifier: specifier.to_string(),
        local: local.to_string(),
        imported: imported.to_string(),
        kind,
        is_type_only: false,
    }
}

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.to_string(),
        scope: None,
        offset: 0,
        context: None,
    }
}

fn binding(name: &str, at: u32, value: ValueExpression) -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: None,
        name: name.to_string(),
        scope: 0,
        declared_at: at,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        value,
    }
}

fn call(
    offset: u32,
    end: u32,
    callee: ValueExpression,
    arguments: Vec<ValueExpression>,
) -> ValueCall {
    ValueCall {
        offset,
        end,
        source_callee: String::new(),
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
