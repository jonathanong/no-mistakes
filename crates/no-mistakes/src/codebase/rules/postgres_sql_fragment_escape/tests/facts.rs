use super::super::resolve::{self, FileCtx};
use super::super::{alias, fragments::Catalog};
use super::support::{assert_from, catalog, import, insert, world};
use crate::codebase::dependencies::extract::{ExportedBinding, ImportedBindingKind};
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
