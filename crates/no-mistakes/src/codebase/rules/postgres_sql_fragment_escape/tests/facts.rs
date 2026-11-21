use super::super::resolve::{self, FileCtx};
use super::super::{alias, fragments::Catalog};
use super::support::{assert_from, bind, catalog, import, insert, world};
use crate::codebase::dependencies::extract::{
    CallableId, ExportedBinding, ImportedBindingKind, ValueExpression, ValueParameter,
};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn multi_line_export_alias_uses_import_facts() {
    let world = world();
    let catalog = catalog(&world);
    let source = "import {\n  widgetColumns,\n} from \"./columns\";\nexport { widgetColumns as orderColumns }";
    let mut aliased = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    assert!(alias_of(&world, &catalog, &aliased).is_empty());
    aliased.imported_bindings.push(import(
        "./columns",
        "widgetColumns",
        "widgetColumns",
        ImportedBindingKind::Named,
    ));
    aliased.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "widgetColumns".to_string(),
        exported: "orderColumns".to_string(),
    });
    assert_from(
        &alias_of(&world, &catalog, &aliased),
        &world.columns,
        &["widgetColumns"],
    );
}

fn alias_of<'a>(
    world: &'a super::support::World,
    catalog: &Catalog<'a>,
    ts: &TsFileFacts,
) -> Vec<super::super::fragments::Fragment> {
    alias::source_alias(
        catalog,
        &world.graph,
        &world.facts,
        &world.escape,
        ts,
        "orderColumns",
        &mut Vec::new(),
    )
}

#[test]
fn import_equals_member_uses_facts_when_the_text_is_a_comment() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/kept.ts");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(
            "import kept = /* import kept = decoy.nope */ columns.widgetColumns;",
        )),
        ..TsFileFacts::default()
    };
    ts.imported_bindings.push(import(
        "./columns",
        "columns",
        "*",
        ImportedBindingKind::Namespace,
    ));
    ts.namespaces.import_equals_members.push((
        "kept".to_string(),
        "columns".to_string(),
        "widgetColumns".to_string(),
    ));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert_from(
        &resolve::name_fragments(&ctx, "kept"),
        &world.columns,
        &["widgetColumns"],
    );
}

#[test]
fn literal_import_span_ignores_a_comment_decoy() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/loaded.ts");
    let source = "const parts = import(/* require(\"./orders/items\") */ \"./columns\");";
    let start = u32::try_from(source.find("import(").expect("import")).expect("offset");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.namespaces
        .literal_import_spans
        .push((start, "./columns".to_string()));
    ts.value_flow.bindings.push(super::support::bind(
        "parts",
        0,
        crate::codebase::dependencies::extract::ValueExpression::Call(start, start + 8),
    ));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert_from(
        &resolve::name_fragments(&ctx, "parts"),
        &world.columns,
        &["orderColumns", "widgetColumns"],
    );
}

#[test]
fn arrow_that_imports_is_not_the_module() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/delayed.ts");
    let source = "const delayed = () => import(\"./columns\");\n";
    let import_at = u32::try_from(source.find("import(").expect("import")).expect("offset");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.namespaces
        .literal_import_spans
        .push((import_at, "./columns".to_string()));
    ts.value_flow.bindings.push(bind(
        "delayed",
        0,
        ValueExpression::Callable(CallableId(import_at)),
    ));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert!(
        resolve::name_fragments(&ctx, "delayed").is_empty(),
        "a function that imports a module is not that module"
    );
}

#[test]
fn spaced_import_initializer_still_resolves() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/spaced.ts");
    let source = "const parts = import (\"./columns\");\n";
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.value_flow
        .bindings
        .push(bind("parts", 0, ValueExpression::Unknown));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert_from(
        &resolve::name_fragments(&ctx, "parts"),
        &world.columns,
        &["orderColumns", "widgetColumns"],
    );
}

#[test]
fn computed_import_does_not_take_a_nested_literal() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/picked.ts");
    let source = "const parts = await import(pick(import(\"./columns\")));\n";
    let nested =
        u32::try_from(source.find("import(\"./columns\")").expect("nested")).expect("offset");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.namespaces
        .literal_import_spans
        .push((nested, "./columns".to_string()));
    ts.value_flow
        .bindings
        .push(bind("parts", 0, ValueExpression::Unknown));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert!(
        resolve::name_fragments(&ctx, "parts").is_empty(),
        "a nested literal import does not name the outer computed import"
    );
}

#[test]
fn namespace_reexport_keeps_every_export() {
    let world = world();
    let catalog = catalog(&world);
    let source = "import * as columns from \"./columns\";\nexport { columns as orderColumns };\n";
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    let both = &["orderColumns", "widgetColumns"];
    assert_from(&alias_of(&world, &catalog, &ts), &world.columns, both);
    ts.imported_bindings.push(import(
        "./columns",
        "columns",
        "*",
        ImportedBindingKind::Namespace,
    ));
    ts.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "columns".to_string(),
        exported: "orderColumns".to_string(),
    });
    assert_from(&alias_of(&world, &catalog, &ts), &world.columns, both);
}

#[test]
fn direct_import_initializer_still_resolves() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/loaded.ts");
    let source = "const parts = import(\"./columns\");\n";
    let import_at = u32::try_from(source.find("import(").expect("import")).expect("offset");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.namespaces
        .literal_import_spans
        .push((import_at, "./columns".to_string()));
    ts.value_flow
        .bindings
        .push(bind("parts", 0, ValueExpression::Unknown));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert_from(
        &resolve::name_fragments(&ctx, "parts"),
        &world.columns,
        &["orderColumns", "widgetColumns"],
    );
}

#[test]
fn outer_call_does_not_take_a_nested_import() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/wrapped.ts");
    let source = "const wrapped = makePlainObject(import(\"./columns\"));\n";
    let call = "makePlainObject(import(\"./columns\"))";
    let start = u32::try_from(source.find(call).expect("call")).expect("offset");
    let end = start + u32::try_from(call.len()).expect("end");
    let import_at = u32::try_from(source.find("import(").expect("import")).expect("offset");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.namespaces
        .literal_import_spans
        .push((import_at, "./columns".to_string()));
    ts.value_flow
        .bindings
        .push(bind("wrapped", 0, ValueExpression::Call(start, end)));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert!(
        resolve::name_fragments(&ctx, "wrapped").is_empty(),
        "an outer call is not the module loaded by a nested import"
    );
}

#[test]
fn parameter_does_not_follow_a_shadowed_import_alias() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/shadow.ts");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(
            "import * as columns from \"./columns\";\nimport Widgets = columns;\n",
        )),
        ..TsFileFacts::default()
    };
    ts.imported_bindings.push(import(
        "./columns",
        "columns",
        "*",
        ImportedBindingKind::Namespace,
    ));
    ts.namespaces
        .import_equals_aliases
        .push(("Widgets".to_string(), "columns".to_string()));
    let owner = CallableId(4);
    ts.value_flow.parameters.push(parameter(owner, "Widgets"));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let mut ctx = file_ctx(&world, &catalog, &file);
    assert_from(
        &resolve::name_fragments(&ctx, "Widgets"),
        &world.columns,
        &["orderColumns", "widgetColumns"],
    );
    ctx.context = Some(owner);
    assert!(
        resolve::name_fragments(&ctx, "Widgets").is_empty(),
        "a parameter shadows the file-level import alias"
    );
}

#[test]
fn a_member_require_is_not_the_whole_module() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/version.ts");
    let source = "const version = require(\"./columns\").version;\n";
    let require_at = u32::try_from(source.find("require(").expect("require")).expect("offset");
    let call = "require(\"./columns\")";
    let end = require_at + u32::try_from(call.len()).expect("end");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.namespaces
        .literal_import_spans
        .push((require_at, "./columns".to_string()));
    ts.value_flow.bindings.push(bind(
        "version",
        0,
        ValueExpression::Member(
            Arc::new(ValueExpression::Call(require_at, end)),
            "version".to_string(),
        ),
    ));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert!(
        resolve::name_fragments(&ctx, "version").is_empty(),
        "a member of a required module is not every export"
    );
}

#[test]
fn a_direct_require_still_resolves_the_namespace() {
    let mut world = world();
    let file = PathBuf::from("/repo/orders/loaded.ts");
    let source = "const parts = require(\"./columns\");\n";
    let require_at = u32::try_from(source.find("require(").expect("require")).expect("offset");
    let mut ts = TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    };
    ts.namespaces
        .literal_import_spans
        .push((require_at, "./columns".to_string()));
    ts.value_flow
        .bindings
        .push(bind("parts", 0, ValueExpression::Unknown));
    insert(&mut world.facts, file.clone(), ts);
    let catalog = catalog(&world);
    let ctx = file_ctx(&world, &catalog, &file);
    assert_from(
        &resolve::name_fragments(&ctx, "parts"),
        &world.columns,
        &["orderColumns", "widgetColumns"],
    );
}

fn parameter(owner: CallableId, name: &str) -> ValueParameter {
    ValueParameter {
        owner,
        index: 0,
        name: name.to_string(),
        scope: 0,
        default: ValueExpression::Unknown,
        projection: Vec::new(),
        initialization_order: 0,
        default_order: 0,
        default_mutations: None,
        invalidated_at: None,
        mutation_contexts: crate::fx::fx_set(),
        type_reference: None,
    }
}

fn file_ctx<'a>(
    world: &'a super::support::World,
    catalog: &'a super::super::fragments::Catalog<'a>,
    file: &'a std::path::Path,
) -> FileCtx<'a> {
    let ts = world.facts.get_ts_facts(file).expect("facts");
    FileCtx {
        graph: &world.graph,
        facts: &world.facts,
        catalog,
        tags: &world.tags,
        file,
        ts,
        source: ts.source.as_deref().unwrap_or(""),
        callbacks: &world.callbacks,
        context: None,
    }
}
