use super::super::{alias, modules, resolve};
use super::support::{catalog, ctx, fragment_facts, insert, world};
use crate::codebase::dependencies::graph::{merge_import_target, ImportTarget};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::sync::Arc;

#[test]
fn recorded_import_targets_replace_neighbor_stems() {
    let mut world = world();
    let columns = world.columns.clone();
    let escape = world.escape.clone();
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "widgets.mts",
        ImportTarget::File(columns.clone()),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "widgets.mts",
        ImportTarget::File(columns.clone()),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "pkg/sql",
        ImportTarget::Unresolved,
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "pkg/sql",
        ImportTarget::File(world.widgets.clone()),
    );
    let catalog = catalog(&world);
    assert_eq!(
        modules::module_file(&world.graph, &world.facts, &catalog, &escape, "widgets.mts"),
        Some(columns),
    );
    assert!(
        modules::module_file(&world.graph, &world.facts, &catalog, &escape, "pkg/sql").is_none()
    );
    let open = alias::hop(
        &catalog,
        &world.graph,
        &world.facts,
        &escape,
        "pkg/sql",
        "orderColumns",
        &mut Vec::new(),
    );
    assert_eq!(
        (open.len(), open[0].open_specifier.as_deref()),
        (1, Some("pkg/sql"))
    );
}

#[test]
fn resolved_tag_import_keeps_a_fragment_exported_under_that_name() {
    let mut world = world();
    let producer = world.columns.clone();
    insert(
        &mut world.facts,
        producer.clone(),
        fragment_facts(&[("default", 10)]),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &producer,
        "@example/sql",
        ImportTarget::File(producer.clone()),
    );
    let catalog = catalog(&world);
    let file_ctx = ctx(&world, &catalog, &producer);
    let found = resolve::name_fragments(&file_ctx, "sql");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].exported, "default");
    assert!(found[0].open_specifier.is_none());

    let mut open_world = super::support::world();
    let producer = open_world.columns.clone();
    insert(
        &mut open_world.facts,
        producer.clone(),
        fragment_facts(&[("default", 10)]),
    );
    let open_catalog = super::support::catalog(&open_world);
    let open_ctx = ctx(&open_world, &open_catalog, &producer);
    assert!(resolve::name_fragments(&open_ctx, "sql").is_empty());
}

#[test]
fn external_targets_are_silent_and_project_gaps_stay_open() {
    let mut world = world();
    let escape = world.escape.clone();
    for specifier in ["node:fs", "orders-widget"] {
        merge_import_target(
            &mut world.graph.import_targets,
            &escape,
            specifier,
            ImportTarget::External,
        );
    }
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "./missing-widget",
        ImportTarget::Unresolved,
    );
    let catalog = catalog(&world);
    for specifier in ["node:fs", "orders-widget"] {
        let found = alias::hop(
            &catalog,
            &world.graph,
            &world.facts,
            &escape,
            specifier,
            "widgetColumns",
            &mut Vec::new(),
        );
        assert!(found.is_empty(), "{specifier}");
    }
    let open = alias::hop(
        &catalog,
        &world.graph,
        &world.facts,
        &escape,
        "./missing-widget",
        "widgetColumns",
        &mut Vec::new(),
    );
    assert_eq!(
        (open.len(), open[0].open_specifier.as_deref()),
        (1, Some("./missing-widget"))
    );
}

#[test]
fn one_ambient_module_names_an_external_specifier() {
    let mut world = world();
    let escape = world.escape.clone();
    let declared = std::path::PathBuf::from("/repo/declared.ts");
    insert(
        &mut world.facts,
        declared.clone(),
        ambient_module("ambient-cols"),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "ambient-cols",
        ImportTarget::External,
    );
    let catalog = catalog(&world);
    assert_eq!(
        modules::module_file(
            &world.graph,
            &world.facts,
            &catalog,
            &escape,
            "ambient-cols"
        ),
        Some(declared)
    );
}

#[test]
fn several_ambient_modules_keep_the_specifier_open() {
    let mut world = world();
    let escape = world.escape.clone();
    insert(
        &mut world.facts,
        std::path::PathBuf::from("/repo/declared-a.ts"),
        ambient_module("orders-widget"),
    );
    insert(
        &mut world.facts,
        std::path::PathBuf::from("/repo/declared-b.ts"),
        ambient_module("orders-widget"),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "orders-widget",
        ImportTarget::External,
    );
    let catalog = catalog(&world);
    let open = alias::hop(
        &catalog,
        &world.graph,
        &world.facts,
        &escape,
        "orders-widget",
        "widgetColumns",
        &mut Vec::new(),
    );
    assert_eq!(
        (open.len(), open[0].open_specifier.as_deref()),
        (1, Some("orders-widget"))
    );
}

fn ambient_module(specifier: &str) -> TsFileFacts {
    TsFileFacts {
        source: Some(Arc::<str>::from(format!(
            "declare module \"{specifier}\" {{}}\n"
        ))),
        ..TsFileFacts::default()
    }
}
