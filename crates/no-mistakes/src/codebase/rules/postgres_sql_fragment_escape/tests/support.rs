use super::super::fragments::{Catalog, Fragment};
use super::super::resolve::FileCtx;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    CallableId, ExportedBinding, ImportedBinding, ImportedBindingKind, ValueBinding, ValueCall,
    ValueExpression,
};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, fx_set};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn path(text: &str) -> PathBuf {
    PathBuf::from(text)
}

pub(super) fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.to_string(),
        scope: None,
        offset: 0,
        context: None,
    }
}

pub(super) fn call(
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

pub(super) fn bind(name: &str, at: u32, value: ValueExpression) -> ValueBinding {
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

pub(super) fn import(
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

fn export(specifier: Option<&str>, local: &str, exported: &str) -> ExportedBinding {
    ExportedBinding {
        specifier: specifier.map(str::to_string),
        local: local.to_string(),
        exported: exported.to_string(),
    }
}

pub(super) fn sql_tag() -> TrustedSqlTag {
    TrustedSqlTag {
        module: "@example/sql".to_string(),
        name: "default".to_string(),
    }
}

fn add_fragment(ts: &mut TsFileFacts, name: &str, start: u32) {
    let end = start + 4;
    ts.exported_bindings.push(export(None, name, name));
    ts.value_flow
        .bindings
        .push(bind(name, start, ValueExpression::Call(start, end)));
    ts.value_flow.calls.push(call(
        start,
        end,
        named("sql"),
        vec![ValueExpression::Unknown],
    ));
    ts.value_flow
        .tagged_templates
        .insert((start, end), Default::default());
}

pub(super) fn fragment_facts(names: &[(&str, u32)]) -> TsFileFacts {
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from("import sql from \"@example/sql\";")),
        ..TsFileFacts::default()
    };
    ts.imported_bindings.push(import(
        "@example/sql",
        "sql",
        "default",
        ImportedBindingKind::Default,
    ));
    for (name, start) in names {
        add_fragment(&mut ts, name, *start);
    }
    ts
}

fn names_of(found: &[Fragment]) -> Vec<&str> {
    let mut names: Vec<&str> = found
        .iter()
        .map(|fragment| fragment.exported.as_str())
        .collect();
    names.sort_unstable();
    names
}

pub(super) fn assert_from(found: &[Fragment], producer: &Path, expected: &[&str]) {
    assert_eq!(names_of(found), expected);
    if !expected.is_empty() {
        assert!(found.iter().all(|fragment| fragment.producer == producer));
    }
}

pub(super) fn span(source: &str, needle: &str) -> (u32, u32) {
    let start = source.find(needle).expect(needle);
    (
        u32::try_from(start).expect("offset"),
        u32::try_from(start + needle.len()).expect("end"),
    )
}

pub(super) struct World {
    pub(super) graph: crate::codebase::dependencies::graph::DepGraph,
    pub(super) facts: CheckFactMap,
    pub(super) tags: Vec<TrustedSqlTag>,
    pub(super) columns: PathBuf,
    pub(super) extra: PathBuf,
    pub(super) index: PathBuf,
    pub(super) escape: PathBuf,
    pub(super) widgets: PathBuf,
    pub(super) ambient: PathBuf,
    pub(super) escape_source: String,
    pub(super) callbacks: crate::fx::FxHashMap<CallableId, PathBuf>,
}

pub(super) fn insert(facts: &mut CheckFactMap, file: PathBuf, ts: TsFileFacts) {
    facts.graph_files.push(file.clone());
    facts.ts.insert(
        file,
        Arc::new(CheckFileFacts {
            ts: Arc::new(ts),
            ..CheckFileFacts::default()
        }),
    );
}

fn escape_source() -> String {
    [
        "import * as columns from \"./columns\";",
        "import orderColumns = missing.widgetColumns;",
        "import legacy = require(\"nope\");",
        "import legacy = require(\"./columns\");",
        "export { widgetColumns as orderColumns }",
        "notimport()",
        "x.import(\"./no\"); import(\"./columns\")",
        "columns[\"widgetColumns\"].append(extra)",
        "missing[\"widgetColumns\"].append(extra)",
        " .append(extra)",
        ")(",
        "columns[idx].append(extra)",
        "1bad[\"x\"].append(extra)",
        "f(columns[\"widgetColumns\"])",
        "columns.then(callback)",
        "ghost.then(callback)",
        "columns.then(other)",
        "import kept = columns.widgetColumns;",
        "import(\"./columns\").then(load)",
    ]
    .join("\n")
}

fn escape_facts(source: &str) -> TsFileFacts {
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.imported_bindings.push(import(
        "./columns",
        "columns",
        "*",
        ImportedBindingKind::Namespace,
    ));
    ts.namespaces
        .opaque_specifiers
        .push("./columns".to_string());
    let pending = span(source, "notimport()");
    ts.value_flow.bindings.push(bind(
        "pending",
        pending.0,
        ValueExpression::Call(pending.0, pending.1),
    ));
    let append = span(source, "columns[\"widgetColumns\"].append(extra)");
    ts.value_flow.calls.push(call(
        append.0,
        append.1,
        ValueExpression::Member(Arc::new(named("columns")), "append".to_string()),
        vec![ValueExpression::Unknown],
    ));
    let resolved = span(source, "columns.then(callback)");
    ts.value_flow.calls.push(call(
        resolved.0,
        resolved.1,
        ValueExpression::Member(Arc::new(named("columns")), "then".to_string()),
        vec![ValueExpression::Callable(CallableId(4))],
    ));
    let ghost = span(source, "ghost.then(callback)");
    ts.value_flow.calls.push(call(
        ghost.0,
        ghost.1,
        ValueExpression::Member(Arc::new(named("ghost")), "then".to_string()),
        vec![ValueExpression::Unknown],
    ));
    let other = span(source, "columns.then(other)");
    ts.value_flow.calls.push(call(
        other.0,
        other.1,
        ValueExpression::Member(Arc::new(named("columns")), "then".to_string()),
        vec![ValueExpression::Unknown],
    ));
    let direct = span(source, "x.import(\"./no\"); import(\"./columns\")");
    let keyword = source[direct.0 as usize..]
        .find(" import(")
        .expect("keyword")
        + 1;
    ts.namespaces.literal_import_spans.push((
        direct.0 + u32::try_from(keyword).expect("offset"),
        "./columns".to_string(),
    ));
    let imported = span(source, "import(\"./columns\").then(load)");
    ts.value_flow.calls.push(call(
        imported.0,
        imported.1,
        ValueExpression::Member(Arc::new(ValueExpression::Unknown), "then".to_string()),
        vec![ValueExpression::Callable(CallableId(9))],
    ));
    ts.value_flow.calls.push(call(
        9_000,
        9_004,
        ValueExpression::Member(Arc::new(ValueExpression::Unknown), "then".to_string()),
        vec![ValueExpression::Unknown],
    ));
    ts.value_flow
        .bindings
        .push(bind("plain", 0, ValueExpression::Unknown));
    ts.value_flow
        .bindings
        .push(bind("gone", 9_000, ValueExpression::Call(9_000, 9_004)));
    ts
}

fn index_facts() -> TsFileFacts {
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from("export * from \"./columns\";\n")),
        ..TsFileFacts::default()
    };
    ts.star_reexport_specifiers.push("./columns".to_string());
    ts.imported_bindings.push(import(
        "./columns",
        "orderColumns",
        "orderColumns",
        ImportedBindingKind::Named,
    ));
    ts.exported_bindings
        .push(export(Some("./columns"), "widgetColumns", "kept"));
    ts.exported_bindings
        .push(export(None, "orderColumns", "orderColumns"));
    ts.exported_bindings.push(export(None, "sql", "default"));
    ts
}

pub(super) fn world() -> World {
    let columns = path("/repo/orders/columns.ts");
    let extra = path("/repo/orders/extra.ts");
    let index = path("/repo/orders/index.ts");
    let escape = path("/repo/orders/escape.ts");
    let widgets = path("/repo/widgets/index.ts");
    let ambient = path("/repo/ambient.ts");
    let escape_source = escape_source();
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    insert(
        &mut facts,
        columns.clone(),
        fragment_facts(&[("widgetColumns", 10), ("orderColumns", 30)]),
    );
    insert(
        &mut facts,
        extra.clone(),
        fragment_facts(&[("widgetColumns", 10)]),
    );
    insert(&mut facts, index.clone(), index_facts());
    insert(&mut facts, escape.clone(), escape_facts(&escape_source));
    insert(&mut facts, widgets.clone(), TsFileFacts::default());
    let ambient_facts = TsFileFacts {
        source: Some(Arc::<str>::from("declare module \"widget-mod\" {}\n")),
        ..TsFileFacts::default()
    };
    insert(&mut facts, ambient.clone(), ambient_facts);
    let mut bad = fragment_facts(&[("badColumns", 10)]);
    bad.operational_error = Some("unreadable".to_string());
    insert(&mut facts, path("/repo/bad.ts"), bad);
    let mut incomplete = fragment_facts(&[("partialColumns", 10)]);
    incomplete.value_flow.incomplete_reason = Some("partial".to_string());
    insert(&mut facts, path("/repo/incomplete.ts"), incomplete);
    insert(&mut facts, path("/"), TsFileFacts::default());
    let mut forward = HashMap::new();
    forward.insert(escape.clone(), vec![widgets.clone()]);
    World {
        graph: from_raw_maps(path("/repo"), forward, HashMap::new()),
        facts,
        tags: vec![sql_tag()],
        columns,
        extra,
        index,
        escape,
        widgets,
        ambient,
        escape_source,
        callbacks: fx_map(),
    }
}

pub(super) fn catalog(world: &World) -> Catalog<'_> {
    Catalog::build(&world.facts, &world.tags, None)
}

pub(super) fn ctx<'a>(world: &'a World, catalog: &'a Catalog<'a>, file: &'a Path) -> FileCtx<'a> {
    let ts = world.facts.get_ts_facts(file).expect("facts");
    let source = ts.source.as_deref().unwrap_or(world.escape_source.as_str());
    FileCtx {
        graph: Some(&world.graph),
        facts: &world.facts,
        catalog,
        tags: &world.tags,
        file,
        ts,
        source,
        callbacks: &world.callbacks,
        context: None,
    }
}
