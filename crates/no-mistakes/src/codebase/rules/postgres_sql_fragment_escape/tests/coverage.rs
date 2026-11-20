use super::super::fragments::dedup;
use super::super::{alias, dynamic, modules, names, resolve, scan, tags, visible};
use super::support::{
    assert_from, bind, call, catalog, ctx, fragment_facts, import, named, span, sql_tag, world,
};
use crate::codebase::dependencies::extract::{CallableId, ImportedBindingKind, ValueExpression};
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::config::v2::NoMistakesConfig;
use std::path::Path;
use std::sync::Arc;

#[test]
fn catalog_skips_files_that_cannot_prove_a_fragment() {
    let world = world();
    let catalog = catalog(&world);
    assert_eq!(catalog.by_export("orderColumns").len(), 1);
    let widgets = catalog.by_export("widgetColumns");
    let producers: Vec<&Path> = widgets
        .iter()
        .map(|fragment| fragment.producer.as_path())
        .collect();
    assert!(producers.contains(&world.columns.as_path()));
    assert!(producers.contains(&world.extra.as_path()));
    assert_eq!(producers.len(), 2);
    assert!(catalog.by_export("badColumns").is_empty());
    assert!(catalog.by_export("partialColumns").is_empty());
    let unresolved_order = alias::hop(
        &catalog,
        &world.graph,
        &world.facts,
        &world.escape,
        "missing",
        "orderColumns",
        &mut Vec::new(),
    );
    assert_eq!(unresolved_order.len(), 1);
    assert_eq!(
        unresolved_order[0].open_specifier.as_deref(),
        Some("missing")
    );
    let unresolved_widget = alias::hop(
        &catalog,
        &world.graph,
        &world.facts,
        &world.escape,
        "missing",
        "widgetColumns",
        &mut Vec::new(),
    );
    assert_eq!(unresolved_widget.len(), 1);
    assert_eq!(
        unresolved_widget[0].open_specifier.as_deref(),
        Some("missing")
    );
}

#[test]
fn aliases_stars_and_module_paths_resolve_one_file() {
    let world = world();
    let catalog = catalog(&world);
    let mut aliased = TsFileFacts::default();
    assert!(alias::source_alias(
        &catalog,
        &world.graph,
        &world.facts,
        &world.escape,
        &aliased,
        "orderColumns",
        &mut Vec::new()
    )
    .is_empty());
    aliased = TsFileFacts {
        source: Some(Arc::<str>::from("export const kept = 1;")),
        ..TsFileFacts::default()
    };
    assert!(alias::source_alias(
        &catalog,
        &world.graph,
        &world.facts,
        &world.escape,
        &aliased,
        "orderColumns",
        &mut Vec::new()
    )
    .is_empty());
    aliased = TsFileFacts {
        source: Some(Arc::<str>::from("export { widgetColumns as orderColumns }")),
        ..TsFileFacts::default()
    };
    assert!(alias::source_alias(
        &catalog,
        &world.graph,
        &world.facts,
        &world.escape,
        &aliased,
        "orderColumns",
        &mut Vec::new()
    )
    .is_empty());
    aliased = TsFileFacts {
        source: Some(Arc::<str>::from(
            "import { widgetColumns } from \"./columns\";\nexport { widgetColumns as orderColumns }",
        )),
        ..TsFileFacts::default()
    };
    assert_from(
        &alias::source_alias(
            &catalog,
            &world.graph,
            &world.facts,
            &world.escape,
            &aliased,
            "orderColumns",
            &mut Vec::new(),
        ),
        &world.columns,
        &["widgetColumns"],
    );

    let mut starred = alias::star_fragments(
        &catalog,
        &world.graph,
        &world.facts,
        &world.index,
        &[],
        &mut Vec::new(),
    );
    dedup(&mut starred);
    assert_from(&starred, &world.columns, &["orderColumns", "widgetColumns"]);
    assert!(alias::star_fragments(
        &catalog,
        &world.graph,
        &world.facts,
        &world.index,
        &[],
        &mut vec![world.index.clone()]
    )
    .is_empty());
    assert!(alias::star_fragments(
        &catalog,
        &world.graph,
        &world.facts,
        Path::new("/repo/missing.ts"),
        &[],
        &mut Vec::new()
    )
    .is_empty());

    assert_from(
        &visible::visible(
            &catalog,
            &world.graph,
            &world.facts,
            &world.columns,
            "widgetColumns",
            &mut Vec::new(),
        ),
        &world.columns,
        &["widgetColumns"],
    );
    assert!(visible::visible(
        &catalog,
        &world.graph,
        &world.facts,
        &world.columns,
        "widgetColumns",
        &mut vec![(world.columns.clone(), "widgetColumns".to_string())]
    )
    .is_empty());
    assert_from(
        &visible::visible_all(
            &catalog,
            &world.graph,
            &world.facts,
            &world.index,
            &mut Vec::new(),
        ),
        &world.columns,
        &["orderColumns", "widgetColumns"],
    );
    assert!(visible::visible_all(
        &catalog,
        &world.graph,
        &world.facts,
        &world.index,
        &mut vec![world.index.clone()]
    )
    .is_empty());
    assert!(visible::visible(
        &catalog,
        &world.graph,
        &world.facts,
        Path::new("/repo/missing.ts"),
        "notExported",
        &mut Vec::new()
    )
    .is_empty());
    assert!(visible::visible(
        &catalog,
        &world.graph,
        &world.facts,
        &world.index,
        "notExported",
        &mut Vec::new()
    )
    .is_empty());

    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            &world.escape,
            "./columns"
        ),
        Some(world.columns.clone())
    );
    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            &world.ambient,
            "./orders"
        ),
        Some(world.index.clone())
    );
    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            &world.escape,
            "widgets.mts"
        ),
        None
    );
    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            &world.escape,
            "widget-mod"
        ),
        Some(world.ambient.clone())
    );
    assert_eq!(
        modules::module_file(&world.graph, &world.facts, &catalog, &world.escape, "dir/"),
        None
    );
    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            Path::new("/repo/unknown.ts"),
            "absent"
        ),
        None
    );
    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            Path::new("/"),
            "./columns"
        ),
        None
    );
    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            &world.escape,
            "./../.."
        ),
        None
    );
}

#[test]
fn dynamic_calls_follow_a_resolved_namespace() {
    let world = world();
    let catalog = catalog(&world);
    let ctx = ctx(&world, &catalog, &world.escape);
    let source = world.escape_source.as_str();
    let both = &["orderColumns", "widgetColumns"];
    let append = span(source, "columns[\"widgetColumns\"].append(extra)");
    assert_from(
        &dynamic::dynamic_head(&ctx, &call(append.0, append.1, named("append"), Vec::new())),
        &world.columns,
        &["widgetColumns"],
    );
    let computed = span(source, "columns[idx].append(extra)");
    assert_from(
        &dynamic::dynamic_head(
            &ctx,
            &call(computed.0, computed.1, named("append"), Vec::new()),
        ),
        &world.columns,
        both,
    );
    let missing = span(source, "missing[\"widgetColumns\"].append(extra)");
    assert!(dynamic::dynamic_head(
        &ctx,
        &call(missing.0, missing.1, named("append"), Vec::new())
    )
    .is_empty());
    let empty = span(source, " .append(extra)");
    assert!(
        dynamic::dynamic_head(&ctx, &call(empty.0, empty.1, named("append"), Vec::new()))
            .is_empty()
    );
    let ident = span(source, "1bad[\"x\"].append(extra)");
    assert!(
        dynamic::dynamic_head(&ctx, &call(ident.0, ident.1, named("append"), Vec::new()))
            .is_empty()
    );
    let wrapped = span(source, "f(columns[\"widgetColumns\"])");
    assert_from(
        &dynamic::dynamic_argument(
            &ctx,
            &call(wrapped.0, wrapped.1, named("use"), Vec::new()),
            true,
        ),
        &world.columns,
        &["widgetColumns"],
    );
    assert!(dynamic::dynamic_argument(
        &ctx,
        &call(wrapped.0, wrapped.1, named("use"), Vec::new()),
        false
    )
    .is_empty());
    let backwards = span(source, ")(");
    assert!(dynamic::dynamic_argument(
        &ctx,
        &call(backwards.0, backwards.1, named("use"), Vec::new()),
        true
    )
    .is_empty());

    let callbacks = dynamic::callback_modules(&ctx);
    assert_eq!(callbacks.len(), 2);
    assert_eq!(callbacks.get(&CallableId(4)), Some(&world.columns));
    assert_eq!(callbacks.get(&CallableId(9)), Some(&world.columns));

    assert!(names::namespace_of(&ctx, "columns", 5).is_none());
    assert!(names::expr_module(&ctx, &named("columns"), 5).is_none());
    assert!(names::namespace_of(&ctx, "pending", 0).is_none());
    assert!(names::namespace_of(&ctx, "plain", 0).is_none());
    assert!(names::namespace_of(&ctx, "gone", 0).is_none());
    assert_eq!(
        names::namespace_of(&ctx, "legacy", 0),
        Some(world.columns.clone())
    );
    let imported = span(source, "x.import(\"./no\"); import(\"./columns\")");
    assert_from(
        &names::import_fragments(&ctx, imported.0, imported.1),
        &world.columns,
        both,
    );
    assert!(names::import_fragments(&ctx, 0, 0).is_empty());
    assert!(resolve::name_fragments(&ctx, "orderColumns").is_empty());
    assert_from(
        &resolve::name_fragments(&ctx, "kept"),
        &world.columns,
        &["widgetColumns"],
    );

    let mut depth = ValueExpression::Unknown;
    for _ in 0..5 {
        depth = ValueExpression::Awaited(Arc::new(depth));
    }
    assert!(resolve::call_fragments(&ctx, &call(0, 0, depth, Vec::new())).is_empty());

    let findings = scan::scan(
        Path::new("/repo"),
        &world.tags,
        &world.graph,
        &world.facts,
        None,
    );
    let mut targets: Vec<&str> = findings
        .iter()
        .map(|finding| finding.target.as_deref().unwrap_or(""))
        .collect();
    targets.sort_unstable();
    assert_eq!(
        targets,
        [
            "@example/sql",
            "@example/sql",
            "@example/sql",
            "orderColumns",
            "widgetColumns",
        ]
    );
    assert!(scan::scan(Path::new("/repo"), &[], &world.graph, &world.facts, None).is_empty());
    assert!(scan::scan(
        Path::new("/repo"),
        &[TrustedSqlTag::default()],
        &world.graph,
        &world.facts,
        None
    )
    .is_empty());
    let unconfigured = super::super::check_with_graph(
        Path::new("/repo"),
        &NoMistakesConfig::default(),
        None,
        &world.facts,
        None,
    )
    .expect("unconfigured rule");
    assert!(unconfigured.is_empty());
}

#[test]
fn tags_require_a_matching_sql_import() {
    let mut ts = fragment_facts(&[("widgetColumns", 10)]);
    let binding = ts.value_flow.bindings[0].clone();
    assert!(tags::is_fragment_binding(&ts, &[sql_tag()], &binding));
    assert!(!tags::is_fragment_binding(
        &ts,
        &[TrustedSqlTag::default()],
        &binding
    ));
    let missing = bind("widgetColumns", 90, ValueExpression::Call(90, 94));
    assert!(!tags::is_fragment_binding(&ts, &[sql_tag()], &missing));
    ts.value_flow.calls.push(call(
        70,
        74,
        ValueExpression::Unknown,
        vec![ValueExpression::Unknown],
    ));
    ts.value_flow
        .tagged_templates
        .insert((70, 74), Default::default());
    let unknown = bind("widgetColumns", 70, ValueExpression::Call(70, 74));
    assert!(!tags::is_fragment_binding(&ts, &[sql_tag()], &unknown));

    let mut named_tag = fragment_facts(&[("widgetColumns", 10)]);
    named_tag.imported_bindings[0] =
        import("@example/sql", "sql", "sql", ImportedBindingKind::Named);
    named_tag.value_flow.calls[0].callee = named("sql");
    let proven = named_tag.value_flow.bindings[0].clone();
    let tag = TrustedSqlTag {
        module: "@example/sql".to_string(),
        name: "sql".to_string(),
    };
    assert!(tags::is_fragment_binding(
        &named_tag,
        std::slice::from_ref(&tag),
        &proven
    ));
    named_tag.imported_bindings[0].specifier = "other".to_string();
    assert!(!tags::is_fragment_binding(&named_tag, &[tag], &proven));
}
